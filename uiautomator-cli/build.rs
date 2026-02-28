use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/");

    let assets_dir = Path::new("assets");

    // 检查并计算 atx-agent 的 MD5
    let atx_agent_path = assets_dir.join("atx-agent");
    if atx_agent_path.exists() {
        let atx_agent = fs::read(&atx_agent_path).expect("无法读取 atx-agent 文件");
        let atx_agent_md5 = format!("{:x}", md5::compute(&atx_agent));
        println!("cargo:rustc-env=ATX_AGENT_MD5={}", atx_agent_md5);
        println!("cargo:rustc-env=ATX_AGENT_SIZE={}", atx_agent.len());
    } else {
        eprintln!("警告: atx-agent 文件不存在");
        eprintln!("请运行以下命令下载资源文件:");
        eprintln!("  cd assets && ./download_atx_agent.sh");
        eprintln!("或者:");
        eprintln!("  cd assets && .\\download_atx_agent.ps1");
        panic!("atx-agent 文件不存在，请先下载资源文件");
    }

    // 检查并计算 app-uiautomator.apk 的 MD5
    let app_apk_path = assets_dir.join("app-uiautomator.apk");
    if app_apk_path.exists() {
        let app_apk = fs::read(&app_apk_path).expect("无法读取 app-uiautomator.apk 文件");
        let app_apk_md5 = format!("{:x}", md5::compute(&app_apk));
        println!("cargo:rustc-env=APP_UIAUTOMATOR_APK_MD5={}", app_apk_md5);
        println!("cargo:rustc-env=APP_UIAUTOMATOR_APK_SIZE={}", app_apk.len());
    } else {
        eprintln!("警告: app-uiautomator.apk 文件不存在");
        panic!("app-uiautomator.apk 文件不存在，请先复制资源文件");
    }

    // 检查并计算 app-uiautomator-test.apk 的 MD5
    let test_apk_path = assets_dir.join("app-uiautomator-test.apk");
    if test_apk_path.exists() {
        let test_apk = fs::read(&test_apk_path).expect("无法读取 app-uiautomator-test.apk 文件");
        let test_apk_md5 = format!("{:x}", md5::compute(&test_apk));
        println!(
            "cargo:rustc-env=APP_UIAUTOMATOR_TEST_APK_MD5={}",
            test_apk_md5
        );
        println!(
            "cargo:rustc-env=APP_UIAUTOMATOR_TEST_APK_SIZE={}",
            test_apk.len()
        );
    } else {
        eprintln!("警告: app-uiautomator-test.apk 文件不存在");
        panic!("app-uiautomator-test.apk 文件不存在，请先下载资源文件");
    }

    println!("✓ 所有资源文件检查完成");
}
