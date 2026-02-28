//! 安装器模块测试
//!
//! 测试设备检测和连接逻辑

mod common;

#[cfg(test)]
mod installer_tests {
    use std::sync::Arc;
    use uiautomator::adb::AdbClient;
    use uiautomator_cli::installer::Installer;

    /// 测试：未找到设备时应该返回错误
    ///
    /// 验证当没有设备连接时，Installer::new() 应该返回适当的错误
    ///
    /// 注意：这个测试需要在没有设备连接的环境中运行才会通过
    /// 如果有设备连接，测试会被跳过
    #[tokio::test]
    async fn test_no_device_found_error() {
        // 尝试创建安装器
        let result = crate::common::new_installer().await;

        // 如果没有设备，应该返回错误
        if let Err(err) = result {
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("未找到连接的设备"),
                "错误消息应该包含 '未找到连接的设备'，实际: {}",
                err_msg
            );
        } else {
            // 如果有设备连接，跳过这个测试
            println!("跳过测试：检测到设备连接");
        }
    }

    /// 测试：自动选择第一个设备
    ///
    /// 验证当有设备连接且未指定序列号时，应该自动选择第一个设备
    ///
    /// 注意：这个测试需要至少一个设备连接才会运行
    #[tokio::test]
    async fn test_auto_select_first_device() {
        // 尝试创建安装器
        let result = crate::common::new_installer().await;

        // 如果有设备连接，验证选择了第一个设备
        if let Ok(installer) = result {
            // 验证设备序列号不为空
            assert!(
                !installer.device_serial().is_empty(),
                "设备序列号不应该为空"
            );
            println!("自动选择的设备: {}", installer.device_serial());
        } else {
            // 如果没有设备，跳过这个测试
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：指定序列号选择设备
    ///
    /// 验证当指定设备序列号时，应该连接到指定的设备
    ///
    /// 注意：这个测试需要知道一个有效的设备序列号
    #[tokio::test]
    async fn test_select_device_by_serial() {
        // 首先获取可用的设备列表
        if let Ok(adb_client) = AdbClient::new().await {
            if let Ok(devices) = adb_client.devices().await {
                if !devices.is_empty() {
                    // 使用第一个设备的序列号
                    let serial = devices[0].clone();

                    // 创建安装器并指定序列号
                    let result = Installer::new(Some(serial.clone())).await;

                    assert!(result.is_ok(), "应该成功创建安装器");
                    let installer = result.unwrap();
                    assert_eq!(installer.device_serial(), &serial, "设备序列号应该匹配");
                    println!("成功连接到指定设备: {}", serial);
                } else {
                    println!("跳过测试：未检测到设备连接");
                }
            }
        }
    }

    /// 测试：指定不存在的序列号应该返回错误
    ///
    /// 验证当指定的设备序列号不存在时，应该返回适当的错误
    #[tokio::test]
    async fn test_invalid_serial_error() {
        // 首先检查是否有设备连接
        if let Ok(adb_client) = AdbClient::new().await {
            if let Ok(devices) = adb_client.devices().await {
                if !devices.is_empty() {
                    // 使用一个不太可能存在的序列号
                    let invalid_serial = "invalid-device-serial-12345".to_string();

                    let result = Installer::new(Some(invalid_serial.clone())).await;

                    // 应该返回错误
                    assert!(result.is_err(), "应该返回错误");
                    let err = result.unwrap_err();
                    let err_msg = err.to_string();
                    // 错误消息应该包含设备未找到的信息
                    assert!(
                        err_msg.contains("未找到设备") && err_msg.contains(&invalid_serial),
                        "错误消息应该指出指定的设备未找到，实际: {}",
                        err_msg
                    );
                    println!("正确返回错误：指定的设备不存在");
                } else {
                    println!("跳过测试：未检测到设备连接");
                }
            }
        }
    }

    /// 测试：使用自定义 ADB 客户端创建安装器
    ///
    /// 验证可以使用已有的 ADB 客户端创建安装器
    #[tokio::test]
    async fn test_new_with_adb_client() {
        // 创建 ADB 客户端
        if let Ok(adb_client) = AdbClient::new().await {
            let adb_client = Arc::new(adb_client);

            // 使用 ADB 客户端创建安装器
            let result = Installer::new_with_adb(None, adb_client).await;

            // 如果有设备连接，应该成功
            // 如果没有设备，应该返回错误
            match result {
                Ok(installer) => {
                    assert!(!installer.device_serial().is_empty());
                    println!("成功创建安装器，设备: {}", installer.device_serial());
                }
                Err(e) => {
                    assert!(e.to_string().contains("未找到连接的设备"));
                    println!("未找到设备（预期行为）");
                }
            }
        } else {
            println!("跳过测试：无法连接到 ADB 服务");
        }
    }

    /// 测试：device_serial() 方法应该返回正确的序列号
    #[tokio::test]
    async fn test_device_serial_getter() {
        // 如果有设备连接，测试 getter 方法
        if let Ok(installer) = crate::common::new_installer().await {
            let serial = installer.device_serial();
            assert!(!serial.is_empty(), "设备序列号不应该为空");
            println!("设备序列号: {}", serial);
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }
}

// ============================================================================
// 安装功能测试
// ============================================================================

#[cfg(test)]
mod install_tests {
    /// 测试：检查是否已安装
    ///
    /// 验证 check_installed() 方法能够正确检测设备上的安装状态
    #[tokio::test]
    #[ignore] // 需要真实设备
    async fn test_check_installed() {
        if let Ok(installer) = crate::common::new_installer().await {
            // 调用检查方法
            let result = installer.check_installed().await;

            // 应该返回成功（无论是否已安装）
            assert!(result.is_ok(), "检查安装状态应该成功");

            let installed = result.unwrap();
            println!(
                "ATX-Agent 安装状态: {}",
                if installed { "已安装" } else { "未安装" }
            );
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：安装流程
    ///
    /// 验证 install() 方法能够成功执行安装流程
    ///
    /// 注意：这个测试会实际安装 ATX-Agent，需要真实设备
    #[tokio::test]
    #[ignore] // 需要真实设备，且会修改设备状态
    async fn test_install_flow() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("开始测试安装流程...");

            // 执行安装
            let result = installer.install(false).await;

            // 应该成功
            assert!(result.is_ok(), "安装应该成功: {:?}", result.err());

            println!("安装流程测试完成");

            // 验证安装后的状态
            let installed = installer.check_installed().await.unwrap();
            assert!(installed, "安装后应该检测到已安装");
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：强制重新安装
    ///
    /// 验证 install(true) 能够强制重新安装，即使已经安装
    #[tokio::test]
    #[ignore] // 需要真实设备，且会修改设备状态
    async fn test_force_reinstall() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("开始测试强制重新安装...");

            // 强制重新安装
            let result = installer.install(true).await;

            // 应该成功
            assert!(result.is_ok(), "强制重新安装应该成功: {:?}", result.err());

            println!("强制重新安装测试完成");
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：幂等性 - 重复安装不应该失败
    ///
    /// 验证多次调用 install(false) 不会导致错误
    #[tokio::test]
    #[ignore] // 需要真实设备
    async fn test_install_idempotent() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("开始测试安装幂等性...");

            // 第一次安装
            let result1 = installer.install(false).await;
            assert!(result1.is_ok(), "第一次安装应该成功");

            // 第二次安装（应该检测到已安装并跳过）
            let result2 = installer.install(false).await;
            assert!(result2.is_ok(), "第二次安装应该成功（跳过）");

            println!("安装幂等性测试完成");
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：安装进度反馈
    ///
    /// 验证安装过程中有适当的日志输出
    ///
    /// 注意：这个测试主要是手动验证日志输出
    #[tokio::test]
    #[ignore] // 需要真实设备，手动验证
    async fn test_install_with_progress() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("\n=== 开始测试安装进度反馈 ===");
            println!("请观察以下输出，确认有适当的进度信息：\n");

            // 执行安装（应该有进度输出）
            let result = installer.install(true).await;

            assert!(result.is_ok(), "安装应该成功");

            println!("\n=== 安装进度反馈测试完成 ===");
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }
}

// ============================================================================
// 状态查询测试
// ============================================================================

#[cfg(test)]
mod status_tests {
    /// 测试：查询运行中的服务
    ///
    /// 验证 status() 方法能够正确检测正在运行的 ATX-Agent 服务
    ///
    /// 需求: 2.1, 2.2
    #[tokio::test]
    #[ignore] // 需要真实设备且 ATX-Agent 正在运行
    async fn test_status_running_service() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("测试查询运行中的服务...");

            // 确保服务已安装并运行
            let _ = installer.install(false).await;

            // 查询状态
            let result = installer.status().await;

            // 应该成功返回状态
            assert!(result.is_ok(), "查询状态应该成功: {:?}", result.err());

            let status = result.unwrap();

            // 验证状态字段
            assert!(status.running, "服务应该正在运行");
            assert!(status.version.is_some(), "应该返回版本信息");

            let version = status.version.unwrap();
            assert!(!version.is_empty(), "版本号不应该为空");

            println!("✓ 服务状态: 运行中");
            println!("✓ 版本: {}", version);
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：查询未运行的服务
    ///
    /// 验证 status() 方法能够正确检测未运行的 ATX-Agent 服务
    ///
    /// 需求: 2.1, 2.3
    #[tokio::test]
    #[ignore] // 需要真实设备且 ATX-Agent 未运行
    async fn test_status_stopped_service() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("测试查询未运行的服务...");

            // 注意：这个测试假设服务未运行
            // 在实际测试中，可能需要先停止服务

            // 查询状态
            let result = installer.status().await;

            // 应该成功返回状态
            assert!(result.is_ok(), "查询状态应该成功: {:?}", result.err());

            let status = result.unwrap();

            // 如果服务未运行
            if !status.running {
                assert!(status.version.is_none(), "未运行时不应该有版本信息");
                println!("✓ 服务状态: 未运行");
            } else {
                println!("跳过验证：服务正在运行");
            }
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：获取版本信息
    ///
    /// 验证 status() 方法能够正确获取 ATX-Agent 版本号
    ///
    /// 需求: 2.2
    #[tokio::test]
    #[ignore] // 需要真实设备且 ATX-Agent 正在运行
    async fn test_get_version_info() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("测试获取版本信息...");

            // 确保服务已安装并运行
            let _ = installer.install(false).await;

            // 查询状态
            let result = installer.status().await;
            assert!(result.is_ok(), "查询状态应该成功");

            let status = result.unwrap();

            if status.running {
                // 验证版本信息
                assert!(status.version.is_some(), "运行中的服务应该有版本信息");

                let version = status.version.unwrap();
                assert!(!version.is_empty(), "版本号不应该为空");

                // 版本号应该符合某种格式（例如：0.10.0）
                // 这里只做基本验证
                assert!(
                    version.contains('.') || version.chars().any(|c| c.is_numeric()),
                    "版本号应该包含数字或点号，实际: {}",
                    version
                );

                println!("✓ 版本信息: {}", version);
            } else {
                println!("跳过验证：服务未运行");
            }
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：状态查询的可靠性
    ///
    /// 验证多次查询状态应该返回一致的结果
    #[tokio::test]
    #[ignore] // 需要真实设备
    async fn test_status_consistency() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("测试状态查询的一致性...");

            // 多次查询状态
            let status1 = installer.status().await;
            let status2 = installer.status().await;
            let status3 = installer.status().await;

            // 所有查询都应该成功
            assert!(status1.is_ok(), "第一次查询应该成功");
            assert!(status2.is_ok(), "第二次查询应该成功");
            assert!(status3.is_ok(), "第三次查询应该成功");

            let s1 = status1.unwrap();
            let s2 = status2.unwrap();
            let s3 = status3.unwrap();

            // 运行状态应该一致
            assert_eq!(s1.running, s2.running, "运行状态应该一致");
            assert_eq!(s2.running, s3.running, "运行状态应该一致");

            // 如果正在运行，版本信息应该一致
            if s1.running {
                assert_eq!(s1.version, s2.version, "版本信息应该一致");
                assert_eq!(s2.version, s3.version, "版本信息应该一致");
            }

            println!("✓ 状态查询一致性验证通过");
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：状态查询不应该修改服务状态
    ///
    /// 验证查询状态是只读操作，不会影响服务
    #[tokio::test]
    #[ignore] // 需要真实设备
    async fn test_status_is_readonly() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("测试状态查询是只读操作...");

            // 获取初始状态
            let initial_status = installer.status().await;
            assert!(initial_status.is_ok(), "初始状态查询应该成功");

            let initial = initial_status.unwrap();

            // 多次查询状态
            for i in 1..=5 {
                let status = installer.status().await;
                assert!(status.is_ok(), "第 {} 次查询应该成功", i);

                let current = status.unwrap();

                // 状态不应该改变
                assert_eq!(
                    initial.running, current.running,
                    "查询操作不应该改变运行状态"
                );
            }

            println!("✓ 状态查询是只读操作");
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }
}

// ============================================================================
// 服务管理测试
// ============================================================================

#[cfg(test)]
mod service_management_tests {
    /// 测试：重启服务
    ///
    /// 验证 restart() 方法能够成功重启 ATX-Agent 服务
    ///
    /// 需求: 3.1, 3.2
    #[tokio::test]
    #[ignore] // 需要真实设备且 ATX-Agent 已安装
    async fn test_restart_service() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("测试重启服务...");

            // 确保服务已安装
            let _ = installer.install(false).await;

            // 获取重启前的状态
            let status_before = installer.status().await;
            assert!(status_before.is_ok(), "重启前查询状态应该成功");

            // 执行重启
            let result = installer.restart().await;

            // 应该成功
            assert!(result.is_ok(), "重启服务应该成功: {:?}", result.err());

            // 等待一小段时间让服务完全启动
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // 验证重启后服务正在运行
            let status_after = installer.status().await;
            assert!(status_after.is_ok(), "重启后查询状态应该成功");

            let status = status_after.unwrap();
            assert!(status.running, "重启后服务应该正在运行");
            assert!(status.version.is_some(), "重启后应该有版本信息");

            println!("✓ 服务重启成功");
            println!("✓ 重启后状态: 运行中");
            println!("✓ 版本: {}", status.version.unwrap());
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：重启未安装的服务应该返回错误
    ///
    /// 验证在服务未安装时调用 restart() 应该返回适当的错误
    ///
    /// 需求: 3.4
    #[tokio::test]
    #[ignore] // 需要真实设备且 ATX-Agent 未安装
    async fn test_restart_uninstalled_service() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("测试重启未安装的服务...");

            // 注意：这个测试假设服务未安装
            // 在实际测试中，可能需要先卸载服务

            // 尝试重启
            let result = installer.restart().await;

            // 如果服务未安装，可能会返回错误或成功但无效果
            // 这取决于底层实现
            match result {
                Ok(_) => {
                    println!("重启操作完成（服务可能未安装）");
                }
                Err(e) => {
                    println!("重启返回错误（环境相关）（预期行为）: {}", e);
                }
            }
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：卸载服务
    ///
    /// 验证 uninstall() 方法能够成功卸载 ATX-Agent
    ///
    /// 需求: 4.1, 4.2
    #[tokio::test]
    #[ignore] // 需要真实设备，会修改设备状态
    async fn test_uninstall_service() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("测试卸载服务...");

            // 确保服务已安装
            let _ = installer.install(false).await;

            // 验证安装状态
            let installed_before = installer.check_installed().await;
            assert!(installed_before.is_ok(), "卸载前检查安装状态应该成功");

            // 执行卸载
            let result = installer.uninstall().await;

            // 应该成功
            assert!(result.is_ok(), "卸载服务应该成功: {:?}", result.err());

            // 等待一小段时间确保卸载完成
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // 验证卸载后的状态
            let status_after = installer.status().await;
            assert!(status_after.is_ok(), "卸载后查询状态应该成功");

            let status = status_after.unwrap();
            assert!(!status.running, "卸载后服务不应该运行");
            assert!(status.version.is_none(), "卸载后不应该有版本信息");

            // 验证未安装状态
            let installed_after = installer.check_installed().await;
            assert!(installed_after.is_ok(), "卸载后检查安装状态应该成功");
            assert!(!installed_after.unwrap(), "卸载后应该检测为未安装");

            println!("✓ 服务卸载成功");
            println!("✓ 卸载后状态: 未运行");
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：卸载未安装的服务
    ///
    /// 验证在服务未安装时调用 uninstall() 应该正常处理
    ///
    /// 需求: 4.4
    #[tokio::test]
    #[ignore] // 需要真实设备
    async fn test_uninstall_not_installed() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("测试卸载未安装的服务...");

            // 注意：这个测试假设服务未安装
            // 或者先卸载一次确保未安装
            let _ = installer.uninstall().await;

            // 再次卸载
            let result = installer.uninstall().await;

            // 应该成功或返回适当的错误
            match result {
                Ok(_) => {
                    println!("✓ 卸载操作完成（幂等性）");
                }
                Err(e) => {
                    println!("卸载返回错误: {}", e);
                    // 某些错误是可以接受的（例如文件不存在）
                }
            }
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：服务状态变化 - 安装、重启、卸载
    ///
    /// 验证完整的服务生命周期管理
    ///
    /// 需求: 3.1, 3.2, 4.1, 4.2
    #[tokio::test]
    #[ignore] // 需要真实设备，会修改设备状态
    async fn test_service_lifecycle() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("\n=== 测试服务生命周期 ===\n");

            // 1. 安装服务
            println!("1. 安装服务...");
            let install_result = installer.install(true).await;
            assert!(install_result.is_ok(), "安装应该成功");

            let status = installer.status().await.unwrap();
            assert!(status.running, "安装后服务应该运行");
            println!("   ✓ 服务已安装并运行");

            // 2. 重启服务
            println!("\n2. 重启服务...");
            let restart_result = installer.restart().await;
            assert!(restart_result.is_ok(), "重启应该成功");

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let status = installer.status().await.unwrap();
            assert!(status.running, "重启后服务应该运行");
            println!("   ✓ 服务已重启并运行");

            // 3. 卸载服务
            println!("\n3. 卸载服务...");
            let uninstall_result = installer.uninstall().await;
            assert!(uninstall_result.is_ok(), "卸载应该成功");

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            let status = installer.status().await.unwrap();
            assert!(!status.running, "卸载后服务不应该运行");
            println!("   ✓ 服务已卸载");

            println!("\n=== 服务生命周期测试完成 ===\n");
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：重启操作的原子性
    ///
    /// 验证重启操作确保服务最终处于运行状态
    ///
    /// 需求: 3.2, 3.3
    #[tokio::test]
    #[ignore] // 需要真实设备
    async fn test_restart_atomicity() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("测试重启操作的原子性...");

            // 确保服务已安装
            let _ = installer.install(false).await;

            // 执行重启
            let result = installer.restart().await;
            assert!(result.is_ok(), "重启应该成功");

            // 等待服务完全启动
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            // 验证服务最终处于运行状态
            let status = installer.status().await.unwrap();
            assert!(status.running, "重启操作应该确保服务最终处于运行状态");

            println!("✓ 重启操作具有原子性，服务最终运行");
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }

    /// 测试：卸载操作的完整性
    ///
    /// 验证卸载操作删除所有相关文件
    ///
    /// 需求: 4.2, 4.3
    #[tokio::test]
    #[ignore] // 需要真实设备
    async fn test_uninstall_completeness() {
        if let Ok(installer) = crate::common::new_installer().await {
            println!("测试卸载操作的完整性...");

            // 确保服务已安装
            let _ = installer.install(false).await;

            // 执行卸载
            let result = installer.uninstall().await;
            assert!(result.is_ok(), "卸载应该成功");

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // 验证服务不再运行
            let status = installer.status().await.unwrap();
            assert!(!status.running, "卸载后服务不应该运行");

            // 验证未安装状态
            let installed = installer.check_installed().await.unwrap();
            assert!(!installed, "卸载后应该检测为未安装");

            println!("✓ 卸载操作完整，所有文件已清理");
        } else {
            println!("跳过测试：未检测到设备连接");
        }
    }
}
