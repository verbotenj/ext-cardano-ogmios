use lazy_static::lazy_static;
use std::env;

lazy_static! {
    static ref CONTROLLER_CONFIG: Config = Config::from_env();
}

pub fn get_config() -> &'static Config {
    &CONTROLLER_CONFIG
}

#[derive(Debug, Clone)]
pub struct Config {
    pub dns_zone: String,
    pub ingress_class: String,
    pub api_key_salt: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            dns_zone: env::var("DNS_ZONE").unwrap_or("demeter.run".into()),
            ingress_class: env::var("INGRESS_CLASS").unwrap_or("ogmios-v1".into()),
            api_key_salt: env::var("API_KEY_SALT").unwrap_or("ogmios-salt".into()),
        }
    }
}
