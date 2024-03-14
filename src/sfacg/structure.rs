use chrono::NaiveDateTime;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;
use zeroize::ZeroizeOnDrop;

use crate::Error;

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Status {
    pub http_code: u16,
    pub error_code: u16,
    pub msg: Option<String>,
}

impl Status {
    #[must_use]
    pub(crate) fn ok(&self) -> bool {
        self.http_code == StatusCode::OK && self.error_code == 200
    }

    // for buy_chapter
    #[must_use]
    pub(crate) fn created(&self) -> bool {
        self.http_code == StatusCode::CREATED && self.error_code == 200
    }

    #[must_use]
    pub(crate) fn not_found(&self) -> bool {
        self.http_code == StatusCode::NOT_FOUND && self.error_code == 404
    }

    #[must_use]
    pub(crate) fn unauthorized(&self) -> bool {
        self.http_code == StatusCode::UNAUTHORIZED && self.error_code == 502
    }

    #[must_use]
    pub(crate) fn already_signed_in(&self) -> bool {
        self.http_code == StatusCode::BAD_REQUEST && self.error_code == 1050
    }

    #[must_use]
    pub(crate) fn not_available(&self) -> bool {
        self.http_code == StatusCode::EXPECTATION_FAILED && self.error_code == 1116
    }

    pub(crate) fn check(self) -> Result<(), Error> {
        if !(self.ok() || self.created()) {
            return Err(Error::Http {
                code: StatusCode::from_u16(self.http_code)?,
                msg: self.msg.unwrap().trim().to_string(),
            })?;
        }

        Ok(())
    }
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct GenericResponse {
    pub status: Status,
}

#[must_use]
#[derive(Serialize, ZeroizeOnDrop)]
pub(crate) struct LogInRequest {
    pub username: String,
    pub password: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct UserInfoResponse {
    pub status: Status,
    pub data: Option<UserInfoData>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserInfoData {
    pub nick_name: String,
    pub avatar: Url,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct MoneyResponse {
    pub status: Status,
    pub data: Option<MoneyData>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MoneyData {
    // 火券
    pub fire_money_remain: u32,
    // 代券
    pub coupons_remain: u32,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct SignRequest {
    pub sign_date: String,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct BookshelfInfoRequest {
    pub expand: &'static str,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookshelfInfoResponse {
    pub status: Status,
    pub data: Option<Vec<BookshelfInfoData>>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookshelfInfoData {
    pub expand: Option<BookshelfInfoExpand>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookshelfInfoExpand {
    pub novels: Option<Vec<BookshelfInfoNovelInfo>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookshelfInfoNovelInfo {
    pub novel_id: u32,
}

#[must_use]
#[derive(Serialize)]
pub(crate) struct NovelInfoRequest {
    pub expand: &'static str,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct NovelInfoResponse {
    pub status: Status,
    pub data: Option<NovelInfoData>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NovelInfoData {
    pub novel_name: String,
    pub novel_cover: Url,
    pub author_name: String,
    pub char_count: i32,
    pub type_id: u16,
    pub sign_status: String,
    pub is_finish: bool,
    pub add_time: NaiveDateTime,
    pub last_update_time: NaiveDateTime,
    pub expand: NovelInfoExpand,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NovelInfoExpand {
    pub type_name: String,
    pub intro: String,
    pub sys_tags: Vec<NovelInfoSysTag>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NovelInfoSysTag {
    pub sys_tag_id: u16,
    pub tag_name: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct VolumeInfosResponse {
    pub status: Status,
    pub data: Option<VolumeInfosData>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VolumeInfosData {
    pub volume_list: Vec<VolumeInfosVolume>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VolumeInfosVolume {
    pub title: String,
    pub chapter_list: Vec<VolumeInfosChapter>,
}

#[must_use]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VolumeInfosChapter {
    pub novel_id: u32,
    pub chap_id: u32,
    pub title: String,
    pub char_count: u32,
    pub is_vip: bool,
    pub need_fire_money: u16,
    #[serde(rename = "AddTime")]
    pub add_time: NaiveDateTime,
    pub update_time: Option<NaiveDateTime>,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentInfosRequest {
    pub expand: &'static str,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ContentInfosResponse {
    pub status: Status,
    pub data: Option<ContentInfosData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ContentInfosData {
    pub expand: ContentInfosExpand,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentInfosExpand {
    pub content: String,
    pub is_content_encrypted: bool,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuyChapterRequest {
    pub order_all: bool,
    pub auto_order: bool,
    pub chap_ids: Vec<u32>,
    pub order_type: &'static str,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct CategoryResponse {
    pub status: Status,
    pub data: Option<Vec<CategoryData>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CategoryData {
    pub type_id: u16,
    pub type_name: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct TagResponse {
    pub status: Status,
    pub data: Option<Vec<Tag>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Tag {
    pub sys_tag_id: u16,
    pub tag_name: String,
}

#[must_use]
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchRequest {
    pub q: String,
    pub is_finish: i8,
    pub update_days: i8,
    pub systagids: Option<String>,
    pub page: u16,
    pub size: u16,
    pub sort: &'static str,
    pub expand: &'static str,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct SearchResponse {
    pub status: Status,
    pub data: Option<SearchData>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct SearchData {
    pub novels: Vec<SearchNovelInfo>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchNovelInfo {
    pub novel_id: u32,
    pub sign_status: String,
    pub char_count: i32,
    pub type_id: u16,
    pub expand: SearchExpand,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchExpand {
    pub sys_tags: Vec<NovelInfoSysTag>,
}

#[must_use]
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NovelsRequest {
    pub charcountbegin: u32,
    pub charcountend: u32,
    pub updatedays: i8,
    pub isfinish: &'static str,
    pub isfree: &'static str,
    pub systagids: Option<String>,
    pub notexcludesystagids: Option<String>,
    pub page: u16,
    pub size: u16,
    pub sort: &'static str,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct NovelsResponse {
    pub status: Status,
    pub data: Option<Vec<NovelsData>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NovelsData {
    pub novel_id: u32,
}
