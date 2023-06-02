use std::fmt::Display;

use serde_json::{Value, json};

pub struct Client {
    db: String,
    password: String,
    uid: u64,
    url: String,
    env: String,
    records: Vec<Value>,
}

impl Client {
    pub async fn new(
        db: String,
        username: String,
        password: String,
        env: String,
        url: String,
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
            .send()
            .await?
            .json::<Value>()
            .await?
            .get("result")
            .ok_or("Failed to get login result")?
            .as_u64()
            .ok_or("Failed to interpret user ID")?;

        dbg!(&uid);

        Ok(Self {
            db,
            password,
            uid: 2,
            env,
            url: format!("{}/jsonrpc", url),
            records: vec![],
        })
    }

    pub fn env(mut self, env: String) -> Self {
        self.env = env;
        self
    }

    pub fn browse(&mut self, id: u64) -> &mut Self {
        self.records = vec![serde_json::to_value(id).unwrap(); 1];
        self
    }

    pub fn create(&mut self) {}

    pub fn write(&mut self) {}

    pub fn search(&mut self) {}

    pub fn read(&mut self) {}

    pub async fn get(&mut self, field: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let resp = reqwest::Client::new()
            .post(self.url.clone())
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
            .get("result")
            .ok_or("Failed to get read result")?
            .as_array()
            .ok_or("Failed to interpret read result")?
            .iter()
            .next()
            .ok_or("Failed to find any records")?
            .get(field)
            .ok_or("Read field not included in result")?
            .clone();
        dbg!(&resp);
        Ok(resp)
    }

    fn execute(&self, method: &str) -> () {}
    // fn execute(&self, method: &str) -> Request<&str> {
    //     Request::new("execute_kw")
    //         .arg(self.db.clone())
    //         .arg(self.uid)
    //         .arg(self.password.clone())
    //         .arg(self.env.clone())
    //         .arg(method)
    // }
}

impl Display for Client {
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
