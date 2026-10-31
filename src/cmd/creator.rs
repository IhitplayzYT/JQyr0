use base64::Engine;
use std::{error::Error, fs, path::PathBuf};
use crate::config::{normalize_bundle_paths, Config, RawFile, Settings, CONFIG_FILE};

pub fn init_main(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut root = PathBuf::from(".");
    let mut src = String::from("http://127.0.0.1:8000");
    let mut pull_endpoint = String::from("/pull");
    let mut check_update_endpoint = String::from("/check/{checksum}");
    let mut source_files = Vec::new();
    let mut raw_contents = Vec::new();
    for arg in &args {
        if let Some(v) = value(arg, &["--working-dir=", "-wd="]) { root = PathBuf::from(v); }
        else if let Some(v) = value(arg, &["--source=", "-src="]) { src = v.to_owned(); }
        else if let Some(v) = value(arg, &["--pull-endpoint=", "-pe="]) { pull_endpoint = v.to_owned(); }
        else if let Some(v) = value(arg, &["--check-update-endpoint=", "--checksum-endpoint=", "-ce="]) { check_update_endpoint = v.to_owned(); }
        else if let Some(v) = value(arg, &["--file=", "-f="]) { source_files.push(PathBuf::from(v)); }
        else if let Some(v) = value(arg, &["--raw-file=", "-rf="]) {
            let path = PathBuf::from(v);
            raw_contents.push(base64::engine::general_purpose::STANDARD.encode(fs::read(&path)?));
            source_files.push(path);
        }
    }
    let dests = normalize_bundle_paths(&source_files)?;
    let mut files = Vec::new();
    let mut raw_files = Vec::new();
    let mut raw_content = raw_contents.into_iter();

    for (source, destination) in source_files.into_iter().zip(dests) {
        if value_from_args_raw(&args, &source) { raw_files.push(RawFile { path: destination, content: raw_content.next().ok_or("raw file content mapping failed")? });}
        else { files.push(destination); }
    }
    let config = Config { default: Settings { root, src, pull_endpoint, check_update_endpoint, files, raw_files } };
    let out = PathBuf::from(CONFIG_FILE);
    config.save(&out)?;
    println!("Wrote {}", out.display());
    Ok(())
}

fn value<'a>(arg: &'a str, prefixes: &[&str]) -> Option<&'a str> { prefixes.iter().find_map(|p| arg.strip_prefix(p)) }

// Find any raw files and return true if any
fn value_from_args_raw(args: &[String], path: &PathBuf) -> bool { args.iter().any(|arg| value(arg, &["--raw-file=", "-rf="]).map(PathBuf::from).as_ref() == Some(path)) }
