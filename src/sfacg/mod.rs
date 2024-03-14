mod structure;
mod utils;

use std::{io::Cursor, path::PathBuf};

use chrono::{DateTime, Utc};
use chrono_tz::{Asia::Shanghai, Tz};
use image::{io::Reader, DynamicImage};
use tokio::sync::OnceCell;
use tracing::{error, info};
use url::Url;

use crate::{
    Category, ChapterInfo, Client, ContentInfo, ContentInfos, Error, FindImageResult,
    FindTextResult, HTTPClient, Identifier, NovelDB, NovelInfo, Options, Tag, UserInfo, VolumeInfo,
    VolumeInfos, WordCountRange,
};
use structure::*;

/// Sfacg client, use it to access Apis
#[must_use]
pub struct SfacgClient {
    proxy: Option<Url>,
    no_proxy: bool,
    cert_path: Option<PathBuf>,

    client: OnceCell<HTTPClient>,
    client_rss: OnceCell<HTTPClient>,

    db: OnceCell<NovelDB>,
}

impl Client for SfacgClient {
    fn proxy(&mut self, proxy: Url) {
        self.proxy = Some(proxy);
    }

    fn no_proxy(&mut self) {
        self.no_proxy = true;
    }

    fn cert(&mut self, cert_path: PathBuf) {
        self.cert_path = Some(cert_path);
    }

    async fn shutdown(&self) -> Result<(), Error> {
        self.client().await?.shutdown()
    }

    async fn add_cookie(&self, cookie_str: &str, url: &Url) -> Result<(), Error> {
        self.client().await?.add_cookie(cookie_str, url)
    }

    async fn log_in(&self, username: String, password: Option<String>) -> Result<(), Error> {
        assert!(!username.is_empty());
        assert!(password.is_some());

        let password = password.unwrap();

        let response = self
            .post("/sessions", LogInRequest { username, password })
            .await?
            .json::<GenericResponse>()
            .await?;
        response.status.check()?;

        Ok(())
    }

    async fn logged_in(&self) -> Result<bool, Error> {
        let response = self.get("/user").await?.json::<GenericResponse>().await?;

        if response.status.unauthorized() {
            Ok(false)
        } else {
            response.status.check()?;
            Ok(true)
        }
    }

    async fn user_info(&self) -> Result<UserInfo, Error> {
        let response = self.get("/user").await?.json::<UserInfoResponse>().await?;
        response.status.check()?;
        let data = response.data.unwrap();

        Ok(UserInfo {
            nickname: data.nick_name.trim().to_string(),
            avatar: Some(data.avatar),
        })
    }

    async fn money(&self) -> Result<u32, Error> {
        let response = self
            .get("/user/money")
            .await?
            .json::<MoneyResponse>()
            .await?;
        response.status.check()?;
        let data = response.data.unwrap();

        Ok(data.fire_money_remain + data.coupons_remain)
    }

    async fn sign(&self) -> Result<(), Error> {
        let now: DateTime<Tz> = Utc::now().with_timezone(&Shanghai);

        let response = self
            .put(
                "/user/newSignInfo",
                SignRequest {
                    sign_date: now.format("%Y-%m-%d").to_string(),
                },
            )
            .await?
            .json::<GenericResponse>()
            .await?;
        if response.status.already_signed_in() {
            info!("{}", response.status.msg.unwrap())
        } else {
            response.status.check()?;
        }

        Ok(())
    }

    async fn bookshelf_infos(&self) -> Result<Vec<u32>, Error> {
        let response = self
            .get_query("/user/Pockets", BookshelfInfoRequest { expand: "novels" })
            .await?
            .json::<BookshelfInfoResponse>()
            .await?;
        response.status.check()?;
        let data = response.data.unwrap();

        let mut result = Vec::with_capacity(32);
        for info in data {
            if info.expand.is_some() {
                let novels = info.expand.unwrap().novels;

                if novels.is_some() {
                    for novel_info in novels.unwrap() {
                        result.push(novel_info.novel_id);
                    }
                }
            }
        }

        Ok(result)
    }

    async fn novel_info(&self, id: u32) -> Result<Option<NovelInfo>, Error> {
        assert!(id > 0 && id <= i32::MAX as u32);

        let response = self
            .get_query(
                format!("/novels/{id}"),
                NovelInfoRequest {
                    expand: "intro,typeName,sysTags",
                },
            )
            .await?
            .json::<NovelInfoResponse>()
            .await?;
        if response.status.not_found() {
            return Ok(None);
        }
        response.status.check()?;
        let data = response.data.unwrap();

        let category = Category {
            id: Some(data.type_id),
            parent_id: None,
            name: data.expand.type_name.trim().to_string(),
        };

        let novel_info = NovelInfo {
            id,
            name: data.novel_name.trim().to_string(),
            author_name: data.author_name.trim().to_string(),
            cover_url: Some(data.novel_cover),
            introduction: SfacgClient::parse_intro(data.expand.intro),
            word_count: SfacgClient::parse_word_count(data.char_count),
            is_vip: Some(data.sign_status == "VIP"),
            is_finished: Some(data.is_finish),
            create_time: Some(data.add_time),
            update_time: Some(data.last_update_time),
            category: Some(category),
            tags: self.parse_tags(data.expand.sys_tags).await?,
        };

        Ok(Some(novel_info))
    }

