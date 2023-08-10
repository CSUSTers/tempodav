pub fn cert_key_path() -> Result<(String, String), String> {
    let config_dir = tauri::api::path::config_dir().ok_or("Failed to get config dir")?;

    let cert_path = config_dir.join("cert.pem").to_str().map(String::from);
    let key_path = config_dir.join("key.pem").to_str().map(String::from);

    match (cert_path, key_path) {
        (Some(cert_path), Some(key_path)) => Ok((cert_path, key_path)),
        _ => Err("Failed to get cert and key path".to_string()),
    }
}
