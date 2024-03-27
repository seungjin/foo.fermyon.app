use anyhow::Result;
use spin_cron_sdk::{cron_component, Error, Metadata};
use spin_sdk::http::{Request, Response};
use spin_sdk::variables;

#[cron_component]
async fn handle_cron_event(metadata: Metadata) -> Result<(), Error> {
    //let _ = foo(metadata).await;
    let key = variables::get("something").unwrap_or_default();
    let resp: Response = spin_sdk::http::send(Request::get(
        "https://random-data-api.fermyon.app/animals/json",
    ))
    .await
    .unwrap();

    println!("{:#?}", resp);

    println!(
        "[{}] Hello this is me running every {}",
        metadata.timestamp, key
    );
    Ok(())
}

async fn foo(metadata: Metadata) -> Result<()> {
    let key = variables::get("something").unwrap_or_default();
    let resp: Response = spin_sdk::http::send(Request::get(
        "https://random-data-api.fermyon.app/animals/json",
    ))
    .await
    .unwrap();

    println!("{:#?}", resp);

    println!(
        "[{}] Hello this is me running every {}",
        metadata.timestamp, key
    );
    Ok(())
}
