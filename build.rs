use std::env;
use std::fs;
use std::path::Path;
#[allow(unused_imports)]
use std::process::Command;
use fs_extra::dir::{copy, CopyOptions};

fn main() {
    let vite_build_dir = Path::new("front/dist"); // マウントポイント (ジャンクション)
    let resolved_path = vite_build_dir
        .canonicalize() // 実際のパスに解決
        .expect("Failed to resolve Vite build directory");

    let target_dir = if env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()) == "release" {
        Path::new("target/release/front/dist")
    } else {
        Path::new("target/debug/front/dist")
    };

    println!("target dir: {}", target_dir.display());
    println!("vite build dir: {}", resolved_path.display());

    if resolved_path.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true; // 上書きを許可
        options.copy_inside = true; // 中身のみコピー

        // ① 必要ならターゲットディレクトリを作成
        fs::create_dir_all(&target_dir).expect("Failed to create target directory");

        // ② resolved_path の中身をコピー
        for entry in resolved_path
            .read_dir()
            .expect("Failed to read resolved path directory")
        {
            if let Ok(entry) = entry {
                let from = entry.path();
                if from.is_dir() {
                    copy(&from, &target_dir, &options).expect("Failed to copy directory content.");
                } else {
                    fs::copy(&from, &target_dir.join(entry.file_name()))
                        .expect("Failed to copy file content.");
                }
            }
        }
    } else {
        eprintln!("Vite build directory `{}` does not exist.", resolved_path.display());
        std::process::exit(1);
    }

    println!("cargo:rerun-if-changed=front/dist");
}