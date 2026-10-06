use std::fs;
use std::process::Command;

fn run_command(cmd: &str, args: &[&str]) -> Result<(), String> {
    println!("> {} {}", cmd, args.join(" "));
    let status = Command::new(cmd)
        .args(args)
        .status()
        .map_err(|e| format!("Failed to execute '{}': {}", cmd, e))?;

    if !status.success() {
        return Err(format!("Command '{} {}' failed with status: {:?}", cmd, args.join(" "), status));
    }
    Ok(())
}

fn get_output(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute '{}': {}", cmd, e))?;

    if !output.status.success() {
        return Err(format!("Command '{}' returned non-zero error: {}", cmd, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn bump_version(current: &str, bump_type: &str) -> Result<String, String> {
    let parts: Vec<&str> = current.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("Invalid semver format: {}", current));
    }

    let mut major: u64 = parts[0].parse().map_err(|_| "Invalid major version")?;
    let mut minor: u64 = parts[1].parse().map_err(|_| "Invalid minor version")?;
    let mut patch: u64 = parts[2].parse().map_err(|_| "Invalid patch version")?;

    match bump_type {
        "major" => {
            major += 1;
            minor = 0;
            patch = 0;
        }
        "minor" => {
            minor += 1;
            patch = 0;
        }
        "patch" | "" => {
            patch += 1;
        }
        explicit_ver if explicit_ver.contains('.') => {
            return Ok(explicit_ver.trim_start_matches('v').to_string());
        }
        other => {
            return Err(format!("Unknown bump type or version '{}'. Use 'patch', 'minor', 'major', or 'X.Y.Z'.", other));
        }
    }

    Ok(format!("{}.{}.{}", major, minor, patch))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SearchForge CI/CD Release Publisher ===");

    // 1. Read Cargo.toml
    let cargo_toml_path = "Cargo.toml";
    let cargo_str = fs::read_to_string(cargo_toml_path)?;

    let mut current_version = String::new();
    for line in cargo_str.lines() {
        if line.trim().starts_with("version =") {
            let ver = line.split('=').nth(1).unwrap().trim().trim_matches('"');
            current_version = ver.to_string();
            break;
        }
    }

    if current_version.is_empty() {
        return Err("Could not find 'version' in Cargo.toml".into());
    }

    // 2. Determine new version
    let arg = std::env::args().nth(1).unwrap_or_else(|| "patch".to_string());
    let next_version = bump_version(&current_version, &arg)?;
    let tag_name = format!("v{}", next_version);

    println!("Current version : {}", current_version);
    println!("New version     : {}", next_version);
    println!("Release Tag     : {}", tag_name);

    // 3. Update Cargo.toml
    let old_needle = format!("version = \"{}\"", current_version);
    let new_replacement = format!("version = \"{}\"", next_version);
    let updated_cargo = cargo_str.replacen(&old_needle, &new_replacement, 1);
    fs::write(cargo_toml_path, updated_cargo)?;
    println!("Updated {} successfully.", cargo_toml_path);

    // 4. Verify project builds or checks cleanly
    println!("\n[1/5] Verifying project health (cargo check)...");
    run_command("cargo", &["check", "--release"])
        .map_err(|e| format!("cargo check failed: {}", e))?;

    // 5. Stage files for git
    println!("\n[2/5] Staging files for Git...");
    run_command("git", &["add", "."])?;

    // 6. Check if git status has changes
    let status_output = get_output("git", &["status", "--porcelain"])?;
    if !status_output.is_empty() {
        let commit_msg = format!("Release {}", tag_name);
        println!("\n[3/5] Committing changes: '{}'...", commit_msg);
        run_command("git", &["commit", "-m", &commit_msg])?;
    } else {
        println!("\n[3/5] No uncommitted changes found.");
    }

    // 7. Create Git Tag
    println!("\n[4/5] Creating annotated git tag: {}...", tag_name);
    let tag_message = format!("SearchForge Release {}", tag_name);
    // Delete local tag if it already exists to avoid conflict
    let _ = run_command("git", &["tag", "-d", &tag_name]);
    run_command("git", &["tag", "-a", &tag_name, "-m", &tag_message])?;

    // 8. Push commit and tag to GitHub
    println!("\n[5/5] Pushing to remote (origin master & tags)...");
    run_command("git", &["push", "origin", "master"])?;
    run_command("git", &["push", "origin", &tag_name, "--force"])?;

    println!("\n=======================================================");
    println!(" Successfully pushed release tag: {}", tag_name);
    println!(" GitHub Actions release workflow triggered!");
    println!(" Multi-platform binaries (Windows, Linux, macOS) will be built & uploaded automatically.");
    println!("=======================================================");

    Ok(())
}
