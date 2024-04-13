use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[must_use]
#[derive(Serialize)]
pub(crate) struct EmptyRequest {}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct GenericResponse {
    pub code: String,
    pub msg: String,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PhoneCodeRequest {
    pub phone: String,
    pub sms_type: &'static str,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LoginRequest {
    pub phone: String,
    pub phone_code: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct LoginResponse {
    pub code: String,
    pub msg: String,
    pub data: LoginData,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LoginData {
    pub user_info: Option<LoginUserInfo>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct LoginUserInfo {
    pub token: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct UserInfoResponse {
    pub code: String,
    pub msg: String,
    pub data: UserInfoData,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserInfoData {
    pub cm_user: Option<UserInfoDataCmUser>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserInfoDataCmUser {
    pub nick_name: String,
    pub img_url: Url,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct MoneyResponse {
    pub code: String,
    pub msg: String,
    pub data: MoneyData,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MoneyData {
    pub account_info: Option<MoneyAccountInfo>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MoneyAccountInfo {
    // 书币
    pub currency_balance: u32,
    //  代币
    pub coupon_balance: u32,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookSelfRequest {
    pub page_no: u16,
    pub page_size: u16,
    pub rank_type: u8,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookSelfResponse {
    pub code: String,
    pub msg: String,
    pub data: BookSelfData,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookSelfData {
    pub book_rack_list: Option<Vec<BookRack>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookRack {
    pub book_id: u32,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookDetailRequest {
    pub book_id: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookDetailResponse {
    pub code: String,
    pub msg: String,
    pub data: BookDetailData,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookDetailData {
    pub book: Option<BookDetail>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookDetail {
    pub book_id: u32,
    pub book_name: Option<String>,
    pub img_url: Option<Url>,
    pub author_name: Option<String>,
    pub word_count: i32,
    pub first_classify: Option<u16>,
    pub first_classify_name: Option<String>,
    pub second_classify: Option<u16>,
    pub second_classify_name: Option<String>,
    pub end_state: Option<String>,
    pub is_vip: Option<String>,
    #[serde(with = "crate::common::date_format_option")]
    pub latest_update_time: Option<NaiveDateTime>,
    pub notes: Option<String>,
    pub tag_list: Option<Vec<BookTag>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookTag {
    pub tag_id: u16,
    pub tag_name: String,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VolumeInfosRequest {
    pub sort_type: &'static str,
    pub page_no: &'static str,
    pub page_size: &'static str,
    pub book_id: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ChapterListResponse {
    pub code: String,
    pub msg: String,
    pub data: ChapterListData,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChapterListData {
    pub book_chapter: Option<BookChapter>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookChapter {
    pub book_id: u32,
    pub chapter_list: Option<Vec<Chapter>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Chapter {
    pub chapter_id: u32,
    pub chapter_name: String,
    pub is_buy: String,
    pub is_fee: String,
    pub price: String,
    #[serde(with = "crate::common::date_format")]
    pub publish_time: NaiveDateTime,
    // 一些小说个别章节的 title 为空
    pub title: Option<String>,
    pub volume_id: u32,
    pub word_count: u32,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentRequest {
    pub book_id: String,
    pub chapter_id: String,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ContentResponse {
    pub code: String,
    pub msg: String,
    pub data: ContentData,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct ContentData {
    pub chapter: Option<ContentChapter>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentChapter {
    pub content: String,
    pub img_list: Option<Vec<ContentImage>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentImage {
    pub img_url: Url,
    pub paragraph_index: usize,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuyChapterRequest {
    pub view_type: &'static str,
    pub consume_type: &'static str,
    pub book_id: String,
    pub product_id: String,
    pub buy_count: &'static str,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CategoryRequest {
    pub page_no: u16,
    pub page_size: u16,
    pub book_type: &'static str,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct CategoryResponse {
    pub code: String,
    pub msg: String,
    pub data: CategoryData,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CategoryData {
    pub classify_list: Option<Vec<Classify>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Classify {
    pub classify_id: u16,
    pub classify_name: String,
    pub child_list: Vec<ChildClassify>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChildClassify {
    pub classify_id: u16,
    pub classify_name: String,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TagsRequest {
    pub page_no: u16,
    pub page_size: u16,
    pub book_type: u16,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct TagsResponse {
    pub code: String,
    pub msg: String,
    pub data: TagsData,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct TagsData {
    pub list: Option<Vec<Tag>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Tag {
    pub tag_id: u16,
    pub tag_name: String,
}

#[must_use]
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchBookListRequest {
    pub page_no: u16,
    pub page_size: u16,
    pub rank_type: &'static str,
    pub keyword: String,
    pub is_fee: Option<String>,
    pub end_state: Option<String>,
    pub classify_ids: Option<String>,
    pub start_word: Option<String>,
    pub end_word: Option<String>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct SearchBookListResponse {
    pub code: String,
    pub msg: String,
    pub data: SearchData,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchData {
    pub es_book_list: Option<Vec<SearchBook>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchBook {
    pub book_id: u32,
    pub first_classify: Option<u16>,
    pub second_classify: Option<u16>,
    pub tag_name: Option<String>,
    #[serde(with = "crate::common::date_format_option")]
    pub latest_update_time: Option<NaiveDateTime>,
}

#[must_use]
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookListRequest {
    pub page_no: u16,
    pub page_size: u16,
    pub rank_type: &'static str,
    pub first_classify: Option<String>,
    pub second_classify: Option<String>,
    pub start_word: Option<String>,
    pub end_word: Option<String>,
    pub end_state: Option<String>,
    pub is_fee: Option<String>,
}

#[must_use]
#[derive(Deserialize)]
pub(crate) struct BookListResponse {
    pub code: String,
    pub msg: String,
    pub data: BookListData,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookListData {
    pub book_list: Option<Vec<BookListBook>>,
}

#[must_use]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookListBook {
    pub book_id: u32,
    #[serde(with = "crate::common::date_format_option")]
    pub latest_update_time: Option<NaiveDateTime>,
    pub tag_list: Option<Vec<BookTag>>,
}
