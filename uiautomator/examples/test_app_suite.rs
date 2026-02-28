//! 完整测试套件
//!
//! 运行所有测试场景

mod test_app;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志
    uiautomator::init_logger();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║        UIAutomator Test App - 完整测试套件                 ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    // 创建测试上下文
    let mut ctx = test_app::TestContext::new().await?;

    // 启动测试应用
    ctx.launch_test_app().await?;

    // 运行基础控件测试
    test_app::basic_controls::run_all_tests(&mut ctx).await?;

    // TODO: 添加更多测试模块
    // gestures::run_all_tests(&mut ctx).await?;
    // input_forms::run_all_tests(&mut ctx).await?;
    // lists::run_all_tests(&mut ctx).await?;
    // dialogs::run_all_tests(&mut ctx).await?;

    // 打印测试摘要
    ctx.print_summary();

    // 根据测试结果返回退出码
    let failed = ctx.results.iter().filter(|r| !r.passed).count();
    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
