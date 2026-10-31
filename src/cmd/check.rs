use sha2::{Digest, Sha256};
use std::{error::Error, fs};
use crate::config::{config_path, endpoint_url, Config};
use base64::Engine;

pub fn check_main(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let path = config_path(&args);
    let config = Config::load(&path)?;
    let checksum = checksum(&config)?;
    let settings = &config.default;
    let endpoint = settings.check_update_endpoint.replace("{checksum}", &checksum).replace("{CHECKSUM}", &checksum);
    let url = endpoint_url(&settings.src, &endpoint);
    let response = reqwest::blocking::Client::new().get(url).send()?.error_for_status()?;
    let body = response.text()?;
    let available = match serde_json::from_str::<bool>(&body) {
        Ok(value) => value,
        Err(_) if body.trim().eq_ignore_ascii_case("true") => true,
        Err(_) if body.trim().eq_ignore_ascii_case("false") => false,
        Err(_) => return Err("Check endpoint must respond with JSON True/False".into()),
    };
    println!("{}", if available { "update available" } else { "up to date" });
    Ok(())
}

pub fn checksum(config: &Config) -> Result<String, Box<dyn Error>> {
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    let root = &config.default.root;
    for path in &config.default.files {
        let full = root.join(path);
        let bytes = fs::read(&full).map_err(|e| format!("cannot hash {}: {e}", full.display()))?;
        entries.push((path.to_string_lossy().into_owned(), bytes));
    }
    for file in &config.default.raw_files { entries.push((file.path.to_string_lossy().into_owned(), base64::engine::general_purpose::STANDARD.decode(&file.content)?)); }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hash = Sha256::new();
    for (path, bytes) in entries {
        hash.update((path.len() as u64).to_be_bytes());
        hash.update(path.as_bytes());
        hash.update((bytes.len() as u64).to_be_bytes());
        hash.update(bytes);
    }
    Ok(format!("{:x}", hash.finalize()))
}
