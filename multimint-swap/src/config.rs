use fedimint_core::api::InviteCode;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

lazy_static::lazy_static! {
    pub static ref CONFIG: Config =
        Config::from_env().expect("Failed to load config from environment");
}

pub struct Config {
    pub data_dir: PathBuf,
    pub invite_code: InviteCode,
    pub host: String,
    pub port: u16,
    pub password: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv::dotenv().ok();

        let data_dir = env::var("DATA_DIR").expect("DATA_DIR must be set");
        let data_dir = PathBuf::from_str(&data_dir).expect("Invalid fm db path");

        let invite_code = InviteCode::from_str(&env::var("INVITE_CODE").expect("INVITE_CODE must be set")).expect("Invalid invite code");

        let host = env::var("HOST").expect("HOST must be set");

        let port =
            u16::from_str(&env::var("PORT").unwrap_or("3000".to_string())).expect("Invalid port");

        let password = env::var("PASSWORD").expect("PASSWORD must be set");

        info!("Loaded config");

        Ok(Self {
            data_dir,
            invite_code,
            host,
            port,
            password,
        })
    }
}
