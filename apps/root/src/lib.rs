use anyhow::Result;
use spin_sdk::{
    http::{IntoResponse, Params, Request, Response, Router},
    http_component,
};

/// A simple Spin HTTP component.
#[http_component]
async fn handle_root(req: Request) -> Result<impl IntoResponse> {
    // let mut router = Router::new();
    // router.get("/", root_page);
    // router.get("/*", not_found_page);
    // router.handle(req)
    //
    //
    println!("Handling request to {:?}", req.header("spin-full-url"));
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("Hello World!")
        .build())
}

// #[http_component]
// fn handle_foo1(req: Request) -> anyhow::Result<impl IntoResponse> {
//     println!("Handling request to {:?}", req.header("spin-full-url"));
//     Ok(Response::builder()
//         .status(200)
//         .header("content-type", "text/plain")
//         .body("Hello World!")
//         .build())
// }

// async fn root_page(_req: Request, _params: Params) -> Result<Response> {
//     Ok(http::Response::builder()
//         .status(200)
//         .header("Content-Type", "text")
//         .body(Some("Hello, Sky".into()))?)
// }

// async fn not_found_page(_req: Request, _params: Params) -> Result<Response> {
//     Ok(http::Response::builder()
//         .status(404)
//         .header("Content-Type", "text")
//         .body(Some("404, Page Not Found".into()))?)
// }
