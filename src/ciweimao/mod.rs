mod server;
mod structure;
mod utils;

use std::{
    io::Cursor,
    path::PathBuf,
    sync::RwLock,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{Duration, Local, NaiveDateTime, TimeZone};
use chrono_tz::Asia::Shanghai;
use hashbrown::HashMap;
use image::{io::Reader, DynamicImage};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::OnceCell;
use tracing::{error, info};
use url::Url;

use crate::{
    Category, ChapterInfo, Client, ContentInfo, ContentInfos, Error, FindImageResult,
    FindTextResult, HTTPClient, NovelDB, NovelInfo, Options, Tag, UserInfo, VolumeInfo,
    VolumeInfos, WordCountRange,
};
use structure::*;

#[must_use]
#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    account: String,
    login_token: String,
}

/// Ciweimao client, use it to access Apis
#[must_use]
pub struct CiweimaoClient {
    proxy: Option<Url>,
    no_proxy: bool,
    cert_path: Option<PathBuf>,

    client: OnceCell<HTTPClient>,
    client_rss: OnceCell<HTTPClient>,

    db: OnceCell<NovelDB>,

    config: RwLock<Option<Config>>,
}

impl Client for CiweimaoClient {
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
        assert!(password.is_some());

        let password = password.unwrap();

        let config = match self.verify_type(&username).await? {
            VerifyType::None => {
                info!("No verification required");
                self.no_verification_login(username, password).await?
            }
            VerifyType::Geetest => {
                info!("Verify with Geetest");
                self.geetest_login(username, password).await?
            }
            VerifyType::VerifyCode => {
                info!("Verify with SMS verification code");
                self.sms_login(username, password).await?
            }
        };

        self.save_token(config);

