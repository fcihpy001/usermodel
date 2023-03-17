use std::fs::read_to_string;
use crate::models::AppConfig;

pub fn get_config() -> AppConfig {
    AppConfig::new(
        read_to_string("application.yml")
            .unwrap()
            .as_ref())
}

pub fn get_jwt_secret() -> String {
    get_config().jwt_secret
}