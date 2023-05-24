use std::time::{SystemTime, UNIX_EPOCH};

use boring::hash::{self, MessageDigest};
use hex_simd::AsciiCase;
use reqwest::Response;
use serde::Serialize;
use tokio::sync::OnceCell;
use url::Url;
use uuid::Uuid;

use crate::{Error, HTTPClient, NovelDB, SfacgClient};

impl SfacgClient {
    const APP_NAME: &str = "sfacg";

    const HOST: &str = "https://api.sfacg.com";
    const USER_AGENT_PREFIX: &str = "boluobao/4.9.62(iOS;16.5)/appStore/";
    const USER_AGENT_RSS: &str = "SFReader/4.9.62 (iPhone; iOS 16.5; Scale/3.00)";

    const USERNAME: &str = "apiuser";
    const PASSWORD: &str = "3s#1-yt6e*Acv@qer";

    const SALT: &str = "FMLxgOdsfxmN!Dt4";

    /// Create a sfacg client
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {
            proxy: None,
            no_proxy: false,
            cert_path: None,
            client: OnceCell::new(),
            client_rss: OnceCell::new(),
            db: OnceCell::new(),
        })
    }

    #[inline]
    pub(crate) async fn client(&self) -> Result<&HTTPClient, Error> {
        self.client
            .get_or_try_init(|| async {
                let device_token = crate::uid();
                let user_agent = SfacgClient::USER_AGENT_PREFIX.to_string() + device_token;

                HTTPClient::builder(SfacgClient::APP_NAME)
                    .accept("application/vnd.sfacg.api+json;version=1")
                    .accept_language("zh-Hans-CN;q=1")
                    .cookie(true)
                    .user_agent(user_agent)
                    .proxy(self.proxy.clone())
                    .no_proxy(self.no_proxy)
                    .cert(self.cert_path.clone())
                    .build()
                    .await
            })
            .await
    }

    #[inline]
    pub(crate) async fn client_rss(&self) -> Result<&HTTPClient, Error> {
        self.client_rss
            .get_or_try_init(|| async {
                HTTPClient::builder(SfacgClient::APP_NAME)
                    .accept("image/webp,image/*,*/*;q=0.8")
                    .accept_language("zh-CN,zh-Hans;q=0.9")
                    .user_agent(SfacgClient::USER_AGENT_RSS)
                    .proxy(self.proxy.clone())
                    .no_proxy(self.no_proxy)
                    .cert(self.cert_path.clone())
                    .build()
                    .await
            })
            .await
    }

    #[inline]
    pub(crate) async fn db(&self) -> Result<&NovelDB, Error> {
        self.db
            .get_or_try_init(|| async { NovelDB::new(SfacgClient::APP_NAME).await })
            .await
    }

    #[inline]
    pub(crate) async fn get<T>(&self, url: T) -> Result<Response, Error>
    where
        T: AsRef<str>,
    {
        Ok(self
            .client()
            .await?
            .get(SfacgClient::HOST.to_string() + url.as_ref())
            .basic_auth(SfacgClient::USERNAME, Some(SfacgClient::PASSWORD))
            .header("sfsecurity", self.sf_security()?)
            .send()
            .await?)
    }

    #[inline]
    pub(crate) async fn get_query<T, E>(&self, url: T, query: &E) -> Result<Response, Error>
    where
        T: AsRef<str>,
        E: Serialize,
    {
        Ok(self
            .client()
            .await?
            .get(SfacgClient::HOST.to_string() + url.as_ref())
            .query(query)
            .basic_auth(SfacgClient::USERNAME, Some(SfacgClient::PASSWORD))
            .header("sfsecurity", self.sf_security()?)
            .send()
            .await?)
    }

    #[inline]
    pub(crate) async fn get_rss(&self, url: &Url) -> Result<Response, Error> {
        let response = self.client_rss().await?.get(url.clone()).send().await?;
        crate::check_status(response.status(), format!("HTTP request failed: `{url}`"))?;

        Ok(response)
    }

    #[inline]
    pub(crate) async fn post<T, E>(&self, url: T, json: &E) -> Result<Response, Error>
    where
        T: AsRef<str>,
        E: Serialize,
    {
        Ok(self
            .client()
            .await?
            .post(SfacgClient::HOST.to_string() + url.as_ref())
            .basic_auth(SfacgClient::USERNAME, Some(SfacgClient::PASSWORD))
            .header("sfsecurity", self.sf_security()?)
            .json(json)
            .send()
            .await?)
    }

    #[inline]
    fn sf_security(&self) -> Result<String, Error> {
        let uuid = Uuid::new_v4();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let device_token = crate::uid();

        let data = format!("{uuid}{timestamp}{device_token}{}", SfacgClient::SALT);
        let md5 = hash::hash(MessageDigest::md5(), data.as_bytes())?;

        Ok(format!(
            "nonce={uuid}&timestamp={timestamp}&devicetoken={device_token}&sign={}",
            hex_simd::encode_to_string(md5, AsciiCase::Upper)
        ))
    }
}
