use std::time::{SystemTime, UNIX_EPOCH};

use boring::hash::{self, MessageDigest};
use hex_simd::AsciiCase;
use reqwest::Response;
use serde::Serialize;
use tokio::sync::OnceCell;
use url::Url;
use uuid::Uuid;

use crate::{here, Error, ErrorLocation, HTTPClient, Location, NovelDB, SfacgClient};

impl SfacgClient {
    const APP_NAME: &str = "sfacg";

    const HOST: &str = "https://api.sfacg.com";
    const USER_AGENT_PREFIX: &str = "boluobao/4.9.38(iOS;16.2)/appStore/";
    const USER_AGENT_RSS: &str = "SFReader/4.9.38 (iPhone; iOS 16.2; Scale/3.00)";

    const USERNAME: &str = "apiuser";
    const PASSWORD: &str = "3s#1-yt6e*Acv@qer";

    const SALT: &str = "FMLxgOdsfxmN!Dt4";

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

    pub(crate) async fn db(&self) -> Result<&NovelDB, Error> {
        self.db
            .get_or_try_init(|| async { NovelDB::new(SfacgClient::APP_NAME).await })
            .await
    }

    pub(crate) async fn get<T>(&self, url: T) -> Result<Response, Error>
    where
        T: AsRef<str>,
    {
        let response = self
            .client()
            .await
            .location(here!())?
            .get(SfacgClient::HOST.to_string() + url.as_ref())
            .basic_auth(SfacgClient::USERNAME, Some(SfacgClient::PASSWORD))
            .header("sfsecurity", self.sf_security().location(here!())?)
            .send()
            .await
            .location(here!())?;

        Ok(response)
    }

    pub(crate) async fn get_query<T, E>(&self, url: T, query: &E) -> Result<Response, Error>
    where
        T: AsRef<str>,
        E: Serialize,
    {
        let response = self
            .client()
            .await
            .location(here!())?
            .get(SfacgClient::HOST.to_string() + url.as_ref())
            .query(query)
            .basic_auth(SfacgClient::USERNAME, Some(SfacgClient::PASSWORD))
            .header("sfsecurity", self.sf_security().location(here!())?)
            .send()
            .await
            .location(here!())?;

        Ok(response)
    }

    pub(crate) async fn get_rss(&self, url: &Url) -> Result<Response, Error> {
        let response = self
            .client_rss()
            .await
            .location(here!())?
            .get(url.clone())
            .send()
            .await
            .location(here!())?;

        Ok(response)
    }

    pub(crate) async fn post<T, E>(&self, url: T, json: &E) -> Result<Response, Error>
    where
        T: AsRef<str>,
        E: Serialize,
    {
        let response = self
            .client()
            .await
            .location(here!())?
            .post(SfacgClient::HOST.to_string() + url.as_ref())
            .basic_auth(SfacgClient::USERNAME, Some(SfacgClient::PASSWORD))
            .header("sfsecurity", self.sf_security().location(here!())?)
            .json(json)
            .send()
            .await
            .location(here!())?;

        Ok(response)
    }

    fn sf_security(&self) -> Result<String, Error> {
        let uuid = Uuid::new_v4();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .location(here!())?
            .as_secs();
        let device_token = crate::uid();

        let data = format!("{}{}{}{}", uuid, timestamp, device_token, SfacgClient::SALT);
        let md5 = hash::hash(MessageDigest::md5(), data.as_bytes()).location(here!())?;

        Ok(format!(
            "nonce={}&timestamp={}&devicetoken={}&sign={}",
            uuid,
            timestamp,
            device_token,
            hex_simd::encode_to_string(md5, AsciiCase::Upper)
        ))
    }
}