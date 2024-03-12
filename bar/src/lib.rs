use anyhow::Result;
use spin_sdk::{
    http::{IntoResponse, Request, Response},
    http_component,
    variables
};
use url::Url;

/// A simple Spin HTTP component.
#[http_component]
async fn handle_bar(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));

    let u = req.uri();
    println!("{u}");
    let parsed_url = Url::parse(u)?;


    let (arg_1, arg_2, arg_3, arg_4, arg_5, arg_6) =
        ( variables::get("arg_foo").expect("Not getting arg1foo"),
          variables::get("arg2").unwrap(),
          variables::get("arg3").unwrap(),
          variables::get("arg4").unwrap(),
          variables::get("arg5").unwrap(),
          variables::get("arg6").unwrap()
    );

    println!("{}, {}, {}, {}, {} ,{}", arg_1, arg_2, arg_3, arg_4, arg_5, arg_6);

    Ok(
        Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("Hello, I'm foo.spinapp.")
            .build()
    )
}