        Ok(())
    }

    async fn logged_in(&self) -> Result<bool, Error> {
        if !self.has_token() {
            return Ok(false);
        }

        let response: GenericResponse = self.post("/reader/get_my_info", EmptyRequest {}).await?;

        if response.code == CiweimaoClient::LOGIN_EXPIRED {
            Ok(false)
        } else {
            utils::check_response_success(response.code, response.tip)?;
            Ok(true)
        }
    }

    async fn user_info(&self) -> Result<UserInfo, Error> {
        let response: UserInfoResponse = self.post("/reader/get_my_info", EmptyRequest {}).await?;
        utils::check_response_success(response.code, response.tip)?;
        let reader_info = response.data.unwrap().reader_info;

        let user_info = UserInfo {
            nickname: reader_info.reader_name.trim().to_string(),
            avatar: reader_info.avatar_url,
        };

        Ok(user_info)
    }

    async fn money(&self) -> Result<u32, Error> {
        let response: PropInfoResponse =
            self.post("/reader/get_prop_info", EmptyRequest {}).await?;
        utils::check_response_success(response.code, response.tip)?;
        let prop_info = response.data.unwrap().prop_info;

        Ok(prop_info.rest_hlb.parse()?)
    }

    async fn sign_in(&self) -> Result<(), Error> {
        let response: GenericResponse = self
            .post(
                "/reader/get_task_bonus_with_sign_recommend",
                SignRequest {
                    // always 1, from `/task/get_all_task_list`
                    task_type: 1,
                },
            )
            .await?;
        if utils::check_already_signed_in(&response.code) {
            info!("{}", CiweimaoClient::ALREADY_SIGNED_IN);
        } else {
            utils::check_response_success(response.code, response.tip)?;
        }

        Ok(())
    }

    async fn bookshelf_infos(&self) -> Result<Vec<u32>, Error> {
        let shelf_ids = self.shelf_list().await?;
        let mut result = Vec::new();

        for shelf_id in shelf_ids {
            let response: BookshelfResponse = self
                .post(
                    "/bookshelf/get_shelf_book_list_new",
                    BookshelfRequest {
                        shelf_id,
                        count: 9999,
                        page: 0,
                        order: "last_read_time",
                    },
                )
                .await?;
            utils::check_response_success(response.code, response.tip)?;

            for novel_info in response.data.unwrap().book_list {
                result.push(novel_info.book_info.book_id.parse()?);
            }
        }

        Ok(result)
    }

    async fn novel_info(&self, id: u32) -> Result<Option<NovelInfo>, Error> {
        assert!(id > 0);

        let response: NovelInfoResponse = self
            .post("/book/get_info_by_id", NovelInfoRequest { book_id: id })
            .await?;
        if response.code == CiweimaoClient::NOT_FOUND {
            return Ok(None);
        }
        utils::check_response_success(response.code, response.tip)?;

        let data = response.data.unwrap().book_info;
        let novel_info = NovelInfo {
            id,
            name: data.book_name.trim().to_string(),
            author_name: data.author_name.trim().to_string(),
            cover_url: data.cover,
            introduction: CiweimaoClient::parse_introduction(data.description),
            word_count: Some(data.total_word_count.parse()?),
            is_vip: Some(data.is_paid),
            is_finished: Some(data.up_status),
            create_time: data.newtime,
            update_time: Some(data.uptime),
            category: self.parse_category(data.category_index).await?,
            tags: self.parse_tags(data.tag_list).await?,
        };

        Ok(Some(novel_info))
    }

    async fn volume_infos(&self, id: u32) -> Result<Option<VolumeInfos>, Error> {
        let response: VolumesResponse = self
            .post(
                "/chapter/get_updated_chapter_by_division_new",
                VolumesRequest { book_id: id },
            )
            .await?;
        utils::check_response_success(response.code, response.tip)?;
        let chapter_list = response.data.unwrap().chapter_list;

        let chapter_prices = self.chapter_prices(id).await?;

        let mut volume_infos = VolumeInfos::new();
        for item in chapter_list {
            let mut volume_info = VolumeInfo {
                title: item.division_name.trim().to_string(),
                chapter_infos: Vec::new(),
            };

            for chapter in item.chapter_list {
                let chapter_id: u32 = chapter.chapter_id.parse()?;
                let price = chapter_prices.get(&chapter_id).copied();
                let mut is_valid = true;

                // e.g. 该章节未审核通过
                if price.is_none() {
                    info!("Price not found: {chapter_id}");
                    is_valid = false;
                }

                let chapter_info = ChapterInfo {
                    novel_id: Some(id),
                    id: chapter_id,
                    title: chapter.chapter_title.trim().to_string(),
                    word_count: Some(chapter.word_count.parse()?),
                    create_time: Some(chapter.mtime),
                    update_time: None,
                    is_vip: Some(chapter.is_paid),
                    price,
                    payment_required: Some(!chapter.auth_access),
                    is_valid: Some(chapter.is_valid && is_valid),
                };

                volume_info.chapter_infos.push(chapter_info);
            }

            volume_infos.push(volume_info);
        }

        Ok(Some(volume_infos))
    }

    async fn content_infos(&self, info: &ChapterInfo) -> Result<ContentInfos, Error> {
        let content;

        match self.db().await?.find_text(info).await? {
            FindTextResult::Ok(str) => {
                content = str;
            }
            other => {
                let cmd = self.chapter_cmd(info.id).await?;
                let key = crate::sha256(cmd.as_bytes());

                let response: ChapsResponse = self
                    .post(
                        "/chapter/get_cpt_ifm",
                        ChapsRequest {
                            chapter_id: info.id.to_string(),
                            chapter_command: cmd,
                        },
                    )
                    .await?;
                utils::check_response_success(response.code, response.tip)?;

                content = simdutf8::basic::from_utf8(&crate::aes_256_cbc_no_iv_base64_decrypt(
                    key,
                    response.data.unwrap().chapter_info.txt_content,
                )?)?
                .to_string();

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
            if line.starts_with("<img") {
                if let Some(url) = CiweimaoClient::parse_image_url(line) {
                    content_infos.push(ContentInfo::Image(url));
                }
            } else {
                content_infos.push(ContentInfo::Text(line.to_string()));
            }
        }

        Ok(content_infos)
    }

    async fn buy_chapter(&self, info: &ChapterInfo) -> Result<(), Error> {
        let response: GenericResponse = self
            .post(
                "/chapter/buy",
                BuyRequest {
                    chapter_id: info.id.to_string(),
                },
            )
            .await?;
        utils::check_response_success(response.code, response.tip)?;

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
                let response: CategoryResponse =
                    self.post("/meta/get_meta_data", EmptyRequest {}).await?;
                utils::check_response_success(response.code, response.tip)?;

                let mut result = Vec::new();
                for category in response.data.unwrap().category_list {
                    for category_detail in category.category_detail {
                        result.push(Category {
                            id: Some(category_detail.category_index.parse()?),
                            parent_id: None,
                            name: category_detail.category_name.trim().to_string(),
                        });
                    }
                }

                result.sort_unstable_by_key(|x| x.id.unwrap());

                Ok(result)
            })
            .await
    }

    async fn tags(&self) -> Result<&Vec<Tag>, Error> {
        static TAGS: OnceCell<Vec<Tag>> = OnceCell::const_new();

        TAGS.get_or_try_init(|| async {
            let response: TagResponse = self
                .post("/book/get_official_tag_list", EmptyRequest {})
                .await?;
            utils::check_response_success(response.code, response.tip)?;

            let mut result = Vec::new();
            for tag in response.data.unwrap().official_tag_list {
                result.push(Tag {
                    id: None,
                    name: tag.tag_name.trim().to_string(),
                });
            }

            result.push(Tag {
                id: None,
                name: String::from("橘子"),
            });
            result.push(Tag {
                id: None,
                name: String::from("变身"),
            });
            result.push(Tag {
                id: None,
                name: String::from("性转"),
            });
            result.push(Tag {
                id: None,
                name: String::from("纯百"),
            });
            result.push(Tag {
                id: None,
                name: String::from("变百"),
            });

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
        let mut category_index = 0;
        if option.category.is_some() {
            category_index = option.category.as_ref().unwrap().id.unwrap();
        }

        let mut tags = Vec::new();
        if option.tags.is_some() {
            for tag in option.tags.as_ref().unwrap() {
                tags.push(json!({
                    "tag": tag.name,
                    "filter": "1"
                }));
            }
        }

        let is_paid = option.is_vip.map(|is_vip| if is_vip { 1 } else { 0 });

        let up_status = option
            .is_finished
            .map(|is_finished| if is_finished { 1 } else { 0 });

        let mut filter_word = None;
        if option.word_count.is_some() {
            match option.word_count.as_ref().unwrap() {
                WordCountRange::RangeTo(range_to) => {
                    if range_to.end < 30_0000 {
                        filter_word = Some(1);
                    }
                }
                WordCountRange::Range(range) => {
                    if range.start >= 30_0000 && range.end < 50_0000 {
                        filter_word = Some(2);
                    } else if range.start >= 50_0000 && range.end < 100_0000 {
                        filter_word = Some(3);
                    } else if range.start >= 100_0000 && range.end < 200_0000 {
                        filter_word = Some(4);
                    }
                }
                WordCountRange::RangeFrom(range_from) => {
                    if range_from.start >= 200_0000 {
                        filter_word = Some(5);
                    }
                }
            }
        }

        let mut filter_uptime = None;
        if option.update_days.is_some() {
            let update_days = *option.update_days.as_ref().unwrap();

            if update_days <= 3 {
                filter_uptime = Some(1)
            } else if update_days <= 7 {
                filter_uptime = Some(2)
            } else if update_days <= 15 {
                filter_uptime = Some(3)
            } else if update_days <= 30 {
                filter_uptime = Some(4)
            }
        }

        let order = if option.keyword.is_some() {
            // When using keyword search, many irrelevant items will appear in the search results
            // If you use sorting, you will not be able to obtain the target items
            None
        } else {
            // 人气排序
            Some("week_click")
        };

        let response: SearchResponse = self
            .post(
                "/bookcity/get_filter_search_book_list",
                SearchRequest {
                    count: size,
                    page,
                    order,
                    category_index,
                    tags: json!(tags).to_string(),
                    key: option.keyword.clone(),
                    is_paid,
                    up_status,
                    filter_uptime,
                    filter_word,
                },
            )
            .await?;
        utils::check_response_success(response.code, response.tip)?;

        let book_list = response.data.unwrap().book_list;
        if book_list.is_empty() {
            return Ok(None);
        }

        let mut result = Vec::new();
        let sys_tags = self.tags().await?;

        for novel_info in book_list {
            let mut tag_names = Vec::new();
            for tag in novel_info.tag_list {
                if let Some(sys_tag) = sys_tags.iter().find(|x| x.name == tag.tag_name.trim()) {
                    tag_names.push(sys_tag.name.clone());
                }
            }

            if CiweimaoClient::match_update_days(option, novel_info.uptime)
                && CiweimaoClient::match_excluded_tags(option, tag_names)
                && CiweimaoClient::match_word_count(option, novel_info.total_word_count.parse()?)
            {
                result.push(novel_info.book_id.parse()?);
            }
        }

        Ok(Some(result))
    }
}

#[must_use]
enum VerifyType {
    None,
    Geetest,
    VerifyCode,
}

impl CiweimaoClient {
    async fn verify_type<T>(&self, username: T) -> Result<VerifyType, Error>
    where
        T: AsRef<str>,
    {
        let response: UseGeetestResponse = self
            .post(
                "/signup/use_geetest",
                UseGeetestRequest {
                    login_name: username.as_ref().to_string(),
                },
            )
            .await?;
        utils::check_response_success(response.code, response.tip)?;

        let need_use_geetest = response.data.unwrap().need_use_geetest;
        if need_use_geetest == "0" {
            Ok(VerifyType::None)
        } else if need_use_geetest == "1" {
            Ok(VerifyType::Geetest)
        } else if need_use_geetest == "2" {
            Ok(VerifyType::VerifyCode)
        } else {
            unreachable!("The value range of need_use_geetest is 0..=2");
        }
    }

    async fn no_verification_login(
        &self,
        username: String,
        password: String,
    ) -> Result<Config, Error> {
        let response: LoginResponse = self
            .post(
                "/signup/login",
                LoginRequest {
                    login_name: username,
                    passwd: password,
                },
            )
            .await?;
        utils::check_response_success(response.code, response.tip)?;

        let data = response.data.unwrap();

        Ok(Config {
            account: data.reader_info.account,
            login_token: data.login_token,
        })
    }

    async fn geetest_login(&self, username: String, password: String) -> Result<Config, Error> {
        let info = self.geetest_info(&username).await?;
        let geetest_challenge = info.challenge.clone();

        let validate = server::run_geetest(info).await?;

        let response: LoginResponse = self
            .post(
                "/signup/login",
                LoginCaptchaRequest {
                    login_name: username,
                    passwd: password,
                    geetest_seccode: validate.clone() + "|jordan",
                    geetest_validate: validate,
                    geetest_challenge,
                },
            )
            .await?;
        utils::check_response_success(response.code, response.tip)?;

        let data = response.data.unwrap();

        Ok(Config {
            account: data.reader_info.account,
            login_token: data.login_token,
        })
    }

    async fn geetest_info<T>(&self, username: T) -> Result<GeetestInfoResponse, Error>
    where
        T: AsRef<str>,
    {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        let response = self
            .get_query(
                "/signup/geetest_first_register",
                GeetestInfoRequest {
                    t: timestamp,
                    user_id: username.as_ref().to_string(),
                },
            )
            .await?
            .json::<GeetestInfoResponse>()
            .await?;

        if response.success != 1 {
            return Err(Error::NovelApi(String::from(
                "`/signup/geetest_first_register` failed",
            )));
        }

        Ok(response)
    }

    async fn sms_login(&self, username: String, password: String) -> Result<Config, Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let response: SendVerifyCodeResponse = self
            .post(
                "/signup/send_verify_code",
                SendVerifyCodeRequest {
                    login_name: username.clone(),
                    timestamp,
                    // always 5
                    verify_type: 5,
                },
            )
            .await?;
        utils::check_response_success(response.code, response.tip)?;

        let response: LoginResponse = self
            .post(
                "/signup/login",
                LoginSMSRequest {
                    login_name: username,
                    passwd: password,
                    to_code: response.data.unwrap().to_code,
                    ver_code: crate::input("Please enter SMS verification code")?,
                },
            )
            .await?;
        utils::check_response_success(response.code, response.tip)?;

        let data = response.data.unwrap();

        Ok(Config {
            account: data.reader_info.account,
            login_token: data.login_token,
        })
    }

    async fn shelf_list(&self) -> Result<Vec<u32>, Error> {
        let response: ShelfListResponse = self
            .post("/bookshelf/get_shelf_list", EmptyRequest {})
            .await?;
        utils::check_response_success(response.code, response.tip)?;

        let mut result = Vec::new();
        for shelf in response.data.unwrap().shelf_list {
            result.push(shelf.shelf_id.parse()?);
        }

        Ok(result)
    }

    async fn chapter_prices(&self, novel_id: u32) -> Result<HashMap<u32, u16>, Error> {
        let response: PriceResponse = self
            .post(
                "/chapter/get_chapter_permission_list",
                PriceRequest { book_id: novel_id },
            )
            .await?;
        utils::check_response_success(response.code, response.tip)?;
        let chapter_permission_list = response.data.unwrap().chapter_permission_list;

        let mut result = HashMap::new();

        for item in chapter_permission_list {
            result.insert(item.chapter_id.parse()?, item.unit_hlb.parse()?);
        }

        Ok(result)
    }

    async fn chapter_cmd(&self, id: u32) -> Result<String, Error> {
        let response: ChapterCmdResponse = self
            .post(
                "/chapter/get_chapter_cmd",
                ChapterCmdRequest {
                    chapter_id: id.to_string(),
                },
            )
            .await?;
        utils::check_response_success(response.code, response.tip)?;

        Ok(response.data.unwrap().command)
    }

    fn match_update_days(option: &Options, update_time: NaiveDateTime) -> bool {
        if option.update_days.is_none() {
            return true;
        }

        let other_time = Shanghai.from_local_datetime(&update_time).unwrap()
            + Duration::try_days(*option.update_days.as_ref().unwrap() as i64).unwrap();

        Local::now() <= other_time
    }

    fn match_word_count(option: &Options, word_count: u32) -> bool {
        if option.word_count.is_none() {
            return true;
        }

        match option.word_count.as_ref().unwrap() {
            WordCountRange::RangeTo(range_to) => word_count <= range_to.end,
            WordCountRange::Range(range) => range.start <= word_count && word_count <= range.end,
            WordCountRange::RangeFrom(range_from) => range_from.start <= word_count,
        }
    }

    fn match_excluded_tags(option: &Options, tag_ids: Vec<String>) -> bool {
        if option.excluded_tags.is_none() {
            return true;
        }

        tag_ids.iter().all(|name| {
            !option
                .excluded_tags
                .as_ref()
                .unwrap()
                .iter()
                .any(|tag| tag.name == *name)
        })
    }

    fn parse_url<T>(str: T) -> Option<Url>
    where
        T: AsRef<str>,
    {
        let str = str.as_ref();
        if str.is_empty() {
            return None;
        }

        match Url::parse(str) {
            Ok(url) => Some(url),
            Err(error) => {
                error!("Url parse failed: {error}, content: {str}");
                None
            }
        }
    }

    async fn parse_tags(&self, tag_list: Vec<NovelInfoTag>) -> Result<Option<Vec<Tag>>, Error> {
        let sys_tags = self.tags().await?;

        let mut result = Vec::new();
        for tag in tag_list {
            let name = tag.tag_name.trim().to_string();

            // Remove non-system tags
            if sys_tags.iter().any(|item| item.name == name) {
                result.push(Tag { id: None, name });
            } else {
                info!("This tag is not a system tag and is ignored: {name}");
            }
        }

        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    async fn parse_category<T>(&self, str: T) -> Result<Option<Category>, Error>
    where
        T: AsRef<str>,
    {
        let str = str.as_ref();
        if str.is_empty() {
            return Ok(None);
        }

        let categories = self.categories().await?;

        match str.parse::<u16>() {
            Ok(index) => match categories.iter().find(|item| item.id == Some(index)) {
                Some(category) => Ok(Some(category.clone())),
                None => {
                    error!("The category index does not exist: {str}");
                    Ok(None)
                }
            },
            Err(error) => {
                error!("category_index parse failed: {error}");
                Ok(None)
            }
        }
    }

    fn parse_introduction<T>(str: T) -> Option<Vec<String>>
    where
        T: AsRef<str>,
    {
        let str = str.as_ref();
        if str.is_empty() {
            return None;
        }

        let introduction = str
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

    fn parse_image_url<T>(str: T) -> Option<Url>
    where
        T: AsRef<str>,
    {
        let str = str.as_ref();
        if str.is_empty() {
            return None;
        }

        let fragment = Html::parse_fragment(str);
        let selector = Selector::parse("img").unwrap();

        let element = fragment.select(&selector).next();
        if element.is_none() {
            error!("No `img` element exists: {str}");
            return None;
        }
        let element = element.unwrap();

        let url = element.value().attr("src");
        if url.is_none() {
            error!("No `src` attribute exists: {str}");
            return None;
        }
        let url = url.unwrap();

        CiweimaoClient::parse_url(url.trim())
    }
}
