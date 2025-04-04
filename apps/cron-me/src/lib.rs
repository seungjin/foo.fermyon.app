use serde_json::json;
use serde_json::Value;
use spin_cron_sdk::{cron_component, Metadata};
use spin_sdk::http::{ErrorCode, IntoResponse, Method, Request, Response};
use std::env;
use std::error::Error;
use std::str;
use thiserror::Error;

#[cron_component]
async fn handle_cron_event(metadata: Metadata) -> anyhow::Result<()> {
    println!("foo foo");
    Ok(())
}
