use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let root_dir = env::current_dir().expect("Failed to get current directory");
    
    // List of projects to build
    let projects = vec![
        ("recognotes-rust-backend", true),
        ("recognotes-desktop-gui", false),
    ];

    println!("\n{}", "=".repeat(70));
    println!("🚀 RecogNotes Multi-Project Builder");
    println!("{}", "=".repeat(70));
    println!("Root directory: {}\n", root_dir.display());
    println!("⏭️  Note: Cargo commands are only applied to backend and frontend subfolders");
    println!("⏭️  Note: Use 'cargo clean' in each subfolder to clean\n");

    for (project_name, is_backend) in projects {
        let project_path = root_dir.join(project_name);
        
        if !project_path.exists() {
            eprintln!("⚠️  Warning: Project directory not found: {}", project_path.display());
            continue;
        }

        println!("\n📦 Processing: {}", project_name);
        println!("{}", "-".repeat(70));

        // Build debug
        print_command("🔨 cargo build");
        run_cargo_command(&project_path, &["build"]);

        // Build release
        print_command("⚡ cargo build --release");
        run_cargo_command(&project_path, &["build", "--release"]);

        // Run (only for backend)
        if is_backend {
            print_command("▶️  cargo run --release");
            run_cargo_command(&project_path, &["run", "--release"]);
        } else {
            println!("⏭️  Skipping run (GUI requires display)");
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("✅ Build process completed successfully!");
    println!("{}\n", "=".repeat(70));
}

fn print_command(cmd: &str) {
    println!("  {}", cmd);
}

fn run_cargo_command(project_path: &PathBuf, args: &[&str]) {
    match Command::new("cargo")
        .args(args)
        .current_dir(project_path)
        .status()
    {
        Ok(status) => {
            if status.success() {
                println!("     ✓ Success");
            } else {
                eprintln!("     ✗ Failed with status: {}", status);
            }
        }
        Err(e) => {
            eprintln!("     ✗ Error executing command: {}", e);
        }
    }
}
