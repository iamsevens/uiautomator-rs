# Third-Party Notices

本仓库包含或依赖以下第三方项目资源。以下说明用于追踪来源，具体授权条款以上游仓库为准。

## Embedded/Bundled Assets

| Component | Source | License (upstream) | Used in |
| --- | --- | --- | --- |
| `u2.jar` | `openatx/uiautomator2` | Upstream project license | `uiautomator` |
| `atx-agent` | `openatx/atx-agent` | MIT | `uiautomator`, `uiautomator-cli` |
| `app-uiautomator.apk` | `openatx/android-uiautomator-server` | Upstream project license | `uiautomator`, `uiautomator-cli` |
| `app-uiautomator-test.apk` | `openatx/android-uiautomator-server` | Upstream project license | `uiautomator`, `uiautomator-cli` |

## Upstream Links

- https://github.com/openatx/uiautomator2
- https://github.com/openatx/atx-agent
- https://github.com/openatx/android-uiautomator-server

## Notes

- `uiautomator-cli/assets/LICENSE` 当前包含 `atx-agent` 相关 MIT 文本。
- 若上游许可证发生变化，请在发布前同步更新本文件与相应 crate 文档。
