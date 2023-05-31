use std::{collections::BTreeMap, fmt::Display};

use xmlrpc::{Request, Value, Error};

#[derive(Clone)]
pub struct Client {
    db: String,
    password: String,
    uid: Value,
    url: String,
    env: String,
    records: Vec<Value>,
}

impl Client {
    pub fn new(
        db: &str,
        username: &str,
        password: &str,
        url: &str,
    ) -> Result<Self, Error> {
        let uid = Request::new("authenticate")
            .arg(db)
            .arg(username)
            .arg(password)
            .arg(Value::Nil)
            .call_url(format!("{}/xmlrpc/2/common", url))?;

        Ok(Self {
            db: db.to_string(),
            password: password.to_string(),
            uid,
            url: format!("{}/xmlrpc/2/object", url),
            env: "res.users".to_string(),
            records: vec![],
        })
    }

    pub fn env(&mut self, env: &str) -> &mut Self {
        self.env = env.to_string();
        self
    }

    pub fn browse(&mut self, id: i32) -> &mut Self {
        self.records = vec![Value::from(id)];
        self
    }

    pub fn create(&mut self, data: Value) -> Result<&mut Self, Error> {
        let url = self.url.clone();
        let resp = self.execute("create")
            .arg(Value::Array(vec![data; 1]))
            .call_url(url)?;
        self.records = vec![resp; 1];
        Ok(self)
    }

    pub fn create2(&mut self, data: serde_xmlrpc::Value) -> Result<&mut Self, Error> {
        let url = self.url.clone();
        let client = reqwest::blocking::Client::new();
        let body = serde_xmlrpc::request_to_string("create", vec![
            serde_xmlrpc::Value::from(self.db.clone()),
            serde_xmlrpc::Value::from(self.uid.as_str().unwrap()),
            serde_xmlrpc::Value::from(self.password.clone()),
            serde_xmlrpc::Value::from(self.env.clone()),
            data,
        ]).unwrap();
        let response = client
            .post(url)
            .body(body)
            .send()
            .unwrap()
            .text()
            .unwrap();
        println!("{response:?}");
        Ok(self)
    }

    pub fn write(&mut self, data: Value) -> Result<&mut Self, Error> {
        let url = self.url.clone();
        let records = self.records.clone();
        let _will_fail = self.execute("write")
            .arg(Value::Array(vec![
                Value::Array(records),
                data,
            ]))
            .call_url(url);
        Ok(self)
    }

    pub fn search(&mut self) {}

    pub fn read(&mut self) {}

    pub fn get(&mut self, field: &str) -> Result<Value, Error> {
        let data = BTreeMap::from([
            ("fields".to_string(), Value::Array(vec![Value::from(field); 1])),
        ]);
        let url = self.url.clone();
        let records = self.records.clone();

        let resp = self.execute("read")
            .arg(Value::Array(records))
            .arg(Value::Struct(data))
            .call_url(url)?[0]
            .get(field)
            .unwrap()
            .to_owned();

        Ok(resp)
    }

    fn execute(&mut self, method: &str) -> Request {
        Request::new("execute_kw")
            .arg(self.db.clone())
            .arg(self.uid.clone())
            .arg(self.password.clone())
            .arg(self.env.clone())
            .arg(method)
    }
}

impl Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.env,
            self.records
                .iter()
                .map(|c| format!("{},", c.as_i32().unwrap()))
                .collect::<String>()
        )
    }
}

pub struct Data(pub Value);

impl<const N: usize> From<[(String, Value); N]> for Data {
    fn from(mut arr: [(String, Value); N]) -> Self {
        arr.sort_by(|a, b| a.0.cmp(&b.0));
        Data(Value::Struct(BTreeMap::from(arr)))
    }
}