    async fn volume_infos(&self, id: u32) -> Result<Option<VolumeInfos>, Error> {
        assert!(id <= i32::MAX as u32);

        let response = self
            .get(format!("/novels/{id}/dirs"))
            .await?
            .json::<VolumeInfosResponse>()
            .await?;

        if response.status.not_available() {
            return Ok(None);
        }

        response.status.check()?;
        let data = response.data.unwrap();

        let mut volumes = VolumeInfos::with_capacity(8);
        for volume in data.volume_list {
            let mut volume_info = VolumeInfo {
                title: volume.title.trim().to_string(),
                chapter_infos: Vec::with_capacity(volume.chapter_list.len()),
            };

            for chapter in volume.chapter_list {
                let chapter_info = ChapterInfo {
                    novel_id: Some(chapter.novel_id),
                    identifier: Identifier::Id(chapter.chap_id),
                    title: chapter.title.trim().to_string(),
                    word_count: Some(chapter.char_count),
                    create_time: Some(chapter.add_time),
                    update_time: chapter.update_time,
                    is_vip: Some(chapter.is_vip),
                    price: Some(chapter.need_fire_money),
                    is_accessible: Some(chapter.need_fire_money == 0),
                    is_valid: None,
                };

                volume_info.chapter_infos.push(chapter_info);
            }

            volumes.push(volume_info);
        }

        Ok(Some(volumes))
    }

    async fn content_infos(&self, info: &ChapterInfo) -> Result<ContentInfos, Error> {
        let content;

        match self.db().await?.find_text(info).await? {
            FindTextResult::Ok(str) => {
                content = str;
            }
            other => {
                let response = self
                    .get_query(
                        format!("/Chaps/{}", info.identifier.to_string()),
                        ContentInfosRequest {
                            expand: "content,isContentEncrypted",
                        },
                    )
                    .await?
                    .json::<ContentInfosResponse>()
                    .await?;
                response.status.check()?;
                let data = response.data.unwrap();

                // Currently this value is false, it may change in the future
                assert!(
                    !data.expand.is_content_encrypted,
                    "Decryption of encrypted content is not supported"
                );

                content = data.expand.content;
                match other {
                    FindTextResult::None => self.db().await?.insert_text(info, &content).await?,
                    FindTextResult::Outdate => self.db().await?.update_text(info, &content).await?,
                    FindTextResult::Ok(_) => (),
                }
            }
        }

        let mut content_infos = ContentInfos::with_capacity(128);
        for line in content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
        {
            if line.starts_with("[img") {
                match SfacgClient::parse_image_url(line) {
                    Ok(url) => content_infos.push(ContentInfo::Image(url)),
                    Err(err) => error!("{err}"),
                }
            } else {
                content_infos.push(ContentInfo::Text(line.to_string()));
            }
        }

        Ok(content_infos)
    }

    async fn buy_chapter(&self, info: &ChapterInfo) -> Result<(), Error> {
        let Identifier::Id(id) = info.identifier else {
            unreachable!()
        };

        let response = self
            .post(
                &format!("/novels/{}/orderedchaps", info.novel_id.unwrap()),
                BuyChapterRequest {
                    order_all: false,
                    auto_order: false,
                    chap_ids: vec![id],
                    order_type: "readOrder",
                },
            )
            .await?
            .json::<GenericResponse>()
            .await?;
        response.status.check()?;

        Ok(())
    }

    async fn image(&self, url: &Url) -> Result<DynamicImage, Error> {
        match self.db().await?.find_image(url).await? {
            FindImageResult::Ok(image) => Ok(image),
            FindImageResult::None => {
                let response = self.get_rss(url).await?;
                let bytes = response.bytes().await?;

                let image = Reader::new(Cursor::new(&bytes))
                    .with_guessed_format()?
                    .decode()?;

                self.db().await?.insert_image(url, bytes).await?;

                Ok(image)
            }
        }
    }

