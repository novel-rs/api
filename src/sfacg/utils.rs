use std::time::{SystemTime, UNIX_EPOCH};

use hex_simd::AsciiCase;
use reqwest::{header::HeaderValue, Response};
use serde::Serialize;
use tokio::sync::OnceCell;
use url::Url;
use uuid::Uuid;

use crate::{Error, HTTPClient, NovelDB, SfacgClient};

#[cfg(target_os = "windows")]
macro_rules! PATH_SEPARATOR {
    () => {
        r"\"
    };
}

#[cfg(not(target_os = "windows"))]
macro_rules! PATH_SEPARATOR {
    () => {
        r"/"
    };
}

include!(concat!(env!("OUT_DIR"), PATH_SEPARATOR!(), "codegen.rs"));

impl SfacgClient {
    const APP_NAME: &'static str = "sfacg";

    const HOST: &'static str = "https://api.sfacg.com";
    const USER_AGENT: &'static str = "boluobao/5.0.60(android;31)/H5/{}/H5";
    const USER_AGENT_RSS: &'static str =
        "Dalvik/2.1.0 (Linux; U; Android 12; sdk_gphone64_arm64 Build/SE1A.220203.002.A1)";

    const USERNAME: &'static str = "androiduser";
    const PASSWORD: &'static str = "1a#$51-yt69;*Acv@qxq";

    const SALT: &'static str = "FN_Q29XHVmfV3mYX";

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

    pub(crate) async fn db(&self) -> Result<&NovelDB, Error> {
        self.db
            .get_or_try_init(|| async { NovelDB::new(SfacgClient::APP_NAME).await })
            .await
    }

    pub(crate) async fn client(&self) -> Result<&HTTPClient, Error> {
        self.client
            .get_or_try_init(|| async {
                let device_token = crate::uid();
                let user_agent = SfacgClient::USER_AGENT.replace("{}", device_token);

                HTTPClient::builder(SfacgClient::APP_NAME)
                    .accept("application/vnd.sfacg.api+json;version=1")
                    .add_header("accept-charset", HeaderValue::from_static("UTF-8"))
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
                    .user_agent(SfacgClient::USER_AGENT_RSS)
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
        Ok(self
            .client()
            .await?
            .get(SfacgClient::HOST.to_string() + url.as_ref())
            .basic_auth(SfacgClient::USERNAME, Some(SfacgClient::PASSWORD))
            .header("sfsecurity", self.sf_security()?)
            .send()
            .await?)
    }

    pub(crate) async fn get_query<T, E>(&self, url: T, query: E) -> Result<Response, Error>
    where
        T: AsRef<str>,
        E: Serialize,
    {
        Ok(self
            .client()
            .await?
            .get(SfacgClient::HOST.to_string() + url.as_ref())
            .query(&query)
            .basic_auth(SfacgClient::USERNAME, Some(SfacgClient::PASSWORD))
            .header("sfsecurity", self.sf_security()?)
            .send()
            .await?)
    }

    pub(crate) async fn post<T, E>(&self, url: T, json: E) -> Result<Response, Error>
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
            .json(&json)
            .send()
            .await?)
    }

    pub(crate) async fn put<T, E>(&self, url: T, json: E) -> Result<Response, Error>
    where
        T: AsRef<str>,
        E: Serialize,
    {
        Ok(self
            .client()
            .await?
            .put(SfacgClient::HOST.to_string() + url.as_ref())
            .basic_auth(SfacgClient::USERNAME, Some(SfacgClient::PASSWORD))
            .header("sfsecurity", self.sf_security()?)
            .json(&json)
            .send()
            .await?)
    }

    pub(crate) async fn get_rss(&self, url: &Url) -> Result<Response, Error> {
        let response = self.client_rss().await?.get(url.clone()).send().await?;
        crate::check_status(response.status(), format!("HTTP request failed: `{url}`"))?;

        Ok(response)
    }

    fn sf_security(&self) -> Result<String, Error> {
        let uuid = Uuid::new_v4();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let device_token = crate::uid();

        let sign = crate::md5_hex(
            format!("{uuid}{timestamp}{device_token}{}", SfacgClient::SALT),
            AsciiCase::Upper,
        );

        Ok(format!(
            "nonce={uuid}&timestamp={timestamp}&devicetoken={device_token}&sign={sign}"
        ))
    }

    pub(crate) fn convert(content: String) -> String {
        let mut result = String::new();

        for c in content.chars() {
            let code_point = c as u32;

            if (19968..=19968 + 0x51A5).contains(&code_point) {
                result.push(*CHARACTER_MAPPER.get(&c).unwrap_or(&c));
            } else {
                result.push(c)
            }
        }

        result
    }
}
