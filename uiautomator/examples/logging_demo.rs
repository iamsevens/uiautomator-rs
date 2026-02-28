//! 日志系统演示示例
//!
//! 运行方式:
//! ```bash
//! # 默认日志级别 (info)
//! cargo run --example logging_demo
//!
//! # 显示调试信息
//! RUST_LOG=debug cargo run --example logging_demo
//!
//! # 只显示警告和错误
//! RUST_LOG=warn cargo run --example logging_demo
//! ```

use log::{debug, error, info, warn};

fn main() {
    // 初始化日志系统
    uiautomator::init_logger();

    info!("日志系统已初始化");
    debug!("这是调试信息 - 默认不显示,需要设置 RUST_LOG=debug");
    info!("这是一般信息 - 默认显示");
    warn!("这是警告信息");
    error!("这是错误信息");

    // 演示在关键操作点记录日志
    info!("开始连接设备...");
    debug!("设备序列号: emulator-5554");
    info!("设备连接成功");

    info!("开始执行自动化操作...");
    debug!("查找元素: text='Settings'");
    info!("元素定位成功");

    warn!("操作耗时较长,可能需要等待");
    info!("操作完成");
}
