use base64::Engine;
use std::{error::Error, fs, path::PathBuf};
use crate::config::{config_path, normalize_added_path, Config, RawFile};

pub fn packer_main(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let path = config_path(&args);
    let mut config = Config::load(&path)?;
    let mut changed = false;
    for arg in &args {
        let item = arg.strip_prefix("--raw-file=").or_else(|| arg.strip_prefix("-rf="));
        if let Some(input) = item {
            let input_path = PathBuf::from(input);
            let bin = fs::read(&input_path)?;
            let exists: Vec<_> = config.default.files.iter().chain(config.default.raw_files.iter().map(|f| &f.path)).cloned().collect();
            let dst = normalize_added_path(&input_path, &exists)?;
            let content = base64::engine::general_purpose::STANDARD.encode(bin);
            config.default.raw_files.retain(|f| f.path != dst);
            config.default.files.retain(|f| f != &dst);
            config.default.raw_files.push(RawFile { path: dst, content });
            changed = true;
        } else if let Some(input) = arg.strip_prefix("--file=").or_else(|| arg.strip_prefix("-f=")) {
            let input_path = PathBuf::from(input);
            let existing: Vec<_> = config.default.files.iter().chain(config.default.raw_files.iter().map(|f| &f.path)).cloned().collect();
            let dst = normalize_added_path(&input_path, &existing)?;
            if !config.default.files.contains(&dst) { config.default.files.push(dst); }
            config.default.raw_files.retain(|f| f.path != PathBuf::from(input));
            changed = true;
        }
    }
    if !changed { return Err("packer requires --file=PATH or --raw-file=PATH".into()); }
    config.save(&path)?;
    println!("Updated {}", path.display());
    Ok(())
}
