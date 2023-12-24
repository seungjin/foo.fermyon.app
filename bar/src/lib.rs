use anyhow::Result;
use spin_sdk::{
    http::{IntoResponse, Params, Request, Response, Router},
    http_component,
};
use url::Url;

/// A simple Spin HTTP component.
#[http_component]
fn handle_bar(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));

    let u = req.uri();
    println!("{u}");
    let parsed_url = Url::parse(u)?;

    Ok(http::Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("Hello, Fermyon")?)
}
