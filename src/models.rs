use serde::{Deserialize, Serialize};

#[derive(Deserialize,Serialize)]
pub struct Resp<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>
}

#[derive(Serialize)]
pub struct ResultResponse {
    pub success: bool
}

#[derive(Deserialize, Debug,Default,Serialize,Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub mysql: String,
    pub redis: String,
    pub jwt_secret: String
}

impl AppConfig {
    pub fn new(yml_data: &str) -> Self {
        let config = match serde_yaml::from_str(yml_data) {
            Ok(e) => e,
            Err(e) => panic!("{}",e)
        };
        config
    }
}

#[derive(Deserialize, Debug,Default,Serialize,Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: i32,
}


#[derive(Deserialize,Serialize,Debug,Clone)]
pub struct Login {
    pub phone: String,
    pub code: u32
}

#[derive(Deserialize,Serialize,Debug,Clone)]
#[derive(sqlx::FromRow)]
pub struct Person {
    pub id: u32,
    pub name: String,
    pub age: String,
    pub phone: String
}

#[derive(Deserialize,Serialize,Debug,Clone)]
pub struct PersonForm {
    pub name: Option<String>,
    pub age: Option<u32>
}

#[derive(Deserialize,Serialize,Debug,Clone)]
pub struct LoginForm {
    pub phone: String,
    pub code: u16
}

// pub enum ResponseC