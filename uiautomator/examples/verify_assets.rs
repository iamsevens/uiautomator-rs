//! 验证资源文件
//!
//! 此示例用于验证所有必需的资源文件是否存在并显示其信息。

use std::path::Path;

fn main() {
    println!("=== 验证 uiautomator 资源文件 ===\n");

    let assets_dir = Path::new("assets");

    // Direct 模式资源
    println!("【Direct 模式资源】");
    check_file(assets_dir.join("u2.jar"), "UiAutomator2 服务 JAR", true);
    check_file(
        assets_dir.join("app-uiautomator.apk"),
        "UiAutomator2 APK",
        true,
    );

    println!();

    // ATX-Agent 模式资源
    println!("【ATX-Agent 模式资源】");
    check_file(assets_dir.join("atx-agent"), "ATX-Agent 二进制文件", false);
    check_file(
        assets_dir.join("app-uiautomator-test.apk"),
        "UiAutomator2 测试 APK",
        false,
    );

    println!();

    // 编译时嵌入的 MD5 值
    println!("【编译时 MD5 值】");
    println!("  u2.jar MD5:                 {}", env!("U2_JAR_MD5"));
    println!("  atx-agent MD5:              {}", env!("ATX_AGENT_MD5"));
    println!(
        "  app-uiautomator.apk MD5:    {}",
        env!("UIAUTOMATOR_APK_MD5")
    );

    println!();
    println!("=== 验证完成 ===");
    println!();
    println!("提示：");
    println!("  - Direct 模式只需要 u2.jar 和 app-uiautomator.apk");
    println!("  - ATX-Agent 模式需要所有文件");
    println!("  - 如果缺少 ATX-Agent 资源，运行：");
    println!("    Linux/macOS: cd assets && ./download_atx_agent.sh");
    println!("    Windows:     cd assets && .\\download_atx_agent.ps1");
}

fn check_file(path: impl AsRef<Path>, description: &str, required: bool) {
    let path = path.as_ref();
    let exists = path.exists();

    let status = if exists {
        "✓"
    } else if required {
        "✗ (必需)"
    } else {
        "- (可选)"
    };

    print!("  {} {:<30}", status, description);

    if exists {
        if let Ok(metadata) = std::fs::metadata(path) {
            let size = metadata.len();
            let size_mb = size as f64 / 1024.0 / 1024.0;
            println!(" ({:.2} MB)", size_mb);
        } else {
            println!();
        }
    } else {
        println!(" (不存在)");
    }
}
