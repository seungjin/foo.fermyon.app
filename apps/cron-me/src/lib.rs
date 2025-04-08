use serde_json::json;
use serde_json::Value;
use spin_cron_sdk::{cron_component, Metadata};
use spin_sdk::http::{ErrorCode, IntoResponse, Method::Get, Request, Response};
use std::env;
use std::error::Error;
use std::str;
use thiserror::Error;

#[cron_component]
async fn handle_cron_event(metadata: Metadata) -> anyhow::Result<()> {
    let request = Request::builder()
        .method(Get)
        .uri("https://www.fermyon.com/")
        .build();

    // Send the request and await the response
    let response: Response = spin_sdk::http::send(request).await?;

    // Use the outbound response body
    println!("{}", response.status());

    Ok(())
}
