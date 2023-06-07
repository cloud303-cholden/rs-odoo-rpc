use std::fmt::Display;

use reqwest::IntoUrl;
use serde::Serialize;
use serde_json::{Value, json};

type BuilderResult<'a, T, U> = Result<&'a mut Client<T, U>, Box<dyn std::error::Error>>;
type ValueResult = Result<Value, Box<dyn std::error::Error>>;

#[derive(Serialize)]
pub struct Client<T, U>
where
    T: Display + serde::ser::Serialize,
    U: IntoUrl + Copy + Display,
{
    db: T,
    password: T,
    uid: u64,
    url: U,
    env: T,
    records: Vec<Value>,
}

impl<T, U> Client<T, U>
where
    T: Display + serde::ser::Serialize,
    U: IntoUrl + Copy + Display,
{
    pub async fn new(
        db: T,
        username: T,
        password: T,
        env: T,
        url: U,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let uid: u64 = reqwest::Client::new()
            .post(format!("{}/jsonrpc", url))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "common",
                    "method": "login",
                    "args": [db, username, password],
                }
            }))
            .send().await?
            .json::<Value>().await?
            .get("result").ok_or("Failed to get login result")?
            .as_u64().ok_or("Failed to interpret user ID")?;

        Ok(Self {
            db,
            password,
            uid,
            env,
            url,
            records: vec![],
        })
    }

    pub fn env(&mut self, env: T) -> &mut Self {
        self.env = env;
        self
    }

    pub fn browse(&mut self, id: u64) -> &mut Self {
        self.records = vec![serde_json::to_value(id).unwrap(); 1];
        self
    }

    pub async fn create(&mut self, data: Value) -> BuilderResult<T, U> {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db,
                        self.uid,
                        self.password,
                        self.env,
                        "create",
                        data,
                    ],
                },
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;
        dbg!(&resp);

        let resp = resp
            .get("result").ok_or("Failed to get read result")?
            .as_array().ok_or("Failed to interpret read result")?;
        self.records = resp.to_vec();
        Ok(self)
    }

    pub async fn write(&mut self, data: Value) -> BuilderResult<T, U> {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db,
                        self.uid,
                        self.password,
                        self.env,
                        "write",
                        self.records,
                        data,
                    ],
                },
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;
        dbg!(&resp);

        Ok(self)
    }

    pub async fn search(&mut self, domain: Value) -> BuilderResult<T, U> {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db,
                        self.uid,
                        self.password,
                        self.env,
                        "search",
                        domain,
                    ],
                },
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;
        dbg!(&resp);

        self.records = resp
            .get("result").ok_or("Failed to get read result")?
            .as_array().ok_or("Failed to interpret result as array")?
            .to_vec();
        Ok(self)
    }

    pub async fn read(&mut self, fields: Value) -> ValueResult {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db,
                        self.uid,
                        self.password,
                        self.env,
                        "read",
                        self.records,
                        fields,
                    ],
                },
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;
        dbg!(&resp);

        Ok(resp)
    }

    pub async fn search_read(&mut self, domain: Value, fields: Value) -> ValueResult {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db,
                        self.uid,
                        self.password,
                        self.env,
                        "search_read",
                        domain,
                        fields,
                    ],
                },
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;
        dbg!(&resp);

        Ok(resp)
    }

    pub async fn get(&mut self, field: &str) -> ValueResult {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db,
                        self.uid,
                        self.password,
                        self.env,
                        "read",
                        self.records,
                        [field],
                    ],
                },
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;
        dbg!(&resp);

        let resp = resp
            .get("result").ok_or("Failed to get read result")?
            .as_array().ok_or("Failed to interpret read result")?
            .iter()
            .next().ok_or("Failed to find any records")?
            .get(field).ok_or("Read field not included in result")?
            .clone();
        Ok(resp)
    }

    pub async fn unlink(&mut self) -> BuilderResult<T, U> {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db,
                        self.uid,
                        self.password,
                        self.env,
                        "unlink",
                        self.records,
                    ],
                },
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;
        dbg!(&resp);

        self.records = vec![];
        Ok(self)
    }

    fn _execute(&self, _method: &str) {}
}

impl<T, U> Display for Client<T, U>
where
    T: Display + serde::ser::Serialize,
    U: IntoUrl + Copy + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.env,
            self.records
                .iter()
                .map(|c| format!("{},", c.as_u64().unwrap()))
                .collect::<String>()
        )
    }
}
