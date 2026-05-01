use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const LIMIT: usize = 250;

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=build.rs");
    let mut over: Vec<(PathBuf, usize)> = Vec::new();
    walk(Path::new("src"), &mut over)?;
    if over.is_empty() {
        return Ok(());
    }
    for (path, n) in &over {
        println!("cargo::error={} has {} lines (limit {})", path.display(), n, LIMIT);
    }
    std::process::exit(1);
}

fn walk(dir: &Path, over: &mut Vec<(PathBuf, usize)>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, over)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let count = fs::read_to_string(&path)?.lines().count();
            if count > LIMIT {
                over.push((path, count));
            }
        }
    }
    Ok(())
}
