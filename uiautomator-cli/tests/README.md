# uiautomator-cli 测试

本目录包含 uiautomator-cli 的所有测试。

## 测试结构

### 单元测试
- `cli_test.rs` - CLI 参数解析测试
- `resources_test.rs` - 资源文件管理测试
- `installer_test.rs` - 安装器逻辑测试（使用 mock）
- `error_test.rs` - 错误消息格式测试

### 集成测试
- `integration_init_test.rs` - init 命令集成测试
- `integration_service_test.rs` - 服务管理集成测试（restart/uninstall）
- `integration_idempotent_test.rs` - 幂等性测试

### 属性测试
- `property_resources_test.rs` - 资源文件完整性属性测试
- `property_idempotent_test.rs` - 幂等性属性测试

## 运行测试

### 运行所有单元测试
```bash
cargo test --lib
```

### 运行所有集成测试（需要真实设备）
```bash
cargo test --test '*' -- --ignored
```

### 运行属性测试
```bash
cargo test property
```

### 运行特定测试
```bash
cargo test test_name
```

## 注意事项

1. 集成测试需要连接真实的 Android 设备
2. 集成测试使用 `#[ignore]` 标记，默认不运行
3. 属性测试会运行多次（默认 100 次）以验证属性
4. Mock 测试不需要真实设备，可以在 CI 中运行
