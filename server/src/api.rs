use crate::api_result::ApiResult;
use crate::login::{validate_user, LoginSettings};
use crate::tokens::TokenGenerator;
use crate::wireguard::Wireguard;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::IpAddr;
use std::sync::Arc;
use structopt::StructOpt;

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
    client_public_key: String,
}

#[actix_web::post("/api/v1/login/start")]
async fn start_login(
    api_server: web::Data<Arc<ApiServer>>,
    data: web::Json<cablescout_api::StartLoginRequest>,
) -> ApiResult {
    let login_token = api_server
        .token_generator
        .generate(LoginData {
            client_public_key: data.client_public_key.clone(),
        })
        .await?;
    Ok(HttpResponse::Ok().json(cablescout_api::StartLoginResponse {
        login_token,
        oidc_client_id: api_server.login_settings.google_client_id.clone(),
    }))
}

#[actix_web::post("/api/v1/login/finish")]
async fn finish_login(
    api_server: web::Data<Arc<ApiServer>>,
    data: web::Json<cablescout_api::FinishLoginRequest>,
) -> ApiResult {
    let login_data: LoginData = api_server
        .token_generator
        .validate(&data.login_token)
        .await?;

    let user_data = validate_user(&api_server.login_settings, &data.id_token).await?;

    let (client_config, session_ends_at) = api_server
        .wireguard
        .clone()
        .start_session(login_data.client_public_key, user_data)
        .await?;
    Ok(
        HttpResponse::Ok().json(cablescout_api::FinishLoginResponse {
            session_ends_at,
            client_config,
        }),
    )
}

pub(crate) struct ApiServer {
    api_settings: ApiSettings,
    login_settings: LoginSettings,
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
        Ok(Arc::new(Self {
            api_settings,
            login_settings,
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
                .data(self.clone())
                .service(start_login)
                .service(finish_login)
        })
        .bind(&bind_address)?
        .run()
        .await?)
    }
}
