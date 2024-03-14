use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use askama::Template;
use axum::{
    extract::{self, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
    task,
};

use super::GeetestInfoResponse;
use crate::Error;

#[derive(RustEmbed)]
#[folder = "templates"]
struct Asset;

struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self),
        )
            .into_response()
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    gt: String,
    challenge: String,
    new_captcha: bool,
}

async fn captcha(
    State(state): State<(GeetestInfoResponse, Sender<String>)>,
) -> Result<IndexTemplate, Error> {
    let (info, _) = state;

    Ok(IndexTemplate {
        gt: info.gt,
        challenge: info.challenge,
        new_captcha: info.new_captcha,
    })
}

async fn geetest_js() -> StaticFile<&'static str> {
    StaticFile("geetest.js")
}

async fn validate(
    extract::Path(validate): extract::Path<String>,
    State(state): State<(GeetestInfoResponse, Sender<String>)>,
) -> Html<&'static str> {
    let (_, tx) = state;
    tx.send(validate).await.unwrap();

    Html("Verification is successful, you can close the browser now")
}

pub(crate) async fn run_geetest(info: GeetestInfoResponse) -> Result<String, Error> {
    let (tx, mut rx) = mpsc::channel(1);

    let app = Router::new()
        .route("/captcha", get(captcha))
        .route("/geetest.js", get(geetest_js))
        .route("/validate/:validate", get(validate))
        .with_state((info, tx));

    let addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        portpicker::pick_unused_port().ok_or(Error::Port(String::from("No ports free")))?,
    );
    let listener = TcpListener::bind(addr).await?;

    let (stop_tx, stop_rx) = oneshot::channel();

    task::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                stop_rx.await.ok();
            })
            .await?;

        Ok::<_, Error>(())
    });

    open::that(format!("http://{}:{}/captcha", addr.ip(), addr.port()))?;

    let validate = rx.recv().await.unwrap();
    stop_tx.send(()).unwrap();

    Ok(validate)
}
