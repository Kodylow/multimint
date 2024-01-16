use fedimint_client::derivable_secret::DerivableSecret;
use fedimint_client::secret::{PlainRootSecretStrategy, RootSecretStrategy};
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
    pub fm_db_path: PathBuf,
    pub invite_code: InviteCode,
    pub host: String,
    pub port: u16,
    pub password: String,
    pub root_secret: DerivableSecret,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv::dotenv().ok();

        let fm_db_path = env::var("FM_DB_PATH").expect("FM_DB_PATH must be set");
        let fm_db_path = PathBuf::from_str(&fm_db_path).expect("Invalid fm db path");

        let invite_code = InviteCode::from_str(&env::var("INVITE_CODE").expect("INVITE_CODE must be set")).expect("Invalid invite code");

        let host = env::var("HOST").expect("HOST must be set");

        let port =
            u16::from_str(&env::var("PORT").unwrap_or("3000".to_string())).expect("Invalid port");

        let password = env::var("PASSWORD").expect("PASSWORD must be set");

        let root_secret = create_root_secret(env::var("SECRET").expect("SECRET must be set"));

        info!("Loaded config");

        Ok(Self {
            fm_db_path,
            invite_code,
            host,
            port,
            password,
            root_secret,
        })
    }
}

pub fn create_root_secret(secret: String) -> DerivableSecret {
    let secret_bytes = secret.as_bytes();
    assert_eq!(secret_bytes.len(), 64, "SECRET must be 64 bytes long");
    let mut secret_array = [0; 64];
    secret_array.copy_from_slice(secret_bytes);
    PlainRootSecretStrategy::to_root_secret(&secret_array)
}
