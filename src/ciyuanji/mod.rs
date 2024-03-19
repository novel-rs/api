mod structure;
mod utils;

use std::{io::Cursor, path::PathBuf, sync::RwLock};

use chrono::{Duration, Local, NaiveDateTime, TimeZone};
use chrono_tz::Asia::Shanghai;
use image::{io::Reader, DynamicImage};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use tracing::{error, info};
use url::Url;

use crate::{
    Category, ChapterInfo, Client, ContentInfo, ContentInfos, Error, FindImageResult,
    FindTextResult, HTTPClient, Identifier, NovelDB, NovelInfo, Options, Tag, UserInfo, VolumeInfo,
    VolumeInfos, WordCountRange,
};
use structure::*;

#[must_use]
#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    token: String,
}

/// Ciyuanji client, use it to access Apis
#[must_use]
pub struct CiyuanjiClient {
    proxy: Option<Url>,
    no_proxy: bool,
    cert_path: Option<PathBuf>,

    client: OnceCell<HTTPClient>,
    client_rss: OnceCell<HTTPClient>,

    db: OnceCell<NovelDB>,

    config: RwLock<Option<Config>>,
}

impl Client for CiyuanjiClient {
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
        self.client().await?.shutdown()?;
        self.do_shutdown()?;
        Ok(())
    }

    async fn add_cookie(&self, cookie_str: &str, url: &Url) -> Result<(), Error> {
        self.client().await?.add_cookie(cookie_str, url)
    }

    async fn log_in(&self, username: String, password: Option<String>) -> Result<(), Error> {
        assert!(!username.is_empty());
        assert!(password.is_none());

        let response = self
            .post(
                "/login/getPhoneCode",
                PhoneCodeRequest {
                    phone: username.clone(),
                    // always 1
                    sms_type: "1",
                },
            )
            .await?
            .json::<GenericResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;

        let response = self
            .post(
                "/login/phone",
                LoginRequest {
                    phone: username,
                    phone_code: crate::input("Please enter SMS verification code")?,
                },
            )
            .await?
            .json::<LoginResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;

        self.save_token(Config {
            token: response.data.user_info.unwrap().token,
        });

        Ok(())
    }

    async fn logged_in(&self) -> Result<bool, Error> {
        if !self.has_token() {
            return Ok(false);
        }

        let response = self
            .get("/user/getUserInfo")
            .await?
            .json::<GenericResponse>()
            .await?;

        if response.code == CiyuanjiClient::FAILED {
            Ok(false)
        } else {
            utils::check_response_success(response.code, response.msg)?;
            Ok(true)
        }
    }

    async fn user_info(&self) -> Result<UserInfo, Error> {
        let response = self
            .get("/user/getUserInfo")
            .await?
            .json::<UserInfoResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;
        let cm_user = response.data.cm_user.unwrap();

        let user_info = UserInfo {
            nickname: cm_user.nick_name.trim().to_string(),
            avatar: Some(cm_user.img_url),
        };

        Ok(user_info)
    }

    async fn money(&self) -> Result<u32, Error> {
        let response = self
            .get("/account/getAccountByUser")
            .await?
            .json::<MoneyResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;
        let account_info = response.data.account_info.unwrap();

        Ok(account_info.currency_balance + account_info.coupon_balance)
    }

    async fn sign(&self) -> Result<(), Error> {
        let response = self
            .post("/sign/sign", EmptyRequest {})
            .await?
            .json::<GenericResponse>()
            .await?;
        if utils::check_already_signed_in(&response.code, &response.msg) {
            info!("{}", CiyuanjiClient::ALREADY_SIGNED_IN_MSG);
        } else {
            utils::check_response_success(response.code, response.msg)?;
        }

        Ok(())
    }

    async fn bookshelf_infos(&self) -> Result<Vec<u32>, Error> {
        let response = self
            .get_query(
                "/bookrack/getUserBookRackList",
                BookSelfRequest {
                    page_no: 1,
                    page_size: 9999,
                    // 1 阅读
                    // 2 更新
                    rank_type: 1,
                },
            )
            .await?
            .json::<BookSelfResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;

        let mut result = Vec::new();

        for item in response.data.book_rack_list.unwrap() {
            result.push(item.book_id);
        }

        Ok(result)
    }

    async fn novel_info(&self, id: u32) -> Result<Option<NovelInfo>, Error> {
        assert!(id > 0);

        let response = self
            .get_query(
                "/book/getBookDetail",
                BookDetailRequest {
                    book_id: id.to_string(),
                },
            )
            .await?
            .json::<BookDetailResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;

        if response.data.book.is_none() {
            return Ok(None);
        }

        let book = response.data.book.unwrap();

        // 该书不存在
        if book.book_id == 0 {
            return Ok(None);
        }

        let category = if book.second_classify.is_some() {
            Some(Category {
                id: Some(book.second_classify.unwrap()),
                parent_id: Some(book.first_classify.unwrap()),
                name: format!(
                    "{}-{}",
                    book.first_classify_name.unwrap().trim(),
                    book.second_classify_name.unwrap().trim()
                ),
            })
        } else if book.first_classify.is_some() {
            Some(Category {
                id: Some(book.first_classify.unwrap()),
                parent_id: None,
                name: book.first_classify_name.unwrap().trim().to_string(),
            })
        } else {
            None
        };

        let novel_info = NovelInfo {
            id,
            name: book.book_name.unwrap().trim().to_string(),
            author_name: book.author_name.unwrap().trim().to_string(),
            cover_url: book.img_url,
            introduction: CiyuanjiClient::parse_intro(book.notes.unwrap()),
            word_count: CiyuanjiClient::parse_word_count(book.word_count),
            is_vip: Some(book.is_vip.unwrap() == "1"),
            is_finished: Some(book.end_state.unwrap() == "1"),
            create_time: None,
            update_time: book.latest_update_time,
            category,
            tags: self.parse_tags(book.tag_list.unwrap()).await?,
        };

        Ok(Some(novel_info))
    }

    async fn volume_infos(&self, id: u32) -> Result<Option<VolumeInfos>, Error> {
        let response = self
            .get_query(
                "/chapter/getChapterListByBookId",
                VolumeInfosRequest {
                    // 1 正序
                    // 2 倒序
                    sort_type: "1",
                    page_no: "1",
                    page_size: "9999",
                    book_id: id.to_string(),
                },
            )
            .await?
            .json::<ChapterListResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;

        let mut volumes = VolumeInfos::new();

        let mut last_volume_id = 0;
        let book_chapter = response.data.book_chapter.unwrap();

        let book_id = book_chapter.book_id;

        if book_chapter.chapter_list.is_some() {
            for chapter in book_chapter.chapter_list.unwrap() {
                let volume_title = chapter.title.unwrap_or_default().trim().to_string();

                if chapter.volume_id != last_volume_id {
                    last_volume_id = chapter.volume_id;

                    volumes.push(VolumeInfo {
                        title: volume_title.clone(),
                        chapter_infos: Vec::new(),
                    })
                }

                let last_volume_title = &mut volumes.last_mut().unwrap().title;
                if last_volume_title.is_empty() && !volume_title.is_empty() {
                    *last_volume_title = volume_title;
                }

                let chapter_info = ChapterInfo {
                    novel_id: Some(book_id),
                    identifier: Identifier::Id(chapter.chapter_id),
                    title: chapter.chapter_name.trim().to_string(),
                    is_vip: Some(chapter.is_fee == "1"),
                    // 去除小数部分
                    price: Some(chapter.price.parse::<f64>().unwrap() as u16),
                    payment_required: Some(
                        chapter.is_fee == "1" && chapter.is_buy == "0",
                    ),
                    is_valid: None,
                    word_count: /*CiyuanjiClient::parse_word_count(chapter.word_count)*/Some(chapter.word_count),
                    create_time: Some(chapter.publish_time),
                    update_time: None,
                };

                volumes.last_mut().unwrap().chapter_infos.push(chapter_info);
            }
        }

        Ok(Some(volumes))
    }

    async fn content_infos(&self, info: &ChapterInfo) -> Result<ContentInfos, Error> {
        let mut content;

        match self.db().await?.find_text(info).await? {
            FindTextResult::Ok(str) => {
                content = str;
            }
            other => {
                let response = self
                    .get_query(
                        "/chapter/getChapterContent",
                        ContentRequest {
                            book_id: info.novel_id.unwrap().to_string(),
                            chapter_id: info.identifier.to_string(),
                        },
                    )
                    .await?
                    .json::<ContentResponse>()
                    .await?;
                utils::check_response_success(response.code, response.msg)?;
                let chapter = response.data.chapter.unwrap();

                content = crate::des_ecb_base64_decrypt(
                    CiyuanjiClient::DES_KEY,
                    chapter.content.replace('\n', ""),
                )?;

                if !chapter.img_list.is_empty() {
                    let mut content_lines: Vec<_> =
                        content.lines().map(|x| x.to_string()).collect();

                    for img in chapter.img_list {
                        let image_str = format!("[img]{}[/img]", img.img_url);
                        content_lines.insert(img.paragraph_index, image_str);
                    }

                    content = content_lines.join("\n");
                }

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
                if let Some(url) = CiyuanjiClient::parse_image_url(line) {
                    content_infos.push(ContentInfo::Image(url));
                }
            } else {
                content_infos.push(ContentInfo::Text(line.to_string()));
            }
        }

        Ok(content_infos)
    }

    async fn buy_chapter(&self, info: &ChapterInfo) -> Result<(), Error> {
        let response = self
            .post(
                "/order/consume",
                BuyChapterRequest {
                    // always 2
                    view_type: "2",
                    // always 1
                    consume_type: "1",
                    book_id: info.novel_id.unwrap().to_string(),
                    product_id: info.identifier.to_string(),
                    buy_count: "1",
                },
            )
            .await?
            .json::<GenericResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;

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
                let mut result = Vec::with_capacity(32);

                // 1 男生
                // 2 漫画
                // 4 女生
                self.get_categories("1", &mut result).await?;
                self.get_categories("4", &mut result).await?;

                result.sort_unstable_by_key(|x| x.id.unwrap());

                Ok(result.into_iter().dedup().collect_vec())
            })
            .await
    }

    async fn tags(&self) -> Result<&Vec<Tag>, Error> {
        static TAGS: OnceCell<Vec<Tag>> = OnceCell::const_new();

        TAGS.get_or_try_init(|| async {
            let mut result = Vec::with_capacity(64);

            self.get_tags(1, &mut result).await?;
            self.get_tags(4, &mut result).await?;

            result.push(Tag {
                id: Some(17),
                name: String::from("无限流"),
            });
            result.push(Tag {
                id: Some(26),
                name: String::from("变身"),
            });
            result.push(Tag {
                id: Some(30),
                name: String::from("百合"),
            });
            result.push(Tag {
                id: Some(96),
                name: String::from("变百"),
            });
            result.push(Tag {
                id: Some(127),
                name: String::from("性转"),
            });
            result.push(Tag {
                id: Some(570),
                name: String::from("纯百"),
            });
            result.push(Tag {
                id: Some(1431),
                name: String::from("复仇"),
            });
            result.push(Tag {
                id: Some(1512),
                name: String::from("魔幻"),
            });
            result.push(Tag {
                id: Some(5793),
                name: String::from("少女"),
            });

            result.sort_unstable_by_key(|x| x.id.unwrap());

            Ok(result.into_iter().dedup().collect_vec())
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

impl CiyuanjiClient {
    async fn do_search_with_keyword(
        &self,
        option: &Options,
        page: u16,
        size: u16,
    ) -> Result<Option<Vec<u32>>, Error> {
        let (start_word, end_word) = CiyuanjiClient::to_word(option);
        let (first_classify, _) = CiyuanjiClient::to_classify_ids(option);

        let response = self
            .get_query(
                "/book/searchBookList",
                SearchBookListRequest {
                    page_no: page + 1,
                    page_size: size,
                    // 0 按推荐
                    // 1 按人气
                    // 2 按销量
                    // 3 按更新
                    rank_type: "1",
                    keyword: option.keyword.as_ref().unwrap().to_string(),
                    is_fee: CiyuanjiClient::to_is_fee(option),
                    end_state: CiyuanjiClient::to_end_state(option),
                    start_word,
                    end_word,
                    classify_ids: first_classify,
                },
            )
            .await?
            .json::<SearchBookListResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;
        let es_book_list = response.data.es_book_list.unwrap();

        if es_book_list.is_empty() {
            return Ok(None);
        }

        let mut result = Vec::new();
        let sys_tags = self.tags().await?;

        for novel_info in es_book_list {
            let mut tag_ids = Vec::new();

            if novel_info.tag_name.is_some() {
                let tag_names = novel_info
                    .tag_name
                    .unwrap()
                    .split(',')
                    .map(|x| x.trim().to_string())
                    .filter(|x| !x.is_empty())
                    .collect_vec();

                for tag_name in tag_names {
                    if let Some(tag) = sys_tags.iter().find(|x| x.name == tag_name) {
                        tag_ids.push(tag.id.unwrap());
                    }
                }
            }

            if CiyuanjiClient::match_update_days(option, novel_info.latest_update_time)
                && CiyuanjiClient::match_tags(option, &tag_ids)
                && CiyuanjiClient::match_excluded_tags(option, &tag_ids)
                && CiyuanjiClient::match_category(
                    option,
                    novel_info.first_classify,
                    novel_info.second_classify,
                )
            {
                result.push(novel_info.book_id);
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
        let (start_word, end_word) = CiyuanjiClient::to_word(option);
        let (first_classify, second_classify) = CiyuanjiClient::to_classify_ids(option);

        let response = self
            .get_query(
                "/book/getBookListByParams",
                BookListRequest {
                    page_no: page + 1,
                    page_size: size,
                    // 1 人气最高
                    // 2 订阅最多
                    // 3 最近更新
                    // 4 最近上架
                    // 6 最近新书
                    rank_type: "1",
                    first_classify,
                    second_classify,
                    start_word,
                    end_word,
                    is_fee: CiyuanjiClient::to_is_fee(option),
                    end_state: CiyuanjiClient::to_end_state(option),
                },
            )
            .await?
            .json::<BookListResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;
        let book_list = response.data.book_list.unwrap();

        if book_list.is_empty() {
            return Ok(None);
        }

        let mut result = Vec::new();
        for novel_info in book_list {
            let mut tag_ids = Vec::new();
            if novel_info.tag_list.is_some() {
                for tags in novel_info.tag_list.unwrap() {
                    tag_ids.push(tags.tag_id);
                }
            }

            if CiyuanjiClient::match_update_days(option, novel_info.latest_update_time)
                && CiyuanjiClient::match_tags(option, &tag_ids)
                && CiyuanjiClient::match_excluded_tags(option, &tag_ids)
            {
                result.push(novel_info.book_id);
            }
        }

        Ok(Some(result))
    }

    fn to_end_state(option: &Options) -> Option<String> {
        option.is_finished.map(|x| {
            if x {
                String::from("1")
            } else {
                String::from("2")
            }
        })
    }

    fn to_is_fee(option: &Options) -> Option<String> {
        option.is_vip.map(|x| {
            if x {
                String::from("1")
            } else {
                String::from("0")
            }
        })
    }

    fn to_word(option: &Options) -> (Option<String>, Option<String>) {
        let mut start_word = None;
        let mut end_word = None;

        if option.word_count.is_some() {
            match option.word_count.as_ref().unwrap() {
                WordCountRange::Range(range) => {
                    start_word = Some(range.start.to_string());
                    end_word = Some(range.end.to_string());
                }
                WordCountRange::RangeFrom(range_from) => {
                    start_word = Some(range_from.start.to_string())
                }
                WordCountRange::RangeTo(range_to) => end_word = Some(range_to.end.to_string()),
            }
        }

        (start_word, end_word)
    }

    fn to_classify_ids(option: &Options) -> (Option<String>, Option<String>) {
        let mut first_classify = None;
        let mut second_classify = None;

        if option.category.is_some() {
            let category = option.category.as_ref().unwrap();

            if category.parent_id.is_some() {
                first_classify = category.parent_id.map(|x| x.to_string());
                second_classify = category.id.map(|x| x.to_string());
            } else {
                first_classify = category.id.map(|x| x.to_string());
            }
        }

        (first_classify, second_classify)
    }

    fn match_update_days(option: &Options, update_time: Option<NaiveDateTime>) -> bool {
        if option.update_days.is_none() || update_time.is_none() {
            return true;
        }

        let other_time = Shanghai.from_local_datetime(&update_time.unwrap()).unwrap()
            + Duration::try_days(*option.update_days.as_ref().unwrap() as i64).unwrap();

        Local::now() <= other_time
    }

    fn match_category(
        option: &Options,
        first_classify: Option<u16>,
        second_classify: Option<u16>,
    ) -> bool {
        if option.category.is_none() {
            return true;
        }

        let category = option.category.as_ref().unwrap();

        if category.parent_id.is_some() {
            category.id == second_classify && category.parent_id == first_classify
        } else {
            category.id == first_classify
        }
    }

    fn match_tags(option: &Options, tag_ids: &[u16]) -> bool {
        if option.tags.is_none() {
            return true;
        }

        option
            .tags
            .as_ref()
            .unwrap()
            .iter()
            .all(|tag| tag_ids.contains(tag.id.as_ref().unwrap()))
    }

    fn match_excluded_tags(option: &Options, tag_ids: &[u16]) -> bool {
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

    fn parse_word_count(word_count: i32) -> Option<u32> {
        // Some novels have negative word counts, e.g. 9326
        if word_count <= 0 {
            None
        } else {
            Some(word_count as u32)
        }
    }

    async fn parse_tags(&self, tag_list: Vec<BookTag>) -> Result<Option<Vec<Tag>>, Error> {
        let sys_tags = self.tags().await?;

        let mut result = Vec::new();
        for tag in tag_list {
            let name = tag.tag_name.trim().to_string();

            // Remove non-system tags
            if sys_tags.iter().any(|item| item.name == name) {
                result.push(Tag {
                    id: Some(tag.tag_id),
                    name,
                });
            } else {
                info!(
                    "This tag is not a system tag and is ignored: {name}({})",
                    tag.tag_id
                );
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

    fn parse_image_url(line: &str) -> Option<Url> {
        let begin = line.find("http").unwrap();
        let end = line.find("[/img]").unwrap();

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

    async fn get_tags(&self, book_type: u16, result: &mut Vec<Tag>) -> Result<(), Error> {
        let response = self
            .get_query(
                "/tag/getAppTagList",
                TagsRequest {
                    page_no: 1,
                    page_size: 99,
                    book_type,
                },
            )
            .await?
            .json::<TagsResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;

        for tag in response.data.list.unwrap() {
            result.push(Tag {
                id: Some(tag.tag_id),
                name: tag.tag_name.trim().to_string(),
            });
        }

        Ok(())
    }

    async fn get_categories(
        &self,
        book_type: &'static str,
        result: &mut Vec<Category>,
    ) -> Result<(), Error> {
        let response = self
            .get_query(
                "/classify/getBookClassifyListByParams",
                CategoryRequest {
                    page_no: 1,
                    page_size: 99,
                    book_type,
                },
            )
            .await?
            .json::<CategoryResponse>()
            .await?;
        utils::check_response_success(response.code, response.msg)?;

        for category in response.data.classify_list.unwrap() {
            let basic_id = category.classify_id;
            let basic_name = category.classify_name.trim().to_string();

            for child_category in category.child_list {
                result.push(Category {
                    id: Some(child_category.classify_id),
                    parent_id: Some(basic_id),
                    name: format!("{basic_name}-{}", child_category.classify_name.trim()),
                });
            }

            result.push(Category {
                id: Some(basic_id),
                parent_id: None,
                name: basic_name,
            });
        }

        Ok(())
    }
}
