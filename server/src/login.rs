use anyhow::{anyhow, Result};
use email_address_parser::EmailAddress;
use openid::DiscoveredClient;
use structopt::StructOpt;
use url::Url;

#[derive(Debug, Clone, StructOpt)]
pub(crate) struct LoginSettings {
    /// OIDC server
    #[structopt(long, env = "OIDC_SERVER")]
    pub oidc_server: Url,

    /// OIDC client ID
    #[structopt(long, env = "OIDC_CLIENT_ID")]
    pub oidc_client_id: String,

    /// OIDC client secret
    #[structopt(long, env = "OIDC_CLIENT_SECRET")]
    pub oidc_client_secret: String,

    /// Email domain, only users with email addresses from this domain can successfully login
    #[structopt(long, env = "EMAIL_DOMAIN")]
    pub email_domain: String,

    /// Login duration, sets how long it might take between when a user
    /// starts the login process and until the moment they post their credentials
    /// back into the server for getting connection information.
    #[structopt(long, env = "LOGIN_DURATION", default_value = "2m")]
    pub login_duration: humantime::Duration,
}

#[derive(Debug, Clone)]
pub struct UserData {
    email: String,
}

pub(crate) struct OidcLogin {
    settings: LoginSettings,
}

impl OidcLogin {
    pub fn new(settings: LoginSettings) -> Self {
        Self { settings }
    }

    async fn client(&self, conn: &actix_web::dev::ConnectionInfo) -> Result<DiscoveredClient> {
        let redirect = Url::parse(&format!("{}://{}/finish", conn.scheme(), conn.host()))?;
        Ok(DiscoveredClient::discover(
            self.settings.oidc_client_id.clone(),
            self.settings.oidc_client_secret.clone(),
            Some(redirect.to_string()),
            self.settings.oidc_server.clone(),
        )
        .await?)
    }

    pub async fn get_auth_url(
        &self,
        conn: &actix_web::dev::ConnectionInfo,
        login_token: &str,
        nonce: &str,
    ) -> Result<Url> {
        let client = self.client(conn).await?;

        let options = openid::Options {
            scope: Some("openid profile email".to_owned()),
            nonce: Some(nonce.to_owned()),
            state: Some(login_token.to_owned()),
            ..Default::default()
        };

        Ok(client.auth_url(&options))
    }

    pub async fn validate_user(
        &self,
        conn: &actix_web::dev::ConnectionInfo,
        auth_code: &str,
        nonce: &str,
    ) -> Result<UserData> {
        let client = self.client(conn).await?;
        let token = client.authenticate(auth_code, Some(nonce), None).await?;
        let userinfo = client.request_userinfo(&token).await?;
        let email = userinfo
            .email
            .ok_or_else(|| anyhow!("Login succeeded but user has no email address"))?;
        let email_address = EmailAddress::parse(&email, None)
            .ok_or_else(|| anyhow!("Login succeeded but could not parse user email address"))?;
        if email_address.get_domain() == self.settings.email_domain {
            Ok(UserData { email })
        } else {
            Err(anyhow!("Wrong email domain: {}", email))
        }
    }
}
