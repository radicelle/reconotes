use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let profile = env::var("PROFILE").unwrap();
    
    // Only copy for release builds
    if profile == "release" {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let exe_name = "recognotes-rust-backend.exe";
        
        let source = Path::new(&manifest_dir)
            .join("target")
            .join("release")
            .join(exe_name);
        
        let dest = Path::new(&manifest_dir).join(exe_name);
        
        // Copy the executable to the root directory
        if source.exists() {
            match fs::copy(&source, &dest) {
                Ok(bytes) => {
                    #[allow(clippy::cast_precision_loss)]
                    let size_mb = bytes as f64 / (1024.0 * 1024.0);
                    println!("cargo:warning=✅ Copied {exe_name} to root folder ({size_mb:.2} MB)");
                    
                    // Show OS-compatible run command
                    let run_cmd = if cfg!(target_os = "windows") {
                        format!(".\\{exe_name}")
                    } else if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
                        format!("./{exe_name}")
                    } else {
                        exe_name.to_string()
                    };
                    
                    println!("cargo:warning=📌 To run the executable:  {run_cmd}");
                }
                Err(e) => {
                    println!("cargo:warning=❌ Failed to copy executable: {e}");
                }
            }
        }
    }
}
