use anyhow::Result;
use odoorpc::{Client, Credentials};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let creds = Credentials {
        username: "username",
        password: "password",
        db: "db",
        url: "http://localhost:8069",
    };
    let mut client = Client::new(
        creds,
        Some("res.users"),
    ).await?;

    let _resp: String = client
        .env("res.partner")
        .browse(vec![1])
        .get("name")
        .await?;

    let _resp: String = client
        .env("res.partner")
        .browse(1)
        .get("name")
        .await?;

    let _resp = client
        .create(json!({
            "data": 1,
        }))
        .await?;

    Ok(())
}

