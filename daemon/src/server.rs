use crate::config::DaemonConfig;
use crate::tunnel::Tunnel;
use cablescout_api::daemon as daemon_api;
use log::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

pub struct Server {
    port: u16,
    daemon_config: Arc<DaemonConfig>,
    tunnel: RwLock<Option<Tunnel>>,
}

impl Server {
    pub fn new(port: u16, daemon_config: Arc<DaemonConfig>) -> Self {
        Self {
            port,
            daemon_config,
            tunnel: Default::default(),
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), self.port);
        tonic::transport::Server::builder()
            .add_service(daemon_api::daemon_server::DaemonServer::new(self))
            .serve(addr)
            .await?;
        Ok(())
    }
}

#[tonic::async_trait]
impl daemon_api::daemon_server::Daemon for Server {
    async fn get_status(
        &self,
        _req: Request<daemon_api::StatusRequest>,
    ) -> Result<Response<daemon_api::StatusResponse>, Status> {
        info!("Handling get_status");
        let tunnel = self.tunnel.read().await;
        Ok(Response::new(daemon_api::StatusResponse {
            config: self.daemon_config.get_tunnels_info().await,
            status: tunnel.as_ref().map(|tunnel| daemon_api::DaemonStatus {
                current_tunnel: tunnel.name(),
                status: tunnel.status().into(),
            }),
        }))
    }

    async fn start_connect_tunnel(
        &self,
        req: Request<daemon_api::StartConnectTunnelRequest>,
    ) -> Result<Response<daemon_api::StartConnectTunnelResponse>, Status> {
        info!("Handling start_connect_tunnel");
        let req = req.into_inner();
        let mut writer = self.tunnel.write().await;
        if writer.is_some() {
            return Err(Status::failed_precondition("Already connected"));
        }

        let tunnel_config = match self.daemon_config.find(&req.name).await {
            None => return Err(Status::not_found("Unknown tunnel")),
            Some(tunnel_config) => tunnel_config,
        };

        let finish_url = tunnel_config
            .finish_url()
            .map_err(|e| Status::internal(e.to_string()))?
            .to_string();

        let mut tunnel = Tunnel::new(req.name, self.daemon_config.clone(), tunnel_config);
        match tunnel.start_connect().await {
            Ok(auth_url) => {
                *writer = Some(tunnel);
                Ok(Response::new(daemon_api::StartConnectTunnelResponse {
                    auth_url: auth_url.to_string(),
                    finish_url,
                }))
            }
            Err(err) => Err(Status::internal(err.to_string())),
        }
    }

    async fn finish_connect_tunnel(
        &self,
        req: Request<daemon_api::FinishConnectTunnelRequest>,
    ) -> Result<Response<daemon_api::FinishConnectTunnelResponse>, Status> {
        info!("Handling finish_connect_tunnel");
        let req = req.into_inner();
        let mut writer = self.tunnel.write().await;

        match writer.as_mut() {
            None => Err(Status::failed_precondition(
                "No tunnel is currently connecting",
            )),
            Some(tunnel) => match tunnel.finish_connect(req.auth_code).await {
                Ok(()) => Ok(Response::new(daemon_api::FinishConnectTunnelResponse {})),
                Err(err) => Err(Status::internal(err.to_string())),
            },
        }
    }

    async fn disconnect_tunnel(
        &self,
        _req: Request<daemon_api::DisconnectTunnelRequest>,
    ) -> Result<Response<daemon_api::DisconnectTunnelResponse>, Status> {
        info!("Handling disconnect_tunnel");
        let mut writer = self.tunnel.write().await;
        match writer.take() {
            None => Err(Status::failed_precondition("Not connected")),
            Some(mut tunnel) => match tunnel.disconnect().await {
                Ok(()) => Ok(Response::new(daemon_api::DisconnectTunnelResponse {})),
                Err(err) => Err(Status::internal(err.to_string())),
            },
        }
    }
}
