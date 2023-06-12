mod structure;
mod utils;

use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use image::{io::Reader, DynamicImage};
use tokio::sync::OnceCell;
use tracing::error;
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

#[async_trait]
impl Client for SfacgClient {
    fn proxy(&mut self, proxy: Url) {
        self.proxy = Some(proxy);
    }

    fn no_proxy(&mut self) {
        self.no_proxy = true;
    }

    fn cert<T>(&mut self, cert_path: T)
    where
        T: AsRef<Path>,
    {
        self.cert_path = Some(cert_path.as_ref().to_path_buf());
    }

    async fn shutdown(&self) -> Result<(), Error> {
        self.client().await?.shutdown()
    }

    async fn add_cookie(&self, cookie_str: &str, url: &Url) -> Result<(), Error> {
        Ok(self.client().await?.add_cookie(cookie_str, url)?)
    }

    async fn login(&self, username: String, password: String) -> Result<(), Error> {
        let response = self
            .post(
                "/sessions",
                &LoginRequest {
                    user_name: username,
                    pass_word: password,
                },
            )
            .await?
            .json::<LoginResponse>()
            .await?;
        response.status.check()?;

        // TODO Is it really necessary?
        let response = self
            .get("/position")
            .await?
            .json::<PositionResponse>()
            .await?;
        response.status.check()?;

        Ok(())
    }

    async fn user_info(&self) -> Result<Option<UserInfo>, Error> {
        let response = self.get("/user").await?.json::<UserResponse>().await?;
        if response.status.unauthorized() {
            return Ok(None);
        }
        response.status.check()?;

        let user_info = UserInfo {
            nickname: response.data.unwrap().nick_name.trim().to_string(),
        };

        Ok(Some(user_info))
    }

    async fn novel_info(&self, id: u32) -> Result<Option<NovelInfo>, Error> {
        assert!(id <= i32::MAX as u32);

        let response = self
            .get_query(
                format!("/novels/{id}"),
                &NovelInfoRequest {
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

        let novel_data = response.data.unwrap();

        let word_count = if novel_data.char_count <= 0 {
            None
        } else {
            Some(novel_data.char_count as u32)
        };

        let category = Category {
            id: Some(novel_data.type_id),
            name: novel_data.expand.type_name.trim().to_string(),
        };

        let novel_info = NovelInfo {
            id,
            name: novel_data.novel_name.trim().to_string(),
            author_name: novel_data.author_name.trim().to_string(),
            cover_url: Some(novel_data.novel_cover),
            introduction: SfacgClient::parse_intro(novel_data.expand.intro),
            word_count,
            is_finished: Some(novel_data.is_finish),
            create_time: Some(novel_data.add_time),
            update_time: Some(novel_data.last_update_time),
            category: Some(category),
            tags: SfacgClient::parse_tags(novel_data.expand.sys_tags),
        };

        Ok(Some(novel_info))
    }

    async fn volume_infos(&self, id: u32) -> Result<VolumeInfos, Error> {
        assert!(id <= i32::MAX as u32);

        let response = self
            .get(format!("/novels/{id}/dirs"))
            .await?
            .json::<NovelsDirsResponse>()
            .await?;
        response.status.check()?;

        let mut volumes = VolumeInfos::new();
        for volume in response.data.unwrap().volume_list {
            let mut volume_info = VolumeInfo {
                title: volume.title.trim().to_string(),
                chapter_infos: vec![],
            };

            for chapter in volume.chapter_list {
                let update_time = if chapter.update_time.is_some() {
                    chapter.update_time
                } else {
                    Some(chapter.add_time)
                };

                let word_count = if chapter.char_count <= 0 {
                    None
                } else {
                    Some(chapter.char_count as u16)
                };

                let chapter_info = ChapterInfo {
                    identifier: Identifier::Id(chapter.chap_id),
                    title: chapter.title.trim().to_string(),
                    word_count,
                    update_time,
                    is_vip: Some(chapter.is_vip),
                    is_accessible: Some(chapter.need_fire_money == 0),
                    is_valid: None,
                };

                volume_info.chapter_infos.push(chapter_info);
            }

            volumes.push(volume_info);
        }

        Ok(volumes)
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
                        &ChapsRequest { expand: "content" },
                    )
                    .await?
                    .json::<ChapsResponse>()
                    .await?;
                response.status.check()?;

                content = response.data.unwrap().expand.content;

                match other {
                    FindTextResult::None => self.db().await?.insert_text(info, &content).await?,
                    FindTextResult::Outdate => self.db().await?.update_text(info, &content).await?,
                    FindTextResult::Ok(_) => (),
                }
            }
        }

