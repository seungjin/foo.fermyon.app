use anyhow::Result;
use spin_cron_sdk::{cron_component, Error, Metadata};
use spin_sdk::http::{Request, Response};

#[cron_component]
async fn handle_cron_event(metadata: Metadata) -> Result<(), Error> {
    let _ = foo().await;
    Ok(())
}

async fn foo() -> Result<()> {
    let url = "https://reqbin.com/echo/get/json";
    let resp: Response = spin_sdk::http::send(Request::get(url)).await.unwrap();

    let a = resp.status();
    println!("----{}", a);
    Ok(())
}
