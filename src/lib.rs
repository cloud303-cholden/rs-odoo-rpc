use std::fmt::Display;

use reqwest::IntoUrl;
use serde::{
    Deserialize,
    Serialize,
};
use serde_xmlrpc::{
    request_to_string,
    response_from_str,
    to_value,
    Value,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error(transparent)]
    XmlRpcError(#[from] serde_xmlrpc::Error),
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
    #[error("Unknown error")]
    Unknown,
}

struct Request<M: AsRef<str>> {
    method: M,
    args: Vec<Value>,
}

impl<M: AsRef<str>> Request<M> {
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

    fn send<'a, T, U>(self, url: U) -> Result<T, RequestError>
    where
        T: Deserialize<'a>,
        U: IntoUrl,
    {
        let body = request_to_string(self.method.as_ref(), self.args)?;
        let resp = reqwest::blocking::Client::new() 
            .post(url)
            .body(body)
            .send()?
            .text()?;
        dbg!(&resp);
        Ok(response_from_str::<T>(resp)?)
    }
}

#[derive(Clone)]
pub struct Client<T>
where
    T: AsRef<str> + Display + Clone
{
    db: T,
    password: T,
    uid: i32,
    url: String,
    env: T,
    records: Vec<Value>,
}

impl<T> Client<T>
where
    Value: From<T>,
    T: AsRef<str> + Display + Clone,
{
    pub fn new(
        db: T,
        username: T,
        password: T,
        env: T,
        url: String,
    ) -> Result<Self, RequestError> {
        let uid = Request::new("authenticate")
            .arg(db.clone())
            .arg(username)
            .arg(password.clone())
            .arg(Value::Nil)
            .send::<i32, _>(format!("{}/xmlrpc/2/common", url))?;

        Ok(Self {
            db,
            password,
            uid,
            url: format!("{}/xmlrpc/2/object", url),
            env,
            records: vec![],
        })
    }

    pub fn env(mut self, env: T) -> Self {
        self.env = env;
        self
    }

    pub fn browse(&mut self, id: i32) -> &mut Self {
        self.records = vec![Value::Int(id); 1];
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

    pub fn get<'a, F>(&mut self, field: &str) -> Result<F, RequestError>
    where
        F: Deserialize<'a>,
    {
        let resp = self.execute("read")
            .arg(Value::Array(self.records.clone()))
            .arg(
                to_value(Get::new(field.to_string()))?
            )
            .send::<Vec<F>, _>(self.url.clone())?;
        resp.into_iter().next().ok_or(RequestError::Unknown)
    }

    fn execute(&self, method: &str) -> Request<&str> {
        Request::new("execute_kw")
            .arg(self.db.clone())
            .arg(self.uid)
            .arg(self.password.clone())
            .arg(self.env.clone())
            .arg(method)
    }
}

#[derive(Serialize)]
struct Get {
    fields: Vec<String>,
}

impl Get {
    fn new(field: String) -> Self {
        Self {
            fields: vec![field; 1],
        }
    }
}

impl<T: AsRef<str> + Display + Clone> Display for Client<T> {
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
