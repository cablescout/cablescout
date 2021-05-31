use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn http_client() -> Result<reqwest::Client> {
    Ok(reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()?)
}

pub async fn http_post<Req, Res>(url: Url, req: Req) -> Result<Res>
where
    Req: Serialize,
    Res: DeserializeOwned,
{
    Ok(http_client()?
        .post(url)
        .json(&req)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}
