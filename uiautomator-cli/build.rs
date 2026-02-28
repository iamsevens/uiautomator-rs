use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/");

    let assets_dir = Path::new("assets");
    let docs_rs = env::var_os("DOCS_RS").is_some();

    emit_asset_md5_and_size(
        &assets_dir.join("atx-agent"),
        "ATX_AGENT_MD5",
        "ATX_AGENT_SIZE",
        docs_rs,
        "atx-agent file is missing. Download assets first:\n  cd assets && ./download_atx_agent.sh\nor\n  cd assets && .\\download_atx_agent.ps1",
    );
    emit_asset_md5_and_size(
        &assets_dir.join("app-uiautomator.apk"),
        "APP_UIAUTOMATOR_APK_MD5",
        "APP_UIAUTOMATOR_APK_SIZE",
        docs_rs,
        "app-uiautomator.apk file is missing. Copy/download assets first.",
    );
    emit_asset_md5_and_size(
        &assets_dir.join("app-uiautomator-test.apk"),
        "APP_UIAUTOMATOR_TEST_APK_MD5",
        "APP_UIAUTOMATOR_TEST_APK_SIZE",
        docs_rs,
        "app-uiautomator-test.apk file is missing. Download assets first.",
    );
}

fn emit_asset_md5_and_size(
    path: &Path,
    md5_var: &str,
    size_var: &str,
    docs_rs: bool,
    missing_message: &str,
) {
    if path.exists() {
        let bytes = fs::read(path).unwrap_or_else(|_| panic!("failed to read {}", path.display()));
        let md5_hash = format!("{:x}", md5::compute(&bytes));
        println!("cargo:rustc-env={md5_var}={md5_hash}");
        println!("cargo:rustc-env={size_var}={}", bytes.len());
    } else if docs_rs {
        println!("cargo:rustc-env={md5_var}=placeholder");
        println!("cargo:rustc-env={size_var}=0");
        println!(
            "cargo:warning=missing {} in docs.rs build, using placeholder",
            path.display()
        );
    } else {
        panic!("{missing_message}");
    }

    println!("cargo:rerun-if-changed={}", path.display());
}