    async fn categories(&self) -> Result<&Vec<Category>, Error> {
        static CATEGORIES: OnceCell<Vec<Category>> = OnceCell::const_new();

        CATEGORIES
            .get_or_try_init(|| async {
                let response = self
                    .get("/noveltypes")
                    .await?
                    .json::<CategoryResponse>()
                    .await?;
                response.status.check()?;
                let data = response.data.unwrap();

                let mut result = Vec::with_capacity(8);
                for tag_data in data {
                    result.push(Category {
                        id: Some(tag_data.type_id),
                        parent_id: None,
                        name: tag_data.type_name.trim().to_string(),
                    });
                }

                result.sort_unstable_by_key(|x| x.id.unwrap());

                Ok(result)
            })
            .await
    }

    async fn tags(&self) -> Result<&Vec<Tag>, Error> {
        static TAGS: OnceCell<Vec<Tag>> = OnceCell::const_new();

        TAGS.get_or_try_init(|| async {
            let response = self
                .get("/novels/0/sysTags")
                .await?
                .json::<TagResponse>()
                .await?;
            response.status.check()?;
            let data = response.data.unwrap();

            let mut result = Vec::with_capacity(64);
            for tag_data in data {
                result.push(Tag {
                    id: Some(tag_data.sys_tag_id),
                    name: tag_data.tag_name.trim().to_string(),
                });
            }

            // Tag that have been removed, but can still be used
            result.push(Tag {
                id: Some(74),
                name: "百合".to_string(),
            });

            result.sort_unstable_by_key(|x| x.id.unwrap());

            Ok(result)
        })
        .await
    }

    async fn search_infos(
        &self,
        option: &Options,
        page: u16,
        size: u16,
    ) -> Result<Option<Vec<u32>>, Error> {
        if option.keyword.is_some() {
            self.do_search_with_keyword(option, page, size).await
        } else {
            self.do_search_without_keyword(option, page, size).await
        }
    }
}

impl SfacgClient {
    async fn do_search_with_keyword(
        &self,
        option: &Options,
        page: u16,
        size: u16,
    ) -> Result<Option<Vec<u32>>, Error> {
        // 0 连载中
        // 1 已完结
        // -1 不限
        let is_finish = if option.is_finished.is_none() {
            -1
        } else if *option.is_finished.as_ref().unwrap() {
            1
        } else {
            0
        };

        // -1 不限
        let update_days = if option.update_days.is_none() {
            -1
        } else {
            option.update_days.unwrap() as i8
        };

        let response = self
            .get_query(
                "/search/novels/result/new",
                SearchRequest {
                    q: option.keyword.as_ref().unwrap().to_string(),
                    is_finish,
                    update_days,
                    systagids: SfacgClient::tag_ids(&option.tags),
                    page,
                    size,
                    // hot 人气最高
                    // update 最新更新
                    // marknum 收藏最高
                    // ticket 月票最多
                    // charcount 更新最多
                    sort: "hot",
                    expand: "sysTags",
                },
            )
            .await?
            .json::<SearchResponse>()
            .await?;
        response.status.check()?;
        let data = response.data.unwrap();

        if data.novels.is_empty() {
            return Ok(None);
        }

        let mut result = Vec::new();
        let sys_tags = self.tags().await?;

        for novel_info in data.novels {
            let mut tag_ids = vec![];

            for tag in novel_info.expand.sys_tags {
                if let Some(sys_tag) = sys_tags.iter().find(|x| x.id.unwrap() == tag.sys_tag_id) {
                    tag_ids.push(sys_tag.id.unwrap());
                }
            }

            if SfacgClient::match_category(option, novel_info.type_id)
                && SfacgClient::match_excluded_tags(option, tag_ids)
                && SfacgClient::match_vip(option, &novel_info.sign_status)
                && SfacgClient::match_word_count(option, novel_info.char_count)
            {
                result.push(novel_info.novel_id);
            }
        }

        Ok(Some(result))
    }

    async fn do_search_without_keyword(
        &self,
        option: &Options,
        page: u16,
        size: u16,
    ) -> Result<Option<Vec<u32>>, Error> {
        let mut category_id = 0;
        if option.category.is_some() {
            category_id = option.category.as_ref().unwrap().id.unwrap();
        }

        // -1 不限
        let updatedays = if option.update_days.is_none() {
            -1
        } else {
            option.update_days.unwrap() as i8
        };

        let isfinish = SfacgClient::bool_to_str(&option.is_finished);
        let isfree = SfacgClient::bool_to_str(&option.is_vip.as_ref().map(|x| !x));

        let systagids = SfacgClient::tag_ids(&option.tags);
        let notexcludesystagids = SfacgClient::tag_ids(&option.excluded_tags);

        let mut charcountbegin = 0;
        let mut charcountend = 0;

        if option.word_count.is_some() {
            match option.word_count.as_ref().unwrap() {
                WordCountRange::Range(range) => {
                    charcountbegin = range.start;
                    charcountend = range.end;
                }
                WordCountRange::RangeFrom(range_from) => charcountbegin = range_from.start,
                WordCountRange::RangeTo(range_to) => charcountend = range_to.end,
            }
        }

        let response = self
            .get_query(
                format!("/novels/{category_id}/sysTags/novels"),
                NovelsRequest {
                    charcountbegin,
                    charcountend,
                    isfinish,
                    isfree,
                    systagids,
                    notexcludesystagids,
                    updatedays,
                    page,
                    size,
                    // latest 最新更新
                    // viewtimes 人气最高
                    // bookmark 收藏最高
                    // ticket 月票最多
                    // charcount 更新最多
                    sort: "viewtimes",
                },
            )
            .await?
            .json::<NovelsResponse>()
            .await?;
        response.status.check()?;
        let data = response.data.unwrap();

        if data.is_empty() {
            return Ok(None);
        }

        let mut result = Vec::new();
        for novel_data in data {
            result.push(novel_data.novel_id);
        }

        Ok(Some(result))
    }