        let mut content_infos = ContentInfos::new();
        for line in content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
        {
            if line.starts_with("[img") {
                if let Some(url) = SfacgClient::parse_image_url(line) {
                    content_infos.push(ContentInfo::Image(url));
                }
            } else {
                content_infos.push(ContentInfo::Text(line.to_string()));
            }
        }

        Ok(content_infos)
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

    async fn search_infos<T>(&self, text: T, page: u16, size: u16) -> Result<Vec<u32>, Error>
    where
        T: AsRef<str> + Send + Sync,
    {
        let response = self
            .get_query(
                "/search/novels/result/new",
                &SearchRequest {
                    page,
                    q: text.as_ref().to_string(),
                    size,
                    sort: "hot",
                },
            )
            .await?
            .json::<SearchResponse>()
            .await?;
        response.status.check()?;

        let mut result = Vec::new();
        if response.data.is_some() {
            for novel_info in response.data.unwrap().novels {
                result.push(novel_info.novel_id);
            }
        }

        Ok(result)
    }

    async fn bookshelf_infos(&self) -> Result<Vec<u32>, Error> {
        let response = self
            .get_query(
                "/user/Pockets",
                &BookshelfRequest {
                    expand: "novels,albums,comics",
                },
            )
            .await?
            .json::<BookshelfResponse>()
            .await?;
        response.status.check()?;

        let mut result = Vec::new();
        if response.data.is_some() {
            for data in response.data.unwrap() {
                if let BookshelfExpand::Novels(novels) = data.expand {
                    for novel_info in novels {
                        result.push(novel_info.novel_id);
                    }
                }
            }
        }

        Ok(result)
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

                let mut result = Vec::new();

                for tag_data in response.data.unwrap() {
                    result.push(Category {
                        id: Some(tag_data.type_id),
                        name: tag_data.type_name.trim().to_string(),
                    });
                }

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

            let mut result = Vec::new();

            for tag_data in response.data.unwrap() {
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

            Ok(result)
        })
        .await
    }

    async fn novels(&self, option: &Options, page: u16, size: u16) -> Result<Vec<u32>, Error> {
        let mut category_id = 0;
        if option.category.is_some() {
            category_id = option.category.as_ref().unwrap().id.unwrap();
        }

        let is_finish = SfacgClient::bool_to_str(&option.is_finished);
        let is_free = SfacgClient::bool_to_str(&option.is_vip.as_ref().map(|x| !x));

        let sys_tag_ids = SfacgClient::tag_ids(&option.tags);
        let not_exclude_sys_tag_ids = SfacgClient::tag_ids(&option.excluded_tags);

        let mut char_count_begin = 0;
        let mut char_count_end = 0;

        if option.word_count.is_some() {
            match option.word_count.as_ref().unwrap() {
                WordCountRange::Range(range) => {
                    char_count_begin = range.start;
                    char_count_end = range.end;
                }
                WordCountRange::RangeFrom(range_from) => char_count_begin = range_from.start,
                WordCountRange::RangeTo(range_to) => char_count_end = range_to.end,
            }
        }

        let response = self
            .get_query(
                format!("/novels/{category_id}/sysTags/novels"),
                &NovelsRequest {
                    fields: "novelId",
                    char_count_begin,
                    char_count_end,
                    is_finish,
                    is_free,
                    sys_tag_ids,
                    not_exclude_sys_tag_ids,
                    updatedays: option.update_days,
                    page,
                    size,
                    sort: "viewtimes",
                },
            )
            .await?
            .json::<NovelsResponse>()
            .await?;
        response.status.check()?;

        let mut result = Vec::new();
        if response.data.is_some() {
            for novel_data in response.data.unwrap() {
                result.push(novel_data.novel_id);
            }
        }

        Ok(result)
    }
}

impl SfacgClient {
    fn parse_tags(sys_tags: Vec<NovelInfoSysTag>) -> Option<Vec<Tag>> {
        let mut result = vec![];
        for tag in sys_tags {
            result.push(Tag {
                id: Some(tag.sys_tag_id),
                name: tag.tag_name.trim().to_string(),
            });
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
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

    fn parse_image_url(line: &str) -> Option<Url> {
        let begin = line.find("http");
        let end = line.find("[/img]");

        if begin.is_none() || end.is_none() {
            error!("Image URL format is incorrect: {line}");
            return None;
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
            Ok(url) => Some(url),
            Err(error) => {
                error!("Image URL parse failed: {error}, content: {line}");
                None
            }
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
}
