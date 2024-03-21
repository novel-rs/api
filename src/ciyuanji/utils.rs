use std::{
    sync::RwLock,
    time::{SystemTime, UNIX_EPOCH},
};

use hex_simd::AsciiCase;
use reqwest::{header::HeaderValue, Response};
use serde::Serialize;
use serde_json::json;
use tokio::sync::OnceCell;
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

use super::Config;
use crate::{CiyuanjiClient, Error, HTTPClient, NovelDB};

impl CiyuanjiClient {
    const APP_NAME: &'static str = "ciyuanji";
    const HOST: &'static str = "https://api.hwnovel.com/api/ciyuanji/client";

    pub(crate) const OK: &'static str = "200";
    pub(crate) const FAILED: &'static str = "400";
    pub(crate) const ALREADY_SIGNED_IN_MSG: &'static str = "今日已签到";

    const VERSION: &'static str = "3.4.1";
    const PLATFORM: &'static str = "1";

    const USER_AGENT: &'static str = "Mozilla/5.0 (Linux; Android 11; Pixel 4 XL Build/RP1A.200720.009; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/92.0.4515.115 Mobile Safari/537.36";
    const USER_AGENT_RSS: &'static str =
        "Dalvik/2.1.0 (Linux; U; Android 12; sdk_gphone64_arm64 Build/SE1A.220203.002.A1)";

    pub(crate) const DES_KEY: &'static str = "ZUreQN0E";
    const KEY_PARAM: &'static str = "NpkTYvpvhJjEog8Y051gQDHmReY54z5t3F0zSd9QEFuxWGqfC8g8Y4GPuabq0KPdxArlji4dSnnHCARHnkqYBLu7iIw55ibTo18";

    /// Create a ciyuanji client
    pub async fn new() -> Result<Self, Error> {
        let config: Option<Config> = crate::load_config_file(CiyuanjiClient::APP_NAME)?;

        Ok(Self {
            proxy: None,
            no_proxy: false,
            cert_path: None,
            client: OnceCell::new(),
            client_rss: OnceCell::new(),
            db: OnceCell::new(),
            config: RwLock::new(config),
        })
    }

    #[must_use]
    pub(crate) fn try_token(&self) -> String {
        if self.has_token() {
            self.config
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .token
                .to_string()
        } else {
            String::default()
        }
    }

    #[must_use]
    pub(crate) fn has_token(&self) -> bool {
        self.config.read().unwrap().is_some()
    }

    pub(crate) fn save_token(&self, config: Config) {
        *self.config.write().unwrap() = Some(config);
    }

    pub(crate) async fn db(&self) -> Result<&NovelDB, Error> {
        self.db
            .get_or_try_init(|| async { NovelDB::new(CiyuanjiClient::APP_NAME).await })
            .await
    }

    pub(crate) async fn client(&self) -> Result<&HTTPClient, Error> {
        self.client
            .get_or_try_init(|| async {
                HTTPClient::builder(CiyuanjiClient::APP_NAME)
                    .add_header("version", HeaderValue::from_static(CiyuanjiClient::VERSION))
                    .add_header(
                        "platform",
                        HeaderValue::from_static(CiyuanjiClient::PLATFORM),
                    )
                    .user_agent(CiyuanjiClient::USER_AGENT)
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
                HTTPClient::builder(CiyuanjiClient::APP_NAME)
                    .user_agent(CiyuanjiClient::USER_AGENT_RSS)
                    .proxy(self.proxy.clone())
                    .no_proxy(self.no_proxy)
                    .cert(self.cert_path.clone())
                    .build()
                    .await
            })
            .await
    }

    pub(crate) async fn get<T>(&self, url: T) -> Result<Response, Error>
    where
        T: AsRef<str>,
    {
        let response = self
            .client()
            .await?
            .get(CiyuanjiClient::HOST.to_string() + url.as_ref())
            .query(&GenericRequest::new(json!({}))?)
            .header("token", self.try_token())
            .send()
            .await?;
        crate::check_status(
            response.status(),
            format!("HTTP request failed: `{}`", url.as_ref()),
        )?;

        Ok(response)
    }

    pub(crate) async fn get_query<T, E>(&self, url: T, query: E) -> Result<Response, Error>
    where
        T: AsRef<str>,
        E: Serialize,
    {
        let response = self
            .client()
            .await?
            .get(CiyuanjiClient::HOST.to_string() + url.as_ref())
            .query(&GenericRequest::new(query)?)
            .header("token", self.try_token())
            .send()
            .await?;
        crate::check_status(
            response.status(),
            format!("HTTP request failed: `{}`", url.as_ref()),
        )?;

        Ok(response)
    }

    pub(crate) async fn post<T, E>(&self, url: T, json: E) -> Result<Response, Error>
    where
        T: AsRef<str>,
        E: Serialize,
    {
        let response = self
            .client()
            .await?
            .post(CiyuanjiClient::HOST.to_string() + url.as_ref())
            .json(&GenericRequest::new(json)?)
            .header("token", self.try_token())
            .send()
            .await?;
        crate::check_status(
            response.status(),
            format!("HTTP request failed: `{}`", url.as_ref()),
        )?;

        Ok(response)
    }

    pub(crate) async fn get_rss(&self, url: &Url) -> Result<Response, Error> {
        let response = self.client_rss().await?.get(url.clone()).send().await?;
        crate::check_status(response.status(), format!("HTTP request failed: `{url}`"))?;

        Ok(response)
    }

    pub(crate) fn do_shutdown(&self) -> Result<(), Error> {
        if self.has_token() {
            crate::save_config_file(
                CiyuanjiClient::APP_NAME,
                self.config.write().unwrap().take(),
            )?;
        } else {
            info!("No data can be saved to the configuration file");
        }

        Ok(())
    }
}

impl Drop for CiyuanjiClient {
    fn drop(&mut self) {
        if let Err(err) = self.do_shutdown() {
            error!("Fail to save config file: `{err}`");
        }
    }
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenericRequest {
    pub param: String,
    pub request_id: String,
    pub sign: String,
    pub timestamp: u128,
}

impl GenericRequest {
    fn new<T>(json: T) -> Result<Self, Error>
    where
        T: Serialize,
    {
        let mut json = serde_json::to_value(&json)?;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        json.as_object_mut()
            .unwrap()
            .insert(String::from("timestamp"), json!(timestamp));

        let param = crate::des_ecb_base64_encrypt(CiyuanjiClient::DES_KEY, json.to_string())?;

        let request_id = Uuid::new_v4().as_simple().to_string();

        let sign = crate::md5_hex(
            base64_simd::STANDARD.encode_to_string(format!(
                "param={param}&requestId={request_id}&timestamp={timestamp}&key={}",
                CiyuanjiClient::KEY_PARAM
            )),
            AsciiCase::Upper,
        );

        Ok(Self {
            param,
            request_id,
            sign,
            timestamp,
        })
    }
}

pub(crate) fn check_response_success(code: String, msg: String) -> Result<(), Error> {
    if code != CiyuanjiClient::OK {
        Err(Error::NovelApi(format!(
            "{} request failed, code: `{code}`, msg: `{}`",
            CiyuanjiClient::APP_NAME,
            msg.trim()
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn check_already_signed_in(code: &str, msg: &str) -> bool {
    code == CiyuanjiClient::FAILED && msg == CiyuanjiClient::ALREADY_SIGNED_IN_MSG
}
