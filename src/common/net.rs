use std::{
    io::BufWriter,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use http::StatusCode;
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CONNECTION},
    redirect, Certificate, Client, Proxy,
};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use tokio::fs;
use tracing::{error, info};
use url::Url;

use crate::Error;

#[inline]
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
    accept: HeaderValue,
    accept_language: HeaderValue,
    user_agent: String,
    cookie: bool,
    allow_compress: bool,
    proxy: Option<Url>,
    no_proxy: bool,
    cert_path: Option<PathBuf>,
}

impl HTTPClientBuilder {
    const COOKIE_FILE_NAME: &str = "cookie.json";

    const COOKIE_FILE_PASSWORD: &str = "gafqad-4Ratne-dirqom";
    const COOKIE_FILE_AAD: &str = "novel-rs-cookie";

    pub(crate) fn new(app_name: &'static str) -> Self {
        Self {
            app_name,
            accept: HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9",
            ),
            accept_language: HeaderValue::from_static("zh-CN,zh;q=0.9"),
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36".to_string(),
            cookie: false,
            allow_compress: true,
            proxy: None,
            no_proxy: false,
            cert_path: None,
        }
    }

    pub(crate) fn accept(self, accept: &'static str) -> Self {
        Self {
            accept: HeaderValue::from_static(accept),
            ..self
        }
    }

    pub(crate) fn accept_language(self, accept_language: &'static str) -> Self {
        Self {
            accept_language: HeaderValue::from_static(accept_language),
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

    pub(crate) async fn build(self) -> Result<HTTPClient, Error> {
        let mut cookie_store = None;
        if self.cookie {
            cookie_store = Some(Arc::new(self.create_cookie_store().await?));
        }

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, self.accept);
        headers.insert(ACCEPT_LANGUAGE, self.accept_language);
        headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));

        let mut client_builder = Client::builder()
            .default_headers(headers)
            .redirect(redirect::Policy::none())
            .http2_keep_alive_interval(Duration::from_secs(10))
            .http2_keep_alive_timeout(Duration::from_secs(60))
            .user_agent(self.user_agent);

        if self.cookie {
            client_builder =
                client_builder.cookie_provider(Arc::clone(cookie_store.as_ref().unwrap()));
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
            cookie_store: RwLock::new(cookie_store),
            client: client_builder.build()?,
        })
    }

    async fn create_cookie_store(&self) -> Result<CookieStoreMutex, Error> {
        let cookie_path = HTTPClientBuilder::cookie_path(self.app_name)?;

        let cookie_store = if fs::try_exists(&cookie_path).await? {
            info!("The cookie file is located at: `{}`", cookie_path.display());

            let json = super::decrypt(
                &cookie_path,
                HTTPClientBuilder::COOKIE_FILE_PASSWORD,
                HTTPClientBuilder::COOKIE_FILE_AAD,
            )?;

            CookieStore::load_json(json.as_bytes())?
        } else {
            info!(
                "The cookie file will be created at: `{}`",
                cookie_path.display()
            );

            fs::create_dir_all(cookie_path.parent().unwrap()).await?;
            CookieStore::default()
        };

        Ok(CookieStoreMutex::new(cookie_store))
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
    cookie_store: RwLock<Option<Arc<CookieStoreMutex>>>,
    client: Client,
}

impl HTTPClient {
    pub(crate) fn builder(app_name: &'static str) -> HTTPClientBuilder {
        HTTPClientBuilder::new(app_name)
    }

    pub(crate) fn add_cookie(&self, cookie_str: &str, url: &Url) -> Result<(), Error> {
        self.cookie_store
            .write()
            .unwrap()
            .as_ref()
            .expect("Cookies not turned on")
            .lock()
            .unwrap()
            .parse(cookie_str, url)?;

        Ok(())
    }

    pub(crate) fn shutdown(&self) -> Result<(), Error> {
        if self.cookie_store.read().unwrap().is_some() {
            let mut writer = BufWriter::new(Vec::new());
            self.cookie_store
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .save_json(&mut writer)?;

            *self.cookie_store.write().unwrap() = None;

            let result = simdutf8::basic::from_utf8(writer.buffer())?.to_string();

            if !result.is_empty() {
                let cookie_path = HTTPClientBuilder::cookie_path(self.app_name)?;

                info!("Save the cookie file at: `{}`", cookie_path.display());
                super::encrypt(
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
        if let Err(error) = self.shutdown() {
            error!("Fail to save cookie: {error}");
        }
    }
}
