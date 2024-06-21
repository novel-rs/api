use std::sync::RwLock;

use const_format::concatcp;
use once_cell::sync::OnceCell as SyncOnceCell;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Response;
use ring::{
    digest::Digest,
    hmac::{self, Key},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Map, Value};
use tokio::sync::OnceCell;
use tracing::{error, info};
use url::{form_urlencoded, Url};

use super::Config;
use crate::{CiweimaoClient, Error, HTTPClient, NovelDB};

impl CiweimaoClient {
    const APP_NAME: &'static str = "ciweimao";
    const HOST: &'static str = "https://app.hbooker.com";

    pub(crate) const OK: &'static str = "100000";
    pub(crate) const LOGIN_EXPIRED: &'static str = "200100";
    pub(crate) const NOT_FOUND: &'static str = "320001";
    pub(crate) const ALREADY_SIGNED_IN: &'static str = "340001";

    pub(crate) const APP_VERSION: &'static str = "2.9.328";
    pub(crate) const DEVICE_TOKEN: &'static str = "ciweimao_";

    const USER_AGENT: &'static str =
        "Android com.kuangxiangciweimao.novel 2.9.328,google, sdk_gphone64_arm64, 31, 12";
    const USER_AGENT_RSS: &'static str =
        "Dalvik/2.1.0 (Linux; U; Android 12; sdk_gphone64_arm64 Build/SE1A.220203.002.A1)";

    const AES_KEY: &'static str = "zG2nSeEfSHfvTCHy5LCcqtBbQehKNLXn";
    const HMAC_KEY: &'static str = "a90f3731745f1c30ee77cb13fc00005a";
    const SIGNATURES: &'static str = concatcp!(CiweimaoClient::HMAC_KEY, "CkMxWNB666");

