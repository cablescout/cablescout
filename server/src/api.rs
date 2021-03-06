use crate::api_result::ApiResult;
use crate::login::{LoginSettings, OidcLogin};
use crate::tokens::{random_string, TokenGenerator};
use crate::wireguard::Wireguard;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use anyhow::Result;
use cablescout_api::server::{
    FinishLoginRequest, FinishLoginResponse, StartLoginRequest, StartLoginResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::IpAddr;
use std::sync::Arc;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
pub struct ApiSettings {
    /// API server bind address, use default value to listen on all interfaces
    #[structopt(long, env = "HTTP_BIND_IP", default_value = "0.0.0.0")]
    http_bind_ip: IpAddr,

    /// API server port
    #[structopt(long, env = "HTTP_PORT", default_value = "8080")]
    http_port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginData {
    device_id: Uuid,
    client_public_key: String,
    nonce: String,
}

#[actix_web::get("/finish")]
async fn finish_page() -> ApiResult {
    Ok(HttpResponse::Ok().body(include_str!("pages/finish.html")))
}

fn get_hostname(req: &web::HttpRequest) -> String {
    req.connection_info()
        .host()
        .split(':')
        .next()
        .unwrap()
        .to_owned()
}

#[actix_web::post("/api/v1/login/start")]
async fn start_login_api(
    req: web::HttpRequest,
    api_server: web::Data<Arc<ApiServer>>,
    data: web::Json<StartLoginRequest>,
) -> ApiResult {
    let nonce = random_string::<15>();

    let login_token = api_server
        .token_generator
        .generate(LoginData {
            device_id: data.device_id,
            client_public_key: data.client_public_key.clone(),
            nonce: nonce.clone(),
        })
        .await?;

    let auth_url = api_server
        .oidc_login
        .get_auth_url(&req.connection_info(), &login_token, &nonce)
        .await?;

    Ok(HttpResponse::Ok().json(StartLoginResponse {
        auth_url,
        login_token,
    }))
}

#[actix_web::post("/api/v1/login/finish")]
async fn finish_login_api(
    req: web::HttpRequest,
    api_server: web::Data<Arc<ApiServer>>,
    data: web::Json<FinishLoginRequest>,
) -> ApiResult {
    let login_data: LoginData = api_server
        .token_generator
        .validate(&data.login_token)
        .await?;

    let user_data = api_server
        .oidc_login
        .validate_user(&req.connection_info(), &data.auth_code, &login_data.nonce)
        .await?;
    let hostname = get_hostname(&req);

    let (interface, peer, session_ends_at) = api_server
        .wireguard
        .clone()
        .start_session(
            &hostname,
            login_data.device_id,
            login_data.client_public_key,
            user_data,
        )
        .await?;
    Ok(HttpResponse::Ok().json(FinishLoginResponse {
        session_ends_at,
        interface,
        peer,
    }))
}

pub(crate) struct ApiServer {
    api_settings: ApiSettings,
    oidc_login: OidcLogin,
    wireguard: Arc<Wireguard>,
    token_generator: TokenGenerator,
}

impl ApiServer {
    pub fn new(
        api_settings: ApiSettings,
        login_settings: LoginSettings,
        wireguard: Arc<Wireguard>,
    ) -> Result<Arc<Self>> {
        let token_generator = TokenGenerator::new(chrono::Duration::from_std(
            login_settings.login_duration.into(),
        )?)?;
        let oidc_login = OidcLogin::new(login_settings);
        Ok(Arc::new(Self {
            api_settings,
            oidc_login,
            wireguard,
            token_generator,
        }))
    }

    fn bind_address(&self) -> String {
        format!(
            "{}:{}",
            self.api_settings.http_bind_ip, self.api_settings.http_port
        )
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let bind_address = self.bind_address();

        Ok(HttpServer::new(move || {
            let json_config = web::JsonConfig::default().error_handler(|err, _req| {
                actix_web::error::ErrorBadRequest(json!({
                    "message": err.to_string(),
                }))
            });

            App::new()
                .wrap(Logger::default())
                .app_data(json_config)
                .app_data(self.clone())
                .service(finish_page)
                .service(start_login_api)
                .service(finish_login_api)
        })
        .bind(&bind_address)?
        .run()
        .await?)
    }
}
