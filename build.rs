use std::process::Command;

/**
 * Stamp a version into the binary. Release builds pass the git tag through
 * `PICKUP_VERSION` (or a `VERSION` file, which survives cross's container
 * boundary); local builds on a tagged commit fall back to the exact git tag.
 * Everything else reports the crate version.
 */
fn main() {
    println!("cargo:rerun-if-env-changed=PICKUP_VERSION");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_TYPE");
    println!("cargo:rerun-if-changed=VERSION");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/tags");

    let version = std::env::var("PICKUP_VERSION")
        .ok()
        .and_then(non_empty)
        .or_else(version_file)
        .or_else(github_tag)
        .or_else(git_exact_tag)
        .unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".into()));

    let version = version.trim();
    let version = version.strip_prefix('v').unwrap_or(version);
    println!("cargo:rustc-env=PICKUP_VERSION={version}");
}

fn non_empty(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn version_file() -> Option<String> {
    std::fs::read_to_string("VERSION").ok().and_then(non_empty)
}

fn github_tag() -> Option<String> {
    if std::env::var("GITHUB_REF_TYPE").ok()? != "tag" {
        return None;
    }
    std::env::var("GITHUB_REF_NAME").ok().and_then(non_empty)
}

fn git_exact_tag() -> Option<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--exact-match", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok().and_then(non_empty)
}
