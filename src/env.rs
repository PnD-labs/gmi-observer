pub fn get_env(key: &str) -> String {
    println!("{:?}", key);
    std::env::var(key).unwrap()
}

#[derive(Debug, Clone)]
pub struct DBEnv {
    pub db_url: String,
    pub username: String,
    pub password: String,
    pub name_space: String,
    pub db_name: String,
    pub db_client_id: String,
    pub db_client_password: String,
}

impl DBEnv {
    pub fn new() -> Self {
        DBEnv {
            db_url: get_env("DB_URL"),
            username: get_env("DB_USERNAME"),
            password: get_env("DB_PASSWORD"),
            name_space: get_env("DB_NAMESPACE"),
            db_name: get_env("DB_NAME"),
            db_client_id: get_env("DB_CLIENT_ID"),
            db_client_password: get_env("DB_CLIENT_PASSWORD"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AMMEnv {
    pub amm_package_id: String,
    pub amm_config_id: String,
}

impl AMMEnv {
    pub fn new() -> Self {
        AMMEnv {
            amm_package_id: get_env("AMM_PACKAGE_ID"),
            amm_config_id: get_env("AMM_CONFIG_ID"),
        }
    }
}
