// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use dav::{
    utils::{WithMutProcedure, WithProcedure},
    TlsConfig,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

mod cert;
use cert::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    ip: Option<String>,
    port: Option<u16>,
    root: Option<String>,
    auth: Option<(String, String)>,

    enable_tls: bool,
    #[serde(skip)]
    tls_cert: Option<TlsCert>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ip: None,
            port: None,
            root: None,
            auth: None,

            enable_tls: false,
            tls_cert: None,
        }
    }
}

impl Config {
    fn update_tls_cert(&mut self, cert: TlsCert) {
        self.tls_cert = Some(cert);
    }

    fn validate(&self) -> Result<(), String> {
        if let Some(ip) = &self.ip {
            if ip.parse::<SocketAddr>().is_err() {
                return Err("Invalid IP address".to_string());
            }
        }

        match &self.root {
            Some(root) => {
                if !std::path::PathBuf::from(root).exists() {
                    return Err("Root path does not exist".to_string());
                }
            }
            None => {
                return Err("Root path is not set".to_string());
            }
        }

        if self.enable_tls {
            if let Some(cert) = &self.tls_cert {
                match cert {
                    TlsCert::Cert { cert, key } => {
                        if cert.is_empty() {
                            return Err("TLS certificate is not set".to_string());
                        }
                        if key.is_empty() {
                            return Err("TLS key is not set".to_string());
                        }
                    }
                    TlsCert::CertFile { cert, key } => {
                        if !std::path::Path::new(&cert).exists() {
                            return Err("TLS certificate file does not exist".to_string());
                        };
                        if !std::path::Path::new(&key).exists() {
                            return Err("TLS key file does not exist".to_string());
                        };
                    }
                }
            } else {
                return Err("TLS certificate is not set".to_string());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
enum TlsCert {
    Cert { cert: Vec<u8>, key: Vec<u8> },
    CertFile { cert: String, key: String },
}

#[derive(Debug)]
struct ServerHandler {
    handle: tokio::task::JoinHandle<()>,
}

#[derive(Debug)]
struct State {
    config: Mutex<Config>,
    server_handler: Mutex<Option<ServerHandler>>,
}

#[tauri::command]
fn update_config(state: tauri::State<State>, mut config: Config) -> Result<(), String> {
    config.validate()?;

    let mut config_guard = state.config.lock();
    config.tls_cert = config_guard.tls_cert.take();
    *config_guard = config;
    Ok(())
}

#[tauri::command]
fn import_tls_or_cert_from_path(
    state: tauri::State<State>,
    cert_path: Option<String>,
    key_path: Option<String>,
) -> Result<(), String> {
    let c = cert_path.map(|p| std::path::PathBuf::from(&p));
    let k = key_path.map(|p| std::path::PathBuf::from(&p));

    if matches!(c, Some(ref c) if !c.exists()) {
        return Err("Certificate file does not exist".to_string());
    }
    if matches!(k, Some(ref k) if !k.exists()) {
        return Err("Key file does not exist".to_string());
    }

    let (stored_cert_path, stored_key_path) = cert_key_path()?;
    if let Some(c) = c {
        std::fs::copy(c, &stored_cert_path).map_err(|e| e.to_string())?;
    }
    if let Some(k) = k {
        std::fs::copy(k, &stored_key_path).map_err(|e| e.to_string())?;
    }

    // let mut config_guard = state.config.lock();
    // config_guard.update_tls_cert(TlsCert::CertFile {
    //     cert: stored_cert_path,
    //     key: stored_key_path,
    // });
    Ok(())
}

#[tauri::command]
fn start_dav_server(state: tauri::State<State>) -> Result<(), String> {
    let mut handler_guard = state.server_handler.lock();
    if let Some(handler) = handler_guard.take() {
        handler.handle.abort();
    }

    let config = state.config.lock().clone();
    let bind_ip: IpAddr = config
        .ip
        .map(|s| s.parse().expect("Invalid IP address"))
        .unwrap_or(Ipv6Addr::UNSPECIFIED.into());
    let bind_port = config
        .port
        .unwrap_or_else(|| if config.enable_tls { 443 } else { 80 });
    let bind = (bind_ip, bind_port).into();

    let mut dav_server = dav::DavServer::builder()
        .bind(bind)
        .root(config.root.clone().expect("Root path is not set"));
    if let Some((user, password)) = &config.auth {
        dav_server = dav_server.authorization(user.clone(), password.clone());
    }
    if config.enable_tls {
        match &config.tls_cert {
            Some(TlsCert::Cert { cert, key }) => {
                dav_server = dav_server.tls(TlsConfig::pem(cert.clone(), key.clone()));
            }
            Some(TlsCert::CertFile { cert, key }) => {
                let cert_bytes = std::fs::read(cert).map_err(|e| e.to_string())?;
                let key_bytes = std::fs::read(key).map_err(|e| e.to_string())?;
                dav_server = dav_server.tls(TlsConfig::pem(cert_bytes, key_bytes));
            }
            None => return Err("TLS certificate is not set".to_string()),
        }
    }

    let dav_server = dav_server.build();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");
    let handle = rt.spawn(async move {
        dav_server.run().await.expect("Failed to run dav server");
    });
    handler_guard.replace(ServerHandler { handle });

    Ok(())
}

#[tauri::command]
fn stop_dav_server(state: tauri::State<State>) -> Result<(), String> {
    let mut handler_guard = state.server_handler.lock();
    if let Some(handler) = handler_guard.take() {
        handler.handle.abort();
    }
    Ok(())
}

fn main() {
    let config = Config::default();
    let state = State {
        config: Mutex::new(config),
        server_handler: Mutex::new(None),
    };
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            update_config,
            import_tls_or_cert_from_path,
            start_dav_server,
            stop_dav_server,
        ])
        .manage(state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
