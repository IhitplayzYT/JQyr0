use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::{Path, PathBuf}};
use base64::Engine;
pub const CONFIG_FILE: &str = "JQyr0.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default: Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub root: PathBuf,
    pub src: String,
    pub pull_endpoint: String,
    pub check_update_endpoint: String,
    #[serde(default)]
    pub files: Vec<PathBuf>,
    #[serde(default)]
    pub raw_files: Vec<RawFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFile {
    pub path: PathBuf,
    pub content: String,
}

impl Config {
    /// Load JQyr.toml from a path
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let raw = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&raw)?;
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        self.validate()?;
        fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        let s = &self.default;
        if s.src.trim().is_empty() || s.pull_endpoint.trim().is_empty() || s.check_update_endpoint.trim().is_empty() { return Err("src, pull, and check_update endpoint cant be empty".into()); }
        if !(s.src.starts_with("http://") || s.src.starts_with("https://")) { return Err("Src must be an http:// or https:// URL".into()); }
        for path in s.files.iter().chain(s.raw_files.iter().map(|f| &f.path)) { validate_relative(path)?;}
        for raw in &s.raw_files { base64::engine::general_purpose::STANDARD.decode(&raw.content)?; }
        Ok(())
    }
}

pub fn validate_relative(path: &Path) -> Result<(), Box<dyn Error>> {
    if path.as_os_str().is_empty() || path.is_absolute() || path.components().any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::RootDir | std::path::Component::Prefix(_))) { return Err(format!("Fpath must be a non empty rel path without '..': {}", path.display()).into());}
    Ok(())
}

pub fn normalize_bundle_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    if paths.iter().any(|p| p.is_absolute()) && paths.iter().any(|p| !p.is_absolute()) { return Err("Bundle paths must either all be abs or rel only".into()); }
    if paths.is_empty() || !paths[0].is_absolute() {
        for path in paths { validate_relative(path)?; }
        return Ok(paths.to_vec());
    }

    // Get common ancestor of all filepath
    let mut common: Vec<std::ffi::OsString> = paths[0].parent().unwrap_or(Path::new("/")).components().map(|c| c.as_os_str().to_owned()).collect();
    for path in paths.iter().skip(1) {
        let parent: Vec<_> = path.parent().unwrap_or(Path::new("/")).components().map(|c| c.as_os_str().to_owned()).collect();
        let shared = common.iter().zip(parent.iter()).take_while(|(a, b)| a == b).count();
        common.truncate(shared);
    }

    let common: PathBuf = common.iter().collect();
    let bundle_dir = common.file_name().ok_or("Can't derive bundle dir from the paths")?;
    let mut result = Vec::with_capacity(paths.len());
    // Ensuire fpaths releative uncommon path is from the common parent path
    for path in paths {
        let tail = path.strip_prefix(&common)?;
        let mut dst= PathBuf::from(bundle_dir);
        dst.push(tail);
        validate_relative(&dst)?;
        result.push(dst);
    }
    Ok(result)
}

pub fn normalize_added_path(path: &Path, existing: &[PathBuf]) -> Result<PathBuf, Box<dyn Error>> {
    if !path.is_absolute() { validate_relative(path)?; return Ok(path.to_path_buf()); }
    if let Some(anchor) = existing.first().and_then(|p| p.components().next()) {
        let components: Vec<_> = path.components().collect();
        if let Some(index) = components.iter().rposition(|c| c.as_os_str() == anchor.as_os_str()) {
            let mut destination = PathBuf::new();
            for component in &components[index..] { destination.push(component.as_os_str()); }
            validate_relative(&destination)?;
            return Ok(destination);
        }
    }
    Ok(normalize_bundle_paths(&[path.to_path_buf()])?.remove(0))
}

pub fn config_path(args: &[String]) -> PathBuf { args.iter().find_map(|arg| arg.strip_prefix("--config=").or_else(|| arg.strip_prefix("-c=")).map(PathBuf::from)).unwrap_or_else(|| PathBuf::from(CONFIG_FILE)) }

pub fn endpoint_url(src: &str, endpoint: &str) -> String {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") { endpoint.to_owned() }
    else { format!("{}/{}", src.trim_end_matches('/'), endpoint.trim_start_matches('/')) }
}

