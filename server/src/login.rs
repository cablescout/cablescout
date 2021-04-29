use super::api_result::LoginError;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
pub struct LoginSettings {
    /// Google client ID (register at https://console.cloud.google.com)
    #[structopt(long, env = "GOOGLE_CLIENT_ID")]
    pub google_client_id: String,

    /// Google client secret
    #[structopt(long, env = "GOOGLE_CLIENT_SECRET")]
    google_client_secret: String,

    /// Login duration, sets how long it might take between when a user
    /// starts the login process and until the moment they post their credentials
    /// back into the server for getting connection information.
    #[structopt(long, env = "LOGIN_DURATION", default_value = "1m")]
    pub login_duration: humantime::Duration,
}

pub struct UserData {}

pub async fn validate_user(
    settings: &LoginSettings,
    id_token: &str,
) -> Result<UserData, LoginError> {
    Err(LoginError::InvalidIdToken)
}
