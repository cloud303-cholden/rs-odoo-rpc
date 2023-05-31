use std::{collections::BTreeMap, fmt::Display};

use xmlrpc::{Request, Value, Error};

pub struct Proxy {
    db: String,
    password: String,
    uid: Value,
    url: String,
    recordset: Recordset,
}

impl Proxy {
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
            recordset: Recordset::default(),
        })
    }

    pub fn env(mut self, model: &str) -> Self {
        self.recordset = Recordset {
            model: model.to_string(),
            ..Default::default()
        };
        self
    }

    pub fn browse(&mut self, id: i32) -> &mut Self {
        self.recordset.ids = vec![Value::from(id)];
        self
    }

    pub fn get(&self, field: &str) -> Result<Value, Error> {
        let mut data = BTreeMap::new();
        data.insert("fields".to_string(), Value::Array(vec![Value::from(field)]));
        let resp = Request::new("execute_kw")
            .arg(self.db.clone())
            .arg(self.uid.clone())
            .arg(self.password.clone())
            .arg(self.recordset.model.clone())
            .arg("read")
            .arg(Value::Array(self.recordset.ids.clone()))
            .arg(Value::Struct(data))
            .call_url(self.url.clone())?[0]
            .get(field)
            .unwrap()
            .to_owned();

        Ok(resp)
    }
}

pub struct Recordset {
    pub model: String,
    pub ids: Vec<Value>,
}

impl Default for Recordset {
    fn default() -> Self {
        Self {
            model: "res.users".to_string(),
            ids: vec![],
        }
    }
}

impl Display for Proxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.recordset.model,
            self.recordset
                .ids
                .iter()
                .map(|c| format!("{},", c.as_i32().unwrap()))
                .collect::<String>()
        )
    }
}
