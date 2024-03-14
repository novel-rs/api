use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;
use zeroize::ZeroizeOnDrop;

#[must_use]
#[derive(Serialize)]
pub(crate) struct EmptyRequest {}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct GenericResponse {
    pub code: String,
    pub tip: Option<String>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct UserInfoResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<UserInfoData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct UserInfoData {
    pub reader_info: UserInfoReaderInfo,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct UserInfoReaderInfo {
    pub reader_name: String,
    // 当头像不存在时，是空字符串
    #[serde(with = "crate::ciweimao::parse_url")]
    pub avatar_url: Option<Url>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct PropInfoResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<PropInfoData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct PropInfoData {
    pub prop_info: PropInfo,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct PropInfo {
    // 猫饼干 + 代币
    pub rest_hlb: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct SignRequest {
    pub task_type: u8,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct BookshelfRequest {
    pub shelf_id: u32,
    pub count: u16,
    pub page: u16,
    pub order: &'static str,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookshelfResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<BookshelfData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookshelfData {
    pub book_list: Vec<BookshelfInfo>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookshelfInfo {
    pub book_info: BookshelfNovelInfo,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookshelfNovelInfo {
    pub book_id: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct NovelInfoRequest {
    pub book_id: u32,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct NovelInfoResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<NovelInfoData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct NovelInfoData {
    pub book_info: BookInfo,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookInfo {
    pub book_name: String,
    pub author_name: String,
    // 有一些小说 cover 为空
    #[serde(with = "crate::ciweimao::parse_url")]
    pub cover: Option<Url>,
    pub description: String,
    pub total_word_count: String,
    #[serde(with = "crate::ciweimao::parse_bool")]
    pub is_paid: bool,
    #[serde(with = "crate::ciweimao::parse_bool")]
    pub up_status: bool,
    // 有一些小说 newtime 为空
    #[serde(with = "crate::common::date_format_option")]
    pub newtime: Option<NaiveDateTime>,
    #[serde(with = "crate::common::date_format")]
    pub uptime: NaiveDateTime,
    pub category_index: String,
    pub tag_list: Vec<NovelInfoTag>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct NovelInfoTag {
    pub tag_name: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct VolumesRequest {
    pub book_id: u32,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct VolumesResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<VolumesData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct VolumesData {
    pub chapter_list: Vec<VolumeInfo>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct VolumeInfo {
    pub division_name: String,
    pub chapter_list: Vec<ChapterInfo>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ChapterInfo {
    pub chapter_id: String,
    pub chapter_title: String,
    pub word_count: String,
    #[serde(with = "crate::common::date_format")]
    pub mtime: NaiveDateTime,
    #[serde(with = "crate::ciweimao::parse_bool")]
    pub is_valid: bool,
    #[serde(with = "crate::ciweimao::parse_bool")]
    pub is_paid: bool,
    #[serde(with = "crate::ciweimao::parse_bool")]
    pub auth_access: bool,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct BuyRequest {
    pub chapter_id: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct ChapsRequest {
    pub chapter_id: String,
    pub chapter_command: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ChapsResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<ChapsData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ChapsData {
    pub chapter_info: ChapsInfo,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ChapsInfo {
    pub txt_content: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct CategoryResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<CategoryData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct CategoryData {
    pub category_list: Vec<Category>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct Category {
    pub category_detail: Vec<CategoryDetail>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct CategoryDetail {
    pub category_index: String,
    pub category_name: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct TagResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<TagData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct TagData {
    pub official_tag_list: Vec<Tag>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct Tag {
    pub tag_name: String,
}

#[must_use]
#[skip_serializing_none]
#[derive(Serialize)]
pub(crate) struct SearchRequest {
    pub count: u16,
    pub page: u16,
    pub order: &'static str,
    pub category_index: u16,
    pub tags: String,
    pub key: Option<String>,
    pub is_paid: Option<u8>,
    pub up_status: Option<u8>,
    pub filter_uptime: Option<u8>,
    pub filter_word: Option<u8>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct SearchResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<SearchData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct SearchData {
    pub book_list: Vec<SearchInfo>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct SearchInfo {
    pub book_id: String,
    pub total_word_count: String,
    #[serde(with = "crate::common::date_format")]
    pub uptime: NaiveDateTime,
    pub tag_list: Vec<NovelInfoTag>,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct UseGeetestRequest {
    pub login_name: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct UseGeetestResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<UseGeetestData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct UseGeetestData {
    pub need_use_geetest: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct GeetestInfoRequest {
    pub t: u128,
    pub user_id: String,
}

#[must_use]
#[derive(Deserialize, Clone)]
pub(crate) struct GeetestInfoResponse {
    pub success: u8,
    pub new_captcha: bool,
    pub gt: String,
    pub challenge: String,
}

#[must_use]
#[derive(Serialize, ZeroizeOnDrop)]
pub(crate) struct SendVerifyCodeRequest {
    pub login_name: String,
    pub timestamp: u128,
    pub verify_type: u8,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct SendVerifyCodeResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<SendVerifyCodeData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct SendVerifyCodeData {
    pub to_code: String,
}

#[must_use]
#[derive(Serialize, ZeroizeOnDrop)]
pub(crate) struct LoginRequest {
    pub login_name: String,
    pub passwd: String,
}

#[must_use]
#[derive(Serialize, ZeroizeOnDrop)]
pub(crate) struct LoginCaptchaRequest {
    pub login_name: String,
    pub passwd: String,
    pub geetest_seccode: String,
    pub geetest_validate: String,
    pub geetest_challenge: String,
}

#[must_use]
#[derive(Serialize, ZeroizeOnDrop)]
pub(crate) struct LoginSMSRequest {
    pub login_name: String,
    pub passwd: String,
    pub to_code: String,
    pub ver_code: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct LoginResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<LoginData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct LoginData {
    pub login_token: String,
    pub reader_info: ReaderInfo,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ReaderInfo {
    pub account: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ShelfListResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<ShelfListData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ShelfListData {
    pub shelf_list: Vec<Shelf>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct Shelf {
    pub shelf_id: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct PriceRequest {
    pub book_id: u32,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct PriceResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<PriceData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct PriceData {
    pub chapter_permission_list: Vec<ChapterPermission>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ChapterPermission {
    pub chapter_id: String,
    pub unit_hlb: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct ChapterCmdRequest {
    pub chapter_id: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ChapterCmdResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<ChapterCmdData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ChapterCmdData {
    pub command: String,
}

pub(crate) mod parse_bool {
    use serde::{Deserialize, Deserializer};

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        assert!(!s.is_empty());

        if s == "1" {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub(crate) mod parse_url {
    use serde::{Deserialize, Deserializer};
    use url::Url;

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match Url::parse(&s) {
            Ok(url) => Ok(Some(url)),
            Err(_) => Ok(None),
        }
    }
}
