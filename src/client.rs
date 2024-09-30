use std::{fmt::Display, sync::Arc};

use anyhow::{Context, Result};
use itertools::Itertools;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use tracing::debug;

use crate::types::{ArrayOrNumber, Credentials, Response};

type BuilderResult<'a, S, E> = Result<&'a mut Client<S, E>>;
type ValueResult = Result<Value>;

pub trait ClientValues: AsRef<str> + std::fmt::Debug {}
impl<S: AsRef<str> + std::fmt::Debug> ClientValues for S {}

#[derive(Serialize, Debug, Clone)]
pub struct Client<S, E>
where
    S: ClientValues,
    E: ClientValues,
{
    #[serde(skip)]
    client: Arc<reqwest::Client>,
    #[serde(skip)]
    credentials: Credentials<S>,
    env: Env<E>,
    pub uid: u64,
    pub records: Vec<u64>,
}

impl<S, E> Display for Client<S, E>
where
    S: ClientValues,
    E: ClientValues,
{
    #[allow(unstable_name_collisions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s: String = self.env.0.as_ref().to_string();
        s.push('(');
        let str_records: String = self
            .records
            .iter()
            .map(|r| r.to_string())
            .intersperse(", ".to_string())
            .collect();
        s.push_str(&str_records);
        s.push(')');
        write!(f, "{}", s)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Env<E: AsRef<str>>(E);

impl Default for Env<&str> {
    fn default() -> Self {
        Self("res.users")
    }
}

impl Default for Env<String> {
    fn default() -> Self {
        Self(String::from("res.users"))
    }
}

impl<S, E> Client<S, E>
where
    S: ClientValues,
    E: ClientValues,
    Env<E>: Default,
{
    pub async fn new(credentials: Credentials<S>, env: Option<E>) -> Result<Self> {
        let env = match env {
            Some(s) => Env(s),
            None => Env::default(),
        };
        let client = Arc::new(reqwest::Client::new());
        let resp = client
            .post(format!("{}/jsonrpc", credentials.url.as_ref()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "common",
                    "method": "login",
                    "args": [
                        credentials.db.as_ref(),
                        credentials.username.as_ref(),
                        credentials.password.as_ref(),
                    ],
                },
            }))
            .send()
            .await?
            .json::<Response<u64>>()
            .await?;
        let uid = resp.result.to_record().unwrap();

        Ok(Self {
            client,
            credentials,
            uid,
            env,
            records: vec![],
        })
    }

    pub fn env(&mut self, env: E) -> &mut Self {
        self.env = Env(env);
        self
    }

    pub fn browse<T: Into<ArrayOrNumber>>(&mut self, ids: T) -> &mut Self {
        // :(
        let ids: ArrayOrNumber = ids.into();
        self.records = ids.into();
        self
    }

    pub fn ids(&self) -> Vec<u64> {
        self.records.clone()
    }

    pub async fn create(&mut self, data: Value) -> BuilderResult<S, E> {
        let resp = self
            .client
            .post(format!("{}/jsonrpc", self.credentials.url.as_ref()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.credentials.db.as_ref(),
                        self.uid,
                        self.credentials.password.as_ref(),
                        self.env.0.as_ref(),
                        "create",
                        data,
                    ],
                },
            }))
            .send()
            .await?
            .json::<Response<u64>>()
            .await
            .context("create failed")?;

        self.records = resp.result.into();

        Ok(self)
    }

    pub async fn write(&mut self, data: Value) -> BuilderResult<S, E> {
        let resp = self
            .client
            .post(format!("{}/jsonrpc", self.credentials.url.as_ref()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.credentials.db.as_ref(),
                        self.uid,
                        self.credentials.password.as_ref(),
                        self.env.0.as_ref(),
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
        // if let Some(error) = resp.as_object().unwrap().get("error") {
        //     return Err(error)
        // }
        debug!(?resp);

        Ok(self)
    }

    pub async fn search(&mut self, domain: Value) -> BuilderResult<S, E> {
        let resp = self
            .client
            .post(format!("{}/jsonrpc", self.credentials.url.as_ref()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.credentials.db.as_ref(),
                        self.uid,
                        self.credentials.password.as_ref(),
                        self.env.0.as_ref(),
                        "search",
                        domain,
                    ],
                },
            }))
            .send()
            .await?
            .json::<Response<u64>>()
            .await?;

        self.records = resp.result.into();
        // .get("result").context("Failed to get read result")?
        // .as_array().context("Failed to interpret result as array")?
        // .to_vec();
        Ok(self)
    }

    pub async fn read(&mut self, fields: Value) -> ValueResult {
        let resp = self
            .client
            .post(format!("{}/jsonrpc", self.credentials.url.as_ref()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.credentials.db.as_ref(),
                        self.uid,
                        self.credentials.password.as_ref(),
                        self.env.0.as_ref(),
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
        let resp = self
            .client
            .post(format!("{}/jsonrpc", self.credentials.url.as_ref()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.credentials.db.as_ref(),
                        self.uid,
                        self.credentials.password.as_ref(),
                        self.env.0.as_ref(),
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

    pub async fn get<T: DeserializeOwned>(&mut self, field: impl AsRef<str>) -> Result<T> {
        let resp = self
            .client
            .post(format!("{}/jsonrpc", self.credentials.url.as_ref()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.credentials.db.as_ref(),
                        self.uid,
                        self.credentials.password.as_ref(),
                        self.env.0.as_ref(),
                        "read",
                        self.records,
                        [field.as_ref()],
                    ],
                },
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        let resp = resp
            .get("result")
            .context("Failed to get read result")?
            .as_array()
            .context("Failed to interpret read result")?
            .iter()
            .next()
            .context("Failed to find any records")?
            .get(field.as_ref())
            .context("Read field not included in result")?
            .clone();
        Ok(serde_json::from_value::<T>(resp)?)
    }

    pub async fn unlink(&mut self) -> BuilderResult<S, E> {
        let _ = self
            .client
            .post(format!("{}/jsonrpc", self.credentials.url.as_ref()))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "service": "object",
                    "method": "execute",
                    "args": [
                        self.credentials.db.as_ref(),
                        self.uid,
                        self.credentials.password.as_ref(),
                        self.env.0.as_ref(),
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
}
