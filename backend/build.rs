use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=assets");

    std::fs::remove_dir_all("build").unwrap_or_default();

    if Command::new("npx")
        .args([
            "tailwindcss",
            "-c",
            "tailwind.config.js",
            "-i",
            "assets/styles/index.css",
            "content",
            "src/frontend/",
            "-o",
            "src/index.css",
            "--minify",
        ])
        .status()
        .is_err()
    {
        eprintln!("Failed to run tailwindcss, styles will not be updated");
    }
}
