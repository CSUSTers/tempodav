// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::{IpAddr, Ipv6Addr};

use dav::{utils::WithMutProcedure, TlsConfig};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

mod cert;
use cert::*;
use tauri::Manager;
use window_vibrancy::NSVisualEffectMaterial;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    ip: Option<String>,
    port: Option<u16>,
    root: Option<String>,
    auth: Option<(String, String)>,

    enable_tls: bool,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
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
            if ip.parse::<IpAddr>().is_err() {
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

        // if self.enable_tls {
        //     if let Some(cert) = &self.tls_cert {
        //         match cert {
        //             TlsCert::Cert { cert, key } => {
        //                 if cert.is_empty() {
        //                     return Err("TLS certificate is not set".to_string());
        //                 }
        //                 if key.is_empty() {
        //                     return Err("TLS key is not set".to_string());
        //                 }
        //             }
        //             TlsCert::CertFile { cert, key } => {
        //                 if !std::path::Path::new(&cert).exists() {
        //                     return Err("TLS certificate file does not exist".to_string());
        //                 };
        //                 if !std::path::Path::new(&key).exists() {
        //                     return Err("TLS key file does not exist".to_string());
        //                 };
        //             }
        //         }
        //     } else {
        //         return Err("TLS certificate is not set".to_string());
        //     }
        // }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
enum TlsCert {
    #[serde(skip_serializing)]
    Cert {
        cert: Vec<u8>,
        key: Vec<u8>,
    },
    CertFile {
        cert: String,
        key: String,
    },
}

impl TlsCert {
    fn use_app_default_path() -> Self {
        let (cert_path, key_path) = cert_key_path().expect("Failed to get cert/key path");
        TlsCert::CertFile {
            cert: cert_path,
            key: key_path,
        }
    }

    fn check_cert_available(&self) -> Result<(), String> {
        match self {
            TlsCert::Cert { cert, key } => {
                if cert.is_empty() || key.is_empty() {
                    return Err("TLS certificate or key is not set".to_string());
                }
                // TODO: check cert validity
                Ok(())
            }
            TlsCert::CertFile { cert, key } => {
                let cert_path = std::path::Path::new(cert);
                let key_path = std::path::Path::new(key);
                if !cert_path.exists() || !key_path.exists() {
                    return Err("TLS certificate or key file does not exist".to_string());
                };
                // TODO: check cert validity
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
struct ServerHandler {
    handle: tokio::task::JoinHandle<()>,
}

#[derive(Debug)]
struct State {
    tokio_runtime: Mutex<Option<tokio::runtime::Runtime>>,
    config: Mutex<Config>,
    server_handler: Mutex<Option<ServerHandler>>,
}

#[tauri::command]
fn get_config(state: tauri::State<State>) -> Config {
    state.config.lock().clone().with_mut(|c| {
        if c.enable_tls && c.tls_cert.is_none() {
            cert_key_path()
                .and_then(|(cert_path, key_path)| {
                    c.tls_cert = Some(TlsCert::CertFile {
                        cert: cert_path,
                        key: key_path,
                    });
                    Ok(())
                })
                .or_else(|e| {
                    eprintln!("Failed to get cert/key path: {}", e);
                    Err(())
                })
                .unwrap_or(());
        }
    })
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
    _state: tauri::State<State>,
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
    println!("config: {:?}", config);
    let bind_ip: IpAddr = config
        .ip
        .map(|s| {
            s.parse::<IpAddr>()
                .map_err(|e| format!("Invaliable Ip Address: {}", e))
        })
        .unwrap_or(Ok(Ipv6Addr::UNSPECIFIED.into()))?;
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
        match &config
            .tls_cert
            .unwrap_or_else(TlsCert::use_app_default_path)
        {
            TlsCert::Cert { cert, key } => {
                dav_server = dav_server.tls(TlsConfig::pem(cert.clone(), key.clone()));
            }
            TlsCert::CertFile { cert, key } => {
                let cert_bytes = std::fs::read(cert).map_err(|e| e.to_string())?;
                let key_bytes = std::fs::read(key).map_err(|e| e.to_string())?;
                dav_server = dav_server.tls(TlsConfig::pem(cert_bytes, key_bytes));
            }
        }
    }

    println!("dav_server: {:?}", dav_server);
    let dav_server = dav_server.build();

    let mut rt_guard = state.tokio_runtime.lock();
    let rt = rt_guard.get_or_insert({
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime");
        rt
    });

    // let rt = tokio::runtime::Builder::new_multi_thread()
    //     .enable_all()
    //     .build()
    //     .expect("Failed to build tokio runtime");

    // let _ = rt.enter();

    let handle = rt.spawn(async move {
        if let Err(err) = dav_server.run().await {
            println!("dav server error: {}", err);
        }
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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
enum DavServerStatus {
    Running,
    Stopped,
}

#[tauri::command]
fn check_dav_server(state: tauri::State<State>) -> Result<DavServerStatus, String> {
    let handler_guard = state.server_handler.lock();
    match handler_guard.as_ref() {
        Some(hander) => {
            if hander.handle.is_finished() {
                println!("dav server is stopped");
                Ok(DavServerStatus::Stopped)
            } else {
                Ok(DavServerStatus::Running)
            }
        }
        None => Ok(DavServerStatus::Stopped),
    }
}

fn main() {
    let config = Config::default();
    let state = State {
        tokio_runtime: Mutex::new(None),
        config: Mutex::new(config),
        server_handler: Mutex::new(None),
    };
    tauri::Builder::default()
        .setup(|app| {
            let main = app.get_window("main").expect("main window not found");

            #[cfg(all(target_os = "macos"))]
            {
                use window_vibrancy::apply_vibrancy;
                apply_vibrancy(&main, NSVisualEffectMaterial::HudWindow, None, None)
                    .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");
            }

            #[cfg(target_os = "windows")]
            {
                use window_vibrancy::apply_mica;
                let r = apply_mica(&main, None);
                if let Err(err) = r {
                    eprintln!("Failed to apply mica: {:?}", err);
                };
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            import_tls_or_cert_from_path,
            start_dav_server,
            stop_dav_server,
            check_dav_server,
        ])
        .manage(state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
