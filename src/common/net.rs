use std::{
    io::BufWriter,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use bytes::Bytes;
use cookie_store::{CookieStore, RawCookie, RawCookieParseError};
use reqwest::{
    header::{HeaderMap, HeaderValue, IntoHeaderName, ACCEPT, CONNECTION},
    redirect, Certificate, Client, Proxy, StatusCode,
};
use tokio::fs;
use tracing::{error, info};
use url::Url;

use crate::Error;

pub(crate) fn check_status<T>(code: StatusCode, msg: T) -> Result<(), Error>
where
    T: AsRef<str>,
{
    if code != StatusCode::OK {
        return Err(Error::Http {
            code,
            msg: msg.as_ref().trim().to_string(),
        });
    }

    Ok(())
}

#[must_use]
pub(crate) struct HTTPClientBuilder {
    app_name: &'static str,
    accept: Option<HeaderValue>,
    user_agent: String,
    cookie: bool,
    allow_compress: bool,
    proxy: Option<Url>,
    no_proxy: bool,
    cert_path: Option<PathBuf>,
    headers: HeaderMap,
}

impl HTTPClientBuilder {
    const COOKIE_FILE_NAME: &'static str = "cookie.json";

    const COOKIE_FILE_PASSWORD: &'static str = "gafqad-4Ratne-dirqom";
    const COOKIE_FILE_AAD: &'static str = "novel-rs-cookie";

    pub(crate) fn new(app_name: &'static str) -> Self {
        Self {
            app_name,
            accept: None,
            user_agent: String::from("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"),
            cookie: false,
            allow_compress: true,
            proxy: None,
            no_proxy: false,
            cert_path: None,
            headers: HeaderMap::new()
        }
    }

    pub(crate) fn accept(self, accept: &'static str) -> Self {
        Self {
            accept: Some(HeaderValue::from_static(accept)),
            ..self
        }
    }

    pub(crate) fn user_agent<T>(self, user_agent: T) -> Self
    where
        T: AsRef<str>,
    {
        Self {
            user_agent: user_agent.as_ref().to_string(),
            ..self
        }
    }

    pub(crate) fn cookie(self, flag: bool) -> Self {
        Self {
            cookie: flag,
            ..self
        }
    }

    pub(crate) fn allow_compress(self, flag: bool) -> Self {
        Self {
            allow_compress: flag,
            ..self
        }
    }

    pub(crate) fn proxy(self, proxy: Option<Url>) -> Self {
        Self { proxy, ..self }
    }

    pub(crate) fn no_proxy(self, flag: bool) -> Self {
        Self {
            no_proxy: flag,
            ..self
        }
    }

    pub(crate) fn cert<T>(self, cert_path: Option<T>) -> Self
    where
        T: AsRef<Path>,
    {
        Self {
            cert_path: cert_path.map(|path| path.as_ref().to_path_buf()),
            ..self
        }
    }

    pub(crate) fn add_header<K>(self, key: K, value: HeaderValue) -> Self
    where
        K: IntoHeaderName,
    {
        let mut result = self;
        result.headers.append(key, value);

        result
    }

    pub(crate) async fn build(self) -> Result<HTTPClient, Error> {
        let mut cookie_provider = None;
        if self.cookie {
            cookie_provider = Some(Arc::new(self.create_cookie_provider().await?));
        }

        let mut headers = self.headers;
        if self.accept.is_some() {
            headers.insert(ACCEPT, self.accept.unwrap());
            headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
        }

        let mut client_builder = Client::builder()
            .default_headers(headers)
            .redirect(redirect::Policy::none())
            .http2_keep_alive_interval(Duration::from_secs(5))
            .user_agent(self.user_agent);

        if !is_ci::cached() {
            client_builder = client_builder
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(30));
        }

        if let Some(jar) = &cookie_provider {
            client_builder = client_builder.cookie_provider(Arc::clone(jar));
        }

        if !self.allow_compress {
            client_builder = client_builder.no_gzip();
            client_builder = client_builder.no_brotli();
            client_builder = client_builder.no_deflate();
        }

        if let Some(proxy) = self.proxy {
            client_builder = client_builder.proxy(Proxy::all(proxy)?);
        }

        if self.no_proxy {
            client_builder = client_builder.no_proxy();
        }

        if let Some(cert_path) = self.cert_path {
            let cert = Certificate::from_pem(&fs::read(cert_path).await?)?;
            client_builder = client_builder.add_root_certificate(cert);
        }

        Ok(HTTPClient {
            app_name: self.app_name,
            cookie_provider,
            client: client_builder.build()?,
        })
    }

    async fn create_cookie_provider(&self) -> Result<Jar, Error> {
        let cookie_path = HTTPClientBuilder::cookie_path(self.app_name)?;

        let cookie_store = if fs::try_exists(&cookie_path).await? {
            info!("The cookie file is located at: `{}`", cookie_path.display());

            if let Ok(json) = super::aes_256_gcm_base64_decrypt(
                &cookie_path,
                HTTPClientBuilder::COOKIE_FILE_PASSWORD,
                HTTPClientBuilder::COOKIE_FILE_AAD,
            ) {
                CookieStore::load_json(json.as_bytes())?
            } else {
                error!("Fail to decrypt the cookie file, a new one will be created");
                CookieStore::default()
            }
        } else {
            info!(
                "The cookie file will be created at: `{}`",
                cookie_path.display()
            );

            fs::create_dir_all(cookie_path.parent().unwrap()).await?;
            CookieStore::default()
        };

        Ok(Jar::new(cookie_store))
    }

    fn cookie_path(app_name: &str) -> Result<PathBuf, Error> {
        let mut config_path = crate::config_dir_path(app_name)?;
        config_path.push(HTTPClientBuilder::COOKIE_FILE_NAME);

        Ok(config_path)
    }
}

