use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

use crate::{CiweimaoClient, Error};

pub(crate) fn check_response(code: String, tip: Option<String>) -> Result<(), Error> {
    if code != CiweimaoClient::OK {
        Err(Error::NovelApi(format!(
            "ciweimao request failed, code: `{code}`, msg: `{}`",
            tip.unwrap().trim()
        )))
    } else {
        Ok(())
    }
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct UserInfoRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
    pub account: String,
    pub login_token: String,
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
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct NovelInfoRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
    pub account: String,
    pub login_token: String,
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
    pub book_info: NovelInfoBookInfo,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct NovelInfoBookInfo {
    pub book_name: String,
    pub author_name: String,
    pub cover: String,
    pub description: String,
    pub total_word_count: String,
    pub up_status: String,
    pub newtime: String,
    pub uptime: String,
    pub category_index: String,
    pub tag: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct VolumesRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
    pub account: String,
    pub login_token: String,
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
    pub chapter_list: Vec<VolumesVolumeInfo>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct VolumesVolumeInfo {
    pub division_name: String,
    pub chapter_list: Vec<VolumesChapterInfo>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct VolumesChapterInfo {
    pub chapter_id: String,
    pub chapter_title: String,
    pub word_count: String,
    pub mtime: String,
    pub is_valid: String,
    pub auth_access: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct ChapsRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
    pub account: String,
    pub login_token: String,
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
#[derive(Serialize)]
pub(crate) struct SearchRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
    pub account: String,
    pub login_token: String,
    pub key: String,
    pub count: u16,
    pub page: u16,
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
    pub book_list: Vec<SearchNovelInfo>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct SearchNovelInfo {
    pub book_id: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct FavoritesRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
    pub account: String,
    pub login_token: String,
    pub shelf_id: u32,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct FavoritesResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<FavoritesData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct FavoritesData {
    pub book_list: Vec<FavoritesInfo>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct FavoritesInfo {
    pub book_info: FavoritesNovelInfo,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct FavoritesNovelInfo {
    pub book_id: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct CategoryRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
    pub account: String,
    pub login_token: String,
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
    pub category_list: Vec<CategoryCategory>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct CategoryCategory {
    pub category_detail: Vec<CategoryDetail>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct CategoryDetail {
    pub category_index: String,
    pub category_name: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct TagRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
    pub account: String,
    pub login_token: String,
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
    pub official_tag_list: Vec<TagTag>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct TagTag {
    pub tag_name: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct NovelsRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
    pub account: String,
    pub login_token: String,
    pub count: u16,
    pub page: u16,
    pub category_index: u16,
    pub order: &'static str,
    pub tags: String,
    pub is_paid: Option<u8>,
    pub up_status: Option<u8>,
    pub filter_uptime: Option<u8>,
    pub filter_word: Option<u8>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct NovelsResponse {
    pub code: String,
    pub tip: Option<String>,
    pub data: Option<NovelsData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct NovelsData {
    pub book_list: Vec<NovelsInfo>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct NovelsInfo {
    pub book_id: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct UseGeetestRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
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
    pub t: u64,
    pub user_id: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct GeetestInfoResponse {
    pub success: u8,
    pub gt: String,
    pub challenge: String,
    pub new_captcha: bool,
}

#[must_use]
#[derive(Serialize, ZeroizeOnDrop)]
pub(crate) struct SendVerifyCodeRequest {
    pub account: String,
    #[zeroize(skip)]
    pub app_version: &'static str,
    #[zeroize(skip)]
    pub device_token: &'static str,
    pub hashvalue: String,
    pub login_name: String,
    pub timestamp: String,
    pub verify_type: String,
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
    #[zeroize(skip)]
    pub app_version: &'static str,
    #[zeroize(skip)]
    pub device_token: &'static str,
    pub login_name: String,
    pub passwd: String,
}

#[must_use]
#[derive(Serialize, ZeroizeOnDrop)]
pub(crate) struct LoginCaptchaRequest {
    #[zeroize(skip)]
    pub app_version: &'static str,
    #[zeroize(skip)]
    pub device_token: &'static str,
    pub login_name: String,
    pub passwd: String,
    pub geetest_seccode: String,
    pub geetest_validate: String,
    pub geetest_challenge: String,
}

#[must_use]
#[derive(Serialize, ZeroizeOnDrop)]
pub(crate) struct LoginSMSRequest {
    #[zeroize(skip)]
    pub app_version: &'static str,
    #[zeroize(skip)]
    pub device_token: &'static str,
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
    pub reader_info: LoginReaderInfo,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct LoginReaderInfo {
    pub account: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct ChapterCmdRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
    pub account: String,
    pub login_token: String,
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

#[must_use]
#[derive(Serialize)]
pub(crate) struct ShelfListRequest {
    pub app_version: &'static str,
    pub device_token: &'static str,
    pub account: String,
    pub login_token: String,
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
    pub shelf_list: Vec<ShelfList>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ShelfList {
    pub shelf_id: String,
}
