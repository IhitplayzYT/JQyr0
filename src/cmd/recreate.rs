use base64::Engine;
use serde::Deserialize;
use std::{collections::HashSet, error::Error, fs, path::PathBuf};
use crate::config::{config_path, endpoint_url, Config, validate_relative};

#[derive(Deserialize)]
struct PulledFile { path: PathBuf, content: String }

pub fn recreate_main(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let config = Config::load(&config_path(&args))?;
    let root = &config.default.root;
    fs::create_dir_all(root)?;
    for file in &config.default.raw_files {
        validate_relative(&file.path)?;
        let dest = root.join(&file.path);
        if let Some(parent) = dest.parent() { fs::create_dir_all(parent)?; }
        fs::write(dest, base64::engine::general_purpose::STANDARD.decode(&file.content)?)?;
    }
    if config.default.files.is_empty() { println!("Recreated embedded files in {}", root.display()); return Ok(()); }
    let paths: Vec<String> = config.default.files.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    let url = endpoint_url(&config.default.src, &config.default.pull_endpoint);
    let response = reqwest::blocking::Client::new().post(url).json(&paths).send()?.error_for_status()?;
    let pulled: Vec<PulledFile> = response.json()?;
    let expected: HashSet<_> = config.default.files.iter().cloned().collect();
    let mut seen = HashSet::new();
    for file in pulled {
        validate_relative(&file.path)?;
        if !expected.contains(&file.path) { return Err(format!("Server returned unrequested path {}", file.path.display()).into()); }
        if !seen.insert(file.path.clone()) { return Err(format!("Recieved duplicate path: {}", file.path.display()).into()); }
        let destination = root.join(file.path);
        if let Some(parent) = destination.parent() { fs::create_dir_all(parent)?; }
        fs::write(destination, base64::engine::general_purpose::STANDARD.decode(file.content)?)?;
    }
    if seen.len() != expected.len() { return Err("Recieved and Requested length do not match".into()); }
    println!("Recreated {} files in {}", seen.len() + config.default.raw_files.len(), root.display());
    Ok(())
}
