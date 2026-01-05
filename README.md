# V2Ray 规则转 AutoProxy 格式订阅

本项目是一个自动化工具，旨在将 [Loyalsoldier/v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat) 提供的 V2Ray 规则列表转换为 **ZeroOmega (原 SwitchyOmega)** 等浏览器插件可用的 **AutoProxy** 格式。

## 🌟 项目特点

- **高性能转换**：核心逻辑采用 **Rust** 编写，处理十万级规则仅需毫秒级时间。
- **全自动化更新**：利用 GitHub Actions 每天定时抓取上游更新并重新生成订阅文件。
- **标准 Base64 编码**：生成的订阅文件经过标准 Base64 编码，确保在各类代理插件中拥有最佳的兼容性。
- **即用型地址**：无需自行配置环境，直接引用本项目生成的链接即可使用。

## 🔗 订阅地址

请在 ZeroOmega / SwitchyOmega 的“规则列表网址”中直接使用以下链接：

| 规则类型 | 原始数据源 | AutoProxy 订阅地址 (Raw) |
| :--- | :--- | :--- |
| **直连列表 (Direct)** | `direct-list.txt` | `https://raw.githubusercontent.com/kcgp007/v2ray-rules-dat_direct-list_convert/main/direct.txt` |
| **代理列表 (Proxy)** | `proxy-list.txt` | `https://raw.githubusercontent.com/kcgp007/v2ray-rules-dat_direct-list_convert/main/proxy.txt` |

## 🛠 使用方法

1.  **新建情景模式**：在插件选项页点击“新建情景模式”，类型选择 **“规则列表 (Rule List)”**。
2.  **配置规则**：
    *   **规则格式**：选择 `AutoProxy`。
    *   **规则列表网址**：从上方表格复制对应的 `Raw` 链接。
3.  **模式匹配**：
    *   将 `direct.txt` 的结果情景模式设为 **`DIRECT`**。
    *   将 `proxy.txt` 的结果情景模式设为您的 **代理服务器**。
4.  **立即更新**：点击“立即更新”并应用选项。

## ⚙️ 工作原理

项目通过 Rust 编写的转换引擎，对上游数据进行以下处理：
- 识别 `domain:` 前缀并转换为 AutoProxy 的 `||` 子域名匹配格式。
- 识别 `full:` 前缀并转换为 `|` 精确匹配格式。
- 识别 `regexp:` 前缀并转换为 `/ /` 正则匹配格式。
- 识别 `keyword:` 前缀并保留为字符串匹配。
- 最终对所有转换后的规则行进行聚合及 **Base64** 编码输出。

## ⏰ 更新频率

- **自动更新**：每天 UTC 0:00（北京时间 08:00）自动执行同步。
- **手动触发**：支持在仓库 `Actions` 页面通过 `workflow_dispatch` 手动触发即时更新。

## 📄 许可证

本项目基于 **GNU General Public License v3.0** 许可证开源。

- 您可以自由复制、修改和分发本项目代码。
- 任何基于本项目代码的衍生作品也必须以 **GPL v3.0** 协议开源。
- 更多细节请参阅 [LICENSE](LICENSE) 文件。

## ⚖️ 免责声明

本项目仅作为格式转换工具，不提供任何代理服务。规则内容由 [Loyalsoldier/v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat) 维护，本项目的准确性依赖于上游数据源及转换逻辑的有效性。用户需遵守当地法律法规使用。
