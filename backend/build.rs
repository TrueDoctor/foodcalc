use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=src/frontend");

    let command = Command::new("npm")
        .args([
            "exec",
            "tailwindcss",
            "--",
            "-c",
            "tailwind.config.js",
            "-i",
            "assets/styles/index.css",
            "-o",
            "src/index.css",
            "--minify",
        ])
        .output();

    match command {
        Ok(output) if output.status.success() => (),
        Ok(output) => {
            println!(
                "cargo:warning=Failed to run tailwindcss, styles will not be updated. stderr:"
            );
            let error = String::from_utf8_lossy(&output.stderr);
            for line in error.lines() {
                println!("cargo:warning={}", line);
            }
        }
        Err(error) => {
            println!(
                "cargo:warning=Failed to run tailwindcss, styles will not be updated. Reason: {}",
                error.to_string().replace('\n', " ")
            );
        }
    }
}
