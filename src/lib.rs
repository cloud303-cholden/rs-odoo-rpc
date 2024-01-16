use std::fmt::Display;

use anyhow::{Result, Context};
use itertools::Itertools;
use serde::Serialize;
use serde_json::{Value, json};

type BuilderResult<'a, S> = Result<&'a mut Client<S>>;
type ValueResult = Result<Value>;

#[derive(Serialize, Debug, Clone)]
pub struct Client<S>
{
    db: S,
    password: S,
    uid: u64,
    url: S,
    env: S,
    records: Vec<Value>,
}

impl<S> Client<S>
where
    S: Into<String> + Clone,
{
    pub async fn new(
        db: S,
        username: S,
        password: S,
        env: S,
        url: S,
    ) -> Result<Self> {
        let uid: u64 = reqwest::Client::new()
            .post(format!("{}/jsonrpc", url.clone().into()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "common",
                    "method": "login",
                    "args": [db.clone().into(), username.clone().into(), password.clone().into()],
                }
            }))
            .send().await?
            .json::<Value>().await?
            .get("result").context("Failed to get login result")?
            .as_u64().context("Failed to interpret user ID")?;

        Ok(Self {
            db,
            password,
            uid,
            env,
            url,
            records: vec![],
        })
    }

    pub fn env(&mut self, env: S) -> &mut Self {
        self.env = env;
        self
    }

    pub fn browse<I: Into<Value>>(&mut self, ids: I) -> &mut Self {
        self.records = match ids.into() {
            Value::Array(i) => i,
            Value::Number(i) => vec![Value::Number(i); 1],
            _ => unreachable!(),
        };
        self
    }

    pub fn ids(&self) -> Vec<Value> {
        self.records.clone()
    }

    pub async fn create(&mut self, data: Value) -> BuilderResult<S> {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url.clone().into()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db.clone().into(),
                        self.uid,
                        self.password.clone().into(),
                        self.env.clone().into(),
                        "create",
                        data,
                    ],
                },
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        self.records = match resp.get("result").context("Failed to get read result")? {
            Value::Array(val) => val.to_vec(),
            Value::Number(val) => vec![serde_json::to_value(val).unwrap()],
            _ => unimplemented!(),
        };
        
        Ok(self)
    }

    pub async fn write(&mut self, data: Value) -> BuilderResult<S> {
        let _ = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url.clone().into()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db.clone().into(),
                        self.uid,
                        self.password.clone().into(),
                        self.env.clone().into(),
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

        Ok(self)
    }

    pub async fn search(&mut self, domain: Value) -> BuilderResult<S> {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url.clone().into()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db.clone().into(),
                        self.uid,
                        self.password.clone().into(),
                        self.env.clone().into(),
                        "search",
                        domain,
                    ],
                },
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        self.records = resp
            .get("result").context("Failed to get read result")?
            .as_array().context("Failed to interpret result as array")?
            .to_vec();
        Ok(self)
    }

    pub async fn read(&mut self, fields: Value) -> ValueResult {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url.clone().into()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db.clone().into(),
                        self.uid,
                        self.password.clone().into(),
                        self.env.clone().into(),
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

        Ok(resp)
    }

    pub async fn search_read(&mut self, domain: Value, fields: Value) -> ValueResult {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url.clone().into()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db.clone().into(),
                        self.uid,
                        self.password.clone().into(),
                        self.env.clone().into(),
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

        Ok(resp)
    }

    pub async fn get(&mut self, field: &str) -> ValueResult {
        let resp = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url.clone().into()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db.clone().into(),
                        self.uid,
                        self.password.clone().into(),
                        self.env.clone().into(),
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

        let resp = resp
            .get("result").context("Failed to get read result")?
            .as_array().context("Failed to interpret read result")?
            .iter()
            .next().context("Failed to find any records")?
            .get(field).context("Read field not included in result")?
            .clone();
        Ok(resp)
    }

    pub async fn unlink(&mut self) -> BuilderResult<S> {
        let _ = reqwest::Client::new()
            .post(format!("{}/jsonrpc", self.url.clone().into()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.db.clone().into(),
                        self.uid,
                        self.password.clone().into(),
                        self.env.clone().into(),
                        "unlink",
                        self.records,
                    ],
                },
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        self.records = vec![];
        Ok(self)
    }

    fn _execute(&self, _method: &str) {}
}

impl<S> Display for Client<S>
where S: Into<String> + Clone + Display,
{
    #[allow(unstable_name_collisions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s: String = self.env.clone().into();
        s.push('(');
        let str_records: String = self.records
            .iter()
            .flat_map(|r| r.as_u64())
            .map(|r| r.to_string())
            .intersperse(", ".to_string())
            .collect();
        s.push_str(&str_records);
        s.push(')');
        write!(f, "{}", s)
    }
}
