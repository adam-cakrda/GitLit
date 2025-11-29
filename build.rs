use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let scss_dir = Path::new("assets/scss");
    let out_dir = Path::new("public");

    println!("cargo:rerun-if-changed=assets/scss");

    if !scss_dir.exists() {
        eprintln!("build.rs: assets/scss not found, skipping SCSS compilation");
        return;
    }

    if !out_dir.exists() {
        if let Err(e) = fs::create_dir_all(out_dir) {
            panic!("Failed to create public dir: {}", e);
        }
    }

    let entries: Vec<PathBuf> = fs::read_dir(scss_dir)
        .expect("Failed to read assets/scss directory")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension().and_then(|s| s.to_str()) == Some("scss")
                && !p.file_name().and_then(|s| s.to_str()).map(|n| n.starts_with('_')).unwrap_or(false)
        })
        .collect();

    for scss_file in entries {
        let file_stem = scss_file
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("Invalid SCSS file name");

        let out_file = out_dir.join(format!("{}.css", file_stem));

        match grass::from_path(&scss_file, &grass::Options::default().style(grass::OutputStyle::Compressed)) {
            Ok(css) => {
                if let Err(e) = fs::write(&out_file, css) {
                    panic!("Failed to write CSS to {}: {}", out_file.display(), e);
                } else {
                    println!("cargo:warning=Compiled {} -> {}", scss_file.display(), out_file.display());
                }
            }
            Err(err) => {
                panic!("SASS compilation failed for {}: {}", scss_file.display(), err);
            }
        }
    }
}
