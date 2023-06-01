use std::{collections::BTreeMap, fmt::Display};

use serde::{Serialize, Deserialize};
use reqwest::IntoUrl;
use serde_xmlrpc::{request_to_string, response_from_str, Value};
use thiserror::Error;

#[derive(Error, Debug)]
enum RequestError {
    #[error(transparent)]
    XmlRpcError(#[from] serde_xmlrpc::Error),
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
}

struct Request<M: Into<String>> {
    method: M,
    args: Vec<Value>,
}

impl<M: Into<String>> Request<M> {
    fn new(method: M) -> Self {
        Self {
            method,
            args: vec![],
        }
    }

    fn arg<V: Into<Value>>(mut self, arg: V) -> Self {
        self.args.push(arg.into());
        self
    }

    fn send<'a, U, T>(&self, url: U) -> Result<T, RequestError>
    where
        U: IntoUrl,
        T: Deserialize<'a>
    {
        let body = request_to_string(&self.method.into(), self.args).unwrap();
        let text = reqwest::blocking::Client::new() 
            .post(url)
            .body(body)
            .send()?
            .text()?;
        let response = response_from_str::<T>(&text)?;
        Ok(response)
    }
}

#[derive(Clone)]
pub struct Client<T: Into<String>> {
    db: T,
    password: T,
    uid: i32,
    url: T,
    env: T,
    records: Vec<Value>,
}

impl<'a, T: Into<String> + From<&'a str>> Client<T> {
    pub fn new(
        db: T,
        username: T,
        password: T,
        url: T,
    ) -> Result<Self, RequestError> {
        let uid = Request::new("authenticate")
            .arg(db)
            .arg(username)
            .arg(password)
            .arg(Value::Nil)
            .send(format!("{}/xmlrpc/2/common", url))?;

        Ok(Self {
            db,
            password,
            uid,
            url: format!("{}/xmlrpc/2/object", url),
            env: "res.users".into(),
            records: vec![],
        })
    }

    pub fn env(mut self, env: T) -> Self {
        self.env = env;
        self
    }

    pub fn browse(mut self, id: i32) -> Self {
        self.records = vec![Value::from(id)];
        self
    }

    // pub fn create(&mut self, data: impl Serialize) -> Result<&mut Self, reqwest::Error> {
    //     let url = self.url.clone();
    //     let client = reqwest::blocking::Client::new();
    //     let body = serde_xmlrpc::request_to_string("execute_kw", vec![
    //         serde_xmlrpc::Value::from(self.db.clone()),
    //         serde_xmlrpc::Value::from(self.uid.as_i32().unwrap()),
    //         serde_xmlrpc::Value::from(self.password.clone()),
    //         serde_xmlrpc::Value::from(self.env.clone()),
    //         serde_xmlrpc::Value::from("create"),
    //         serde_xmlrpc::Value::Array(vec![serde_xmlrpc::to_value(data).unwrap(); 1]),
    //     ]).unwrap();
    //     println!("{body:?}");
    //     let response = client
    //         .post(url)
    //         .body(body)
    //         .send()?
    //         .text()?;
    //
    //     let id = serde_xmlrpc::response_from_str::<i32>(&response).unwrap();
    //     println!("{id:?}");
    //     self.records = vec![Value::from(id); 1];
    //     Ok(self)
    // }

    // pub fn write(&mut self, data: Value) -> Result<&mut Self, RequestError> {
    //     let url = self.url.clone();
    //     let records = self.records.clone();
    //     let _will_fail = self.execute("write")
    //         .arg(Value::Array(vec![
    //             Value::Array(records),
    //             data,
    //         ]))
    //         .call_url(url);
    //     Ok(self)
    // }

    // pub fn search(&mut self) {}

    // pub fn read(&mut self) {}

    // pub fn get(&mut self, field: &str) -> Result<Value, RequestError> {
    //     let data = BTreeMap::from([
    //         ("fields".to_string(), Value::Array(vec![Value::from(field); 1])),
    //     ]);
    //     let url = self.url.clone();
    //     let records = self.records.clone();
    //
    //     let resp = self.execute("read")
    //         .arg(Value::Array(records))
    //         .arg(Value::Struct(data))
    //         .call_url(url)?[0]
    //         .get(field)
    //         .unwrap()
    //         .to_owned();
    //
    //     Ok(resp)
    // }

    fn execute(&self, method: &str) -> Request<&str> {
        Request::new("execute_kw")
            .arg(self.db)
            .arg(self.uid)
            .arg(self.password)
            .arg(self.env)
            .arg(method)
        // Request {
        //     method: "execute_kw".into(),
        //     args: vec![
        //         Value::from(self.db.clone()),
        //         Value::from(self.uid.as_i32().unwrap()),
        //         Value::from(self.password.clone()),
        //         Value::from(self.env.clone()),
        //         Value::from(method),
        //     ],
        // }
    }
}

impl<T: Into<String>> Display for Client<T> {
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