    fn parse_word_count(word_count: i32) -> Option<u32> {
        // Some novels have negative word counts
        if word_count <= 0 {
            None
        } else {
            Some(word_count as u32)
        }
    }

    async fn parse_tags(&self, tag_list: Vec<NovelInfoSysTag>) -> Result<Option<Vec<Tag>>, Error> {
        let sys_tags = self.tags().await?;

        let mut result = Vec::new();
        for tag in tag_list {
            let id = tag.sys_tag_id;
            let name = tag.tag_name.trim().to_string();

            // Remove non-system tags
            if sys_tags.iter().any(|sys_tag| sys_tag.id.unwrap() == id) {
                result.push(Tag { id: Some(id), name });
            } else {
                info!("This tag is not a system tag and is ignored: {name}");
            }
        }

        if result.is_empty() {
            Ok(None)
        } else {
            result.sort_unstable_by_key(|x| x.id.unwrap());
            Ok(Some(result))
        }
    }

    fn parse_intro(intro: String) -> Option<Vec<String>> {
        let introduction = intro
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect::<Vec<String>>();

        if introduction.is_empty() {
            None
        } else {
            Some(introduction)
        }
    }

    fn parse_image_url(line: &str) -> Result<Url, Error> {
        let begin = line.find("http");
        let end = line.find("[/img]");

        if begin.is_none() || end.is_none() {
            return Err(Error::NovelApi(format!(
                "Image URL format is incorrect: {line}"
            )));
        }

        let begin = begin.unwrap();
        let end = end.unwrap();

        let url = line
            .chars()
            .skip(begin)
            .take(end - begin)
            .collect::<String>()
            .trim()
            .to_string();

        match Url::parse(&url) {
            Ok(url) => Ok(url),
            Err(error) => Err(Error::NovelApi(format!(
                "Image URL parse failed: {error}, content: {line}"
            ))),
        }
    }

    fn bool_to_str(flag: &Option<bool>) -> &'static str {
        if flag.is_some() {
            if *flag.as_ref().unwrap() {
                "is"
            } else {
                "not"
            }
        } else {
            "both"
        }
    }

    fn tag_ids(tags: &Option<Vec<Tag>>) -> Option<String> {
        tags.as_ref().map(|tags| {
            tags.iter()
                .map(|tag| tag.id.unwrap().to_string())
                .collect::<Vec<String>>()
                .join(",")
        })
    }

    fn match_vip(option: &Options, sign_status: &str) -> bool {
        if option.is_vip.is_none() {
            return true;
        }

        if *option.is_vip.as_ref().unwrap() {
            sign_status == "VIP"
        } else {
            sign_status != "VIP"
        }
    }

    fn match_excluded_tags(option: &Options, tag_ids: Vec<u16>) -> bool {
        if option.excluded_tags.is_none() {
            return true;
        }

        tag_ids.iter().all(|id| {
            !option
                .excluded_tags
                .as_ref()
                .unwrap()
                .iter()
                .any(|tag| tag.id.unwrap() == *id)
        })
    }

    fn match_category(option: &Options, category_id: u16) -> bool {
        !option
            .category
            .as_ref()
            .is_some_and(|category| category.id.unwrap() != category_id)
    }

    fn match_word_count(option: &Options, word_count: i32) -> bool {
        if option.word_count.is_none() {
            return true;
        }

        if word_count <= 0 {
            return true;
        }

        let word_count = word_count as u32;
        match option.word_count.as_ref().unwrap() {
            WordCountRange::Range(range) => {
                if word_count >= range.start && word_count < range.end {
                    return true;
                }
            }
            WordCountRange::RangeFrom(range_from) => {
                if word_count >= range_from.start {
                    return true;
                }
            }
            WordCountRange::RangeTo(rang_to) => {
                if word_count < rang_to.end {
                    return true;
                }
            }
        }

        false
    }
}
