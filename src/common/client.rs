use std::{
    ops::{Range, RangeFrom, RangeTo},
    path::PathBuf,
};

use chrono::NaiveDateTime;
use image::DynamicImage;
use url::Url;

use crate::Error;

/// Logged-in user information
#[must_use]
#[derive(Debug)]
pub struct UserInfo {
    /// User's nickname
    pub nickname: String,
    /// User's avatar
    pub avatar: Option<Url>,
}

/// Novel information
#[must_use]
#[derive(Debug, Default)]
pub struct NovelInfo {
    /// Novel id
    pub id: u32,
    /// Novel name
    pub name: String,
    /// Author name
    pub author_name: String,
    /// Url of the novel cover
    pub cover_url: Option<Url>,
    /// Novel introduction
    pub introduction: Option<Vec<String>>,
    /// Novel word count
    pub word_count: Option<u32>,
    /// Is the novel a VIP
    pub is_vip: Option<bool>,
    /// Is the novel finished
    pub is_finished: Option<bool>,
    /// Novel creation time
    pub create_time: Option<NaiveDateTime>,
    /// Novel last update time
    pub update_time: Option<NaiveDateTime>,
    /// Novel category
    pub category: Option<Category>,
    /// Novel tags
    pub tags: Option<Vec<Tag>>,
}

impl PartialEq for NovelInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Novel category
#[must_use]
#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    /// Category id
    pub id: Option<u16>,
    /// Parent category id
    pub parent_id: Option<u16>,
    /// Category name
    pub name: String,
}

impl ToString for Category {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

/// Novel tag
#[must_use]
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    /// Tag id
    pub id: Option<u16>,
    /// Tag name
    pub name: String,
}

impl ToString for Tag {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

/// Volume information
pub type VolumeInfos = Vec<VolumeInfo>;

/// Volume information
#[must_use]
#[derive(Debug)]
pub struct VolumeInfo {
    /// Volume title
    pub title: String,
    /// Chapter information
    pub chapter_infos: Vec<ChapterInfo>,
}

/// Chapter information
#[must_use]
#[derive(Debug, Default)]
pub struct ChapterInfo {
    /// Novel id
    pub novel_id: Option<u32>,
    /// Chapter id
    pub id: u32,
    /// Chapter title
    pub title: String,
    /// Whether this chapter can only be read by VIP users
    pub is_vip: Option<bool>,
    /// Chapter price
    pub price: Option<u16>,
    /// Is the chapter accessible
    pub payment_required: Option<bool>,
    /// Is the chapter valid
    pub is_valid: Option<bool>,
    /// Word count
    pub word_count: Option<u32>,
    /// Chapter creation time
    pub create_time: Option<NaiveDateTime>,
    /// Chapter last update time
    pub update_time: Option<NaiveDateTime>,
}

impl ChapterInfo {
    /// Is this chapter available
    pub fn payment_required(&self) -> bool {
        !self.payment_required.as_ref().is_some_and(|x| !x)
    }

    /// Is this chapter valid
    pub fn is_valid(&self) -> bool {
        !self.is_valid.as_ref().is_some_and(|x| !x)
    }

    /// Is this chapter available for download
    pub fn can_download(&self) -> bool {
        !self.payment_required() && self.is_valid()
    }
}

/// Content information
pub type ContentInfos = Vec<ContentInfo>;

/// Content information
#[must_use]
#[derive(Debug)]
pub enum ContentInfo {
    /// Text content
    Text(String),
    /// Image content
    Image(Url),
}

/// Options used by the search
#[derive(Debug, Default)]
pub struct Options {
    /// Keyword
    pub keyword: Option<String>,
    /// Is it finished
    pub is_finished: Option<bool>,
    /// Whether this chapter can only be read by VIP users
    pub is_vip: Option<bool>,
    /// Category
    pub category: Option<Category>,
    /// Included tags
    pub tags: Option<Vec<Tag>>,
    /// Excluded tags
    pub excluded_tags: Option<Vec<Tag>>,
    /// The number of days since the last update
    pub update_days: Option<u8>,
    /// Word count
    pub word_count: Option<WordCountRange>,
}

/// Word count range
#[derive(Debug)]
pub enum WordCountRange {
    /// Set minimum and maximum word count
    Range(Range<u32>),
    /// Set minimum word count
    RangeFrom(RangeFrom<u32>),
    /// Set maximum word count
    RangeTo(RangeTo<u32>),
}

/// Traits that abstract client behavior
#[trait_variant::make(Send)]
pub trait Client {
    /// set proxy
    fn proxy(&mut self, proxy: Url);

    /// Do not use proxy (environment variables used to set proxy are ignored)
    fn no_proxy(&mut self);

    /// Set the certificate path for use with packet capture tools
    fn cert(&mut self, cert_path: PathBuf);

    /// Stop the client, save the data
    async fn shutdown(&self) -> Result<(), Error>;

    /// Add cookie
    async fn add_cookie(&self, cookie_str: &str, url: &Url) -> Result<(), Error>;

    /// Login in
    async fn log_in(&self, username: String, password: Option<String>) -> Result<(), Error>;

    /// Check if you are logged in
    async fn logged_in(&self) -> Result<bool, Error>;

    /// Get the information of the logged-in user
    async fn user_info(&self) -> Result<UserInfo, Error>;

    /// Get user's existing money
    async fn money(&self) -> Result<u32, Error>;

    /// Sign
    async fn sign(&self) -> Result<(), Error>;

    /// Get the favorite novel of the logged-in user and return the novel id
    async fn bookshelf_infos(&self) -> Result<Vec<u32>, Error>;

    /// Get Novel Information
    async fn novel_info(&self, id: u32) -> Result<Option<NovelInfo>, Error>;

    /// Get volume Information
    async fn volume_infos(&self, id: u32) -> Result<Option<VolumeInfos>, Error>;

    /// Get content Information
    async fn content_infos(&self, info: &ChapterInfo) -> Result<ContentInfos, Error>;

    /// Buy chapter
    async fn buy_chapter(&self, info: &ChapterInfo) -> Result<(), Error>;

    /// Download image
    async fn image(&self, url: &Url) -> Result<DynamicImage, Error>;

    /// Get all categories
    async fn categories(&self) -> Result<&Vec<Category>, Error>;

    /// Get all tags
    async fn tags(&self) -> Result<&Vec<Tag>, Error>;

    /// Search all matching novels
    async fn search_infos(
        &self,
        option: &Options,
        page: u16,
        size: u16,
    ) -> Result<Option<Vec<u32>>, Error>;
}
