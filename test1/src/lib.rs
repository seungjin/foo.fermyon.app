use anyhow::Result;
use spin_sdk::http_component;
use spin_sdk::http::{
    self, IncomingResponse, IntoResponse, Method, Params, Request, RequestBuilder, Response,
};

/// A simple Spin HTTP component.
#[http_component]
async fn handle_test1(req: Request) -> Result<Response> {
    println!("Handling request to {:?}", req.header("spin-full-url"));


    let request = RequestBuilder::new(Method::Get, "https://httpbin.org/get") 
        .body("hi!")
        .build();
    let response: IncomingResponse = http::send(request).await?;
    let status = response.status();

    let body = String::from_utf8(response.into_body().await.unwrap()).unwrap();
    println!("status --> {status}");
    println!("response body -->\n{body}");


    
    Ok(
        http::Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("Hello, Fermyon").build()
    )
   
}
