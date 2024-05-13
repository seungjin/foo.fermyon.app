mod bindings;

use crate::bindings::seungjin::rock::calc::{add, sub};

use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;

/// A simple Spin HTTP component.
#[http_component]
fn handle_foofoo2(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));

    let foo = add(1, 2);
    let bar = sub(3, 1);
    println!("{}, {}", foo, bar);

    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("Hello, Fermyon")
        .build())
}