#[must_use]
pub(crate) struct HTTPClient {
    app_name: &'static str,
    cookie_provider: Option<Arc<Jar>>,
    client: Client,
}

impl HTTPClient {
    pub(crate) fn builder(app_name: &'static str) -> HTTPClientBuilder {
        HTTPClientBuilder::new(app_name)
    }

    pub(crate) fn add_cookie(&self, cookie_str: &str, url: &Url) -> Result<(), Error> {
        self.cookie_provider
            .as_ref()
            .expect("Cookies not turned on")
            .0
            .write()
            .unwrap()
            .parse(cookie_str, url)?;

        Ok(())
    }

    pub(crate) fn shutdown(&self) -> Result<(), Error> {
        if self.cookie_provider.is_some() {
            let mut writer = BufWriter::new(Vec::new());
            self.cookie_provider
                .as_ref()
                .unwrap()
                .0
                .read()
                .unwrap()
                .save_json(&mut writer)?;
            let result = simdutf8::basic::from_utf8(writer.buffer())?.to_string();

            if !result.is_empty() {
                let cookie_path = HTTPClientBuilder::cookie_path(self.app_name)?;
                info!("Save the cookie file at: `{}`", cookie_path.display());

                super::aes_256_gcm_base64_encrypt(
                    result,
                    cookie_path,
                    HTTPClientBuilder::COOKIE_FILE_PASSWORD,
                    HTTPClientBuilder::COOKIE_FILE_AAD,
                )?;
            }
        }

        Ok(())
    }
}

impl Deref for HTTPClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl Drop for HTTPClient {
    fn drop(&mut self) {
        if let Err(err) = self.shutdown() {
            error!("Fail to save cookie: {err}");
        }
    }
}

struct Jar(RwLock<CookieStore>);

impl Jar {
    fn new(cookie_store: CookieStore) -> Jar {
        Jar(RwLock::new(cookie_store))
    }
}

impl reqwest::cookie::CookieStore for Jar {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        let mut write = self.0.write().unwrap();
        set_cookies(&mut write, cookie_headers, url);
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        let read = self.0.read().unwrap();
        cookies(&read, url)
    }
}

fn set_cookies(
    cookie_store: &mut CookieStore,
    cookie_headers: &mut dyn Iterator<Item = &HeaderValue>,
    url: &url::Url,
) {
    let cookies = cookie_headers.filter_map(|val| {
        std::str::from_utf8(val.as_bytes())
            .map_err(RawCookieParseError::from)
            .and_then(RawCookie::parse)
            .map(|c| c.into_owned())
            .ok()
    });
    cookie_store.store_response_cookies(cookies, url);
}

fn cookies(cookie_store: &CookieStore, url: &url::Url) -> Option<HeaderValue> {
    let s = cookie_store
        .get_request_values(url)
        .map(|(name, value)| format!("{}={}", name, value))
        .collect::<Vec<_>>()
        .join("; ");

    if s.is_empty() {
        return None;
    }

    HeaderValue::from_maybe_shared(Bytes::from(s)).ok()
}