    /// Create a ciweimao client
    pub async fn new() -> Result<Self, Error> {
        let config: Option<Config> = crate::load_config_file(CiweimaoClient::APP_NAME)?;

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
    pub(crate) fn try_account(&self) -> String {
        if self.has_token() {
            self.config
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .account
                .to_string()
        } else {
            String::default()
        }
    }

    #[must_use]
    pub(crate) fn try_login_token(&self) -> String {
        if self.has_token() {
            self.config
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .login_token
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
            .get_or_try_init(|| async { NovelDB::new(CiweimaoClient::APP_NAME).await })
            .await
    }

    pub(crate) async fn client(&self) -> Result<&HTTPClient, Error> {
        self.client
            .get_or_try_init(|| async {
                HTTPClient::builder(CiweimaoClient::APP_NAME)
                    .user_agent(CiweimaoClient::USER_AGENT)
                    // 因为 HTTP response body 是加密的，所以压缩是没有意义的
                    .allow_compress(false)
                    .proxy(self.proxy.clone())
                    .no_proxy(self.no_proxy)
                    .cert(self.cert_path.clone())
                    .build()
                    .await
            })
            .await
    }

    async fn client_rss(&self) -> Result<&HTTPClient, Error> {
        self.client_rss
            .get_or_try_init(|| async {
                HTTPClient::builder(CiweimaoClient::APP_NAME)
                    .user_agent(CiweimaoClient::USER_AGENT_RSS)
                    .proxy(self.proxy.clone())
                    .no_proxy(self.no_proxy)
                    .cert(self.cert_path.clone())
                    .build()
                    .await
            })
            .await
    }

    pub(crate) async fn get_query<T, E>(&self, url: T, query: E) -> Result<Response, Error>
    where
        T: AsRef<str>,
        E: Serialize,
    {
        let response = self
            .client()
            .await?
            .get(CiweimaoClient::HOST.to_string() + url.as_ref())
            .query(&query)
            .send()
            .await?;
        crate::check_status(
            response.status(),
            format!("HTTP request failed: `{}`", url.as_ref()),
        )?;

        Ok(response)
    }

    pub(crate) async fn post<T, E, R>(&self, url: T, form: E) -> Result<R, Error>
    where
        T: AsRef<str>,
        E: Serialize,
        R: DeserializeOwned,
    {
        let mut count = 0;

        let response = loop {
            let response = self
                .client()
                .await?
                .post(CiweimaoClient::HOST.to_string() + url.as_ref())
                .form(&self.append_param(&form)?)
                .send()
                .await;

            if let Ok(response) = response {
                break response;
            } else {
                info!(
                    "HTTP request failed: `{}`, retry, number of times: `{}`",
                    response.as_ref().unwrap_err(),
                    count + 1
                );

                count += 1;
                if count > 3 {
                    response?;
                }
            }
        };

        crate::check_status(
            response.status(),
            format!("HTTP request failed: `{}`", url.as_ref()),
        )?;

        let bytes = response.bytes().await?;
        let bytes = crate::aes_256_cbc_no_iv_base64_decrypt(CiweimaoClient::get_aes_key(), &bytes)?;

        let str = simdutf8::basic::from_utf8(&bytes)?;
        Ok(serde_json::from_str(str)?)
    }

    pub(crate) async fn get_rss(&self, url: &Url) -> Result<Response, Error> {
        let response = self.client_rss().await?.get(url.clone()).send().await?;
        crate::check_status(response.status(), format!("HTTP request failed: `{url}`"))?;

        Ok(response)
    }

    fn append_param<T>(&self, query: T) -> Result<Map<String, Value>, Error>
    where
        T: Serialize,
    {
        let mut value = serde_json::to_value(query)?;
        let object = value.as_object_mut().unwrap();

        object.insert(
            String::from("app_version"),
            json!(CiweimaoClient::APP_VERSION),
        );
        object.insert(
            String::from("device_token"),
            json!(CiweimaoClient::DEVICE_TOKEN),
        );

        let rand_str = CiweimaoClient::get_rand_str();
        let p = self.hmac(&rand_str);
        object.insert(String::from("rand_str"), json!(rand_str));
        object.insert(String::from("p"), json!(p));

        if self.has_token() {
            object.insert(String::from("account"), json!(self.try_account()));
            object.insert(String::from("login_token"), json!(self.try_login_token()));
        }

        Ok(value.as_object().unwrap().clone())
    }

    #[must_use]
    fn get_aes_key() -> &'static [u8] {
        static AES_KEY: SyncOnceCell<Digest> = SyncOnceCell::new();
        AES_KEY
            .get_or_init(|| crate::sha256(CiweimaoClient::AES_KEY.as_bytes()))
            .as_ref()
    }

    #[must_use]
    fn get_hmac_key() -> &'static Key {
        static HMAC_KEY: SyncOnceCell<Key> = SyncOnceCell::new();
        HMAC_KEY
            .get_or_init(|| hmac::Key::new(hmac::HMAC_SHA256, CiweimaoClient::HMAC_KEY.as_bytes()))
    }

    fn get_rand_str() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(|c| char::from(c).to_lowercase().to_string())
            .collect()
    }

    fn hmac(&self, rand_str: &str) -> String {
        let msg: String = form_urlencoded::Serializer::new(String::new())
            .append_pair("account", &self.try_account())
            .append_pair("app_version", CiweimaoClient::APP_VERSION)
            .append_pair("rand_str", rand_str)
            .append_pair("signatures", CiweimaoClient::SIGNATURES)
            .finish();

        let tag = hmac::sign(CiweimaoClient::get_hmac_key(), msg.as_bytes());
        base64_simd::STANDARD.encode_to_string(tag.as_ref())
    }

    pub(crate) fn do_shutdown(&self) -> Result<(), Error> {
        if self.has_token() {
            crate::save_config_file(
                CiweimaoClient::APP_NAME,
                self.config.write().unwrap().take(),
            )?;
        } else {
            info!("No data can be saved to the configuration file");
        }

        Ok(())
    }
}

impl Drop for CiweimaoClient {
    fn drop(&mut self) {
        if let Err(err) = self.do_shutdown() {
            error!("Fail to save config file: `{err}`");
        }
    }
}

pub(crate) fn check_response_success(code: String, tip: Option<String>) -> Result<(), Error> {
    if code != CiweimaoClient::OK {
        Err(Error::NovelApi(format!(
            "{} request failed, code: `{code}`, msg: `{}`",
            CiweimaoClient::APP_NAME,
            tip.unwrap().trim()
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn check_already_signed_in(code: &str) -> bool {
    code == CiweimaoClient::ALREADY_SIGNED_IN
}
