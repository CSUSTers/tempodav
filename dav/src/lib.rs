use std::net::{Ipv6Addr, SocketAddr};

use anyhow::{Context, Result};
use axum::{
    extract::{State, TypedHeader},
    headers::Authorization,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use dav_server::{fakels::FakeLs, localfs::LocalFs};
use tower::service_fn;
use utils::WithProcedure;

pub mod utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DavConfig {
    bind: SocketAddr,
    root: Option<String>,
    http_path: Option<String>,
    user: Option<String>,
    password: Option<String>,
}

impl Default for DavConfig {
    fn default() -> Self {
        DavConfig {
            bind: SocketAddr::from((Ipv6Addr::UNSPECIFIED, 8080)),
            root: None,
            http_path: None,
            user: None,
            password: None,
        }
    }
}

impl DavConfig {
    pub fn bind(mut self, bind: SocketAddr) -> Self {
        self.bind = bind;
        self
    }

    pub fn root(mut self, root: String) -> Self {
        self.root = Some(root);
        self
    }

    pub fn http_path(mut self, path: String) -> Self {
        self.http_path = Some(path);
        self
    }

    pub fn authorization(mut self, user: String, password: String) -> Self {
        self.user = Some(user);
        self.password = Some(password);
        self
    }

    pub fn no_authorization(mut self) -> Self {
        self.user = None;
        self.password = None;
        self
    }
}

impl DavConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> DavServer {
        DavServer::new(self)
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(root) = &self.root {
            let path = std::path::PathBuf::try_from(root).context("invaliable path")?;
            if !path.exists() {
                return Err(anyhow::anyhow!("root path not exists"));
            }
            if !path.is_dir() {
                return Err(anyhow::anyhow!("root path is not a directory"));
            }
        } else {
            return Err(anyhow::anyhow!("root path not set"));
        }

        if let Some(path) = &self.http_path {
            if !path.starts_with('/') || !path.ends_with('/') {
                return Err(anyhow::anyhow!("http path must start and end with /"));
            }
        }

        match (&self.user, &self.password) {
            (Some(_), Some(_)) | (None, None) => {}
            _ => return Err(anyhow::anyhow!("user and password must be both set or not")),
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DavServer {
    config: DavConfig,
}

impl DavServer {
    pub fn new(config: DavConfig) -> Self {
        DavServer { config }
    }

    pub fn builder() -> DavConfig {
        DavConfig::new()
    }

    pub async fn run(&self) -> Result<()> {
        self.config.validate()?;

        let path_prefix = self.config.http_path.as_deref().unwrap_or("/");
        let account = match (&self.config.user, &self.config.password) {
            (Some(user), Some(password)) => Some(Account::new(user.clone(), password.clone())),
            _ => None,
        };

        let dav_service_handler = dav_server::DavHandler::builder()
            .strip_prefix(path_prefix)
            .filesystem(LocalFs::new(
                self.config.root.clone().unwrap(),
                false,
                false,
                false,
            ))
            .locksystem(FakeLs::new())
            .build_handler();
        let dav_router = axum::Router::new()
            .route_service(
                path_prefix,
                service_fn(move |req| {
                    let dav_service_handler = dav_service_handler.clone();
                    async move { Ok(dav_service_handler.handle(req).await) }
                }),
            )
            .with(|r| match account {
                None => r,
                Some(account) => r.route_layer(axum::middleware::from_fn_with_state(
                    account,
                    http_basic_authorize_middleware,
                )),
            });

        axum::Server::bind(&self.config.bind)
            .serve(dav_router.into_make_service())
            .await
            .context("failed to start dav server")
    }
}

#[derive(Debug, Clone)]
struct Account {
    user: String,
    password: String,
}

impl Account {
    fn new(user: String, password: String) -> Self {
        Account { user, password }
    }
}

async fn http_basic_authorize_middleware<B>(
    State(account): State<Account>,
    TypedHeader(auth): TypedHeader<Authorization<axum::headers::authorization::Basic>>,
    req: Request<B>,
    next: Next<B>,
) -> Response {
    let req_user = auth.username();
    let req_password = auth.password();

    let Account { user, password } = account;
    if user == req_user && password == req_password {
        next.run(req).await
    } else {
        (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
    }
}

#[cfg(test)]
mod tests {
    #[ignore]
    #[tokio::test]
    async fn test_dav_server() {
        use super::*;
        println!("pwd: {:?}", std::env::current_dir().unwrap());
        let server = DavServer::builder()
            .root("../public".to_string())
            .bind("0.0.0.0:8080".parse().unwrap())
            .build();
        server.run().await.unwrap();
    }
}
