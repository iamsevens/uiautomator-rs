[English](./SECURITY.md) | [简体中文](./SECURITY.zh-CN.md)

# Security Policy

## Supported Versions

| Version | Supported |
| --- | --- |
| `main` | Yes |
| latest release | Yes |
| older releases | Best effort |

## Reporting a Vulnerability

请不要在公开 Issue 直接披露安全细节。

推荐流程：
1. 使用 GitHub 的私密漏洞上报（Private Vulnerability Reporting / Security Advisory）。
2. 提供最小复现信息：影响范围、触发条件、复现步骤、潜在影响。
3. 如私密上报入口不可用，请先创建不含细节的占位 Issue（例如标题 `Security report request`），维护者会引导到私密渠道。

## Response Targets

- 48 小时内确认收到报告（工作日）。
- 7 天内给出初步评估（严重级别与修复计划）。
- 修复后在发布说明中公开致谢（如你同意）。

## Scope

本策略覆盖：
- `uiautomator/`
- `uiautomator-cli/`
- 发布流程与 CI 配置

