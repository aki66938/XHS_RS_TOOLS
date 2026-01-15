# XHS-RS-TOOLS

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Language](https://img.shields.io/badge/language-Rust%20%7C%20Python-orange.svg)

**[仅供学习研究使用 / For Educational Purposes Only]**

## ⚠️ 免责声明 (Disclaimer)

1.  **学习用途**: 本项目仅供编程爱好者学习 Rust 与 Python 混合开发、逆向分析思路及浏览器自动化技术使用。
2.  **严禁商用**: 严禁将本项目用于任何商业用途、黑灰产或其他非法目的。
3.  **后果自负**: 使用本项目产生的任何后果（包括但不限于账号封禁、法律责任）由使用者自行承担。开发者不对任何衍生作品或使用行为负责。
4.  **接口变动**: 项目基于特定时间点的目标网站状态开发，不保证长期有效性。

## 📖 项目概述

本项目是一个基于 **Rust** (后端 API) 和 **Playwright** (前端采集) 的小红书 API 逆向与自动化工具集。

### 核心实现逻辑 (The "Secret Sauce")

小红书 Web 端接口的风控机制极为严格，核心在于：**Cookie、Request Header 与 X-S 签名强绑定**。
简单的签名生成往往无法通过校验，因为服务端会校验签名的生成环境（Cookie、Canvas 指纹等）是否与请求发起者一致。

本项目采用 **"Capture & Replay" (捕获与重放)** 策略解决此问题：
1.  **真实环境登录**: 使用 Playwright 模拟真实浏览器登录，获取合法的 Session 和 Cookie。
2.  **签名捕获**: 在浏览器操作过程中（如浏览 Feed、搜索），自动拦截并提取官方生成的合法 `X-s` 签名及其对应的完整 Payload。
3.  **持久化会话**: 将 (Cookie + Header + Signature + Payload) 作为一个原子单元存储。
4.  **服务端重放**: Rust 后端通过 API 暴露这些能力。当客户端请求某个接口时，后端会**强制使用**数据库中存储的、已通过验证的 Payload 和签名进行请求，从而完美规避反爬策略。

## 🚀 当前功能 (v1.2.0)

以下均为目前已实现并验证的功能：

*   **二维码扫码登录**: 从 DOM 中获取二维码图片信息，可以直接复制到浏览器展示，也可以客户端自行实现在控制台显示 ASCII 二维码。
*   **实时登录状态查询** (NEW): 通过 `/api/auth/qrcode-status` 轮询扫码状态（未扫码/已扫码/登录成功）。
*   **签名采集进度查询** (NEW): 通过 `/api/auth/capture-status` 查询后台签名采集是否完成。
*   **会话持久化**: 登录成功后自动保存 Session，支持长时间复用（具体失效时间不可保证），无需频繁扫码。
*   **全频道 Feed 采集**: 支持首页推荐及所有子频道（穿搭、美食、情感、游戏等 11 个频道）的数据获取。
    *   *特色*: 采用拟人化遍历算法，精准捕获各频道专属签名。
*   **热搜榜单**: 获取实时热搜关键词。
*   **通知页采集**: 获取评论和 @ 、新增关注通知。
*   **图文详情**: 获取指定笔记的评论列表，支持分页。

## 📅 开发日志 (Dev Log)

| 版本 | 日期 | 更新内容 | 备注 |
| :--- | :--- | :--- | :--- |
| **v1.2.0** | 2026-01-16 | **新增端点与代码优化** | |
| | | - 🎉 新增 `/api/auth/qrcode-status` | 实时查询扫码登录状态 (0:未扫码/1:已扫码/2:成功) |
| | | - 🎉 新增 `/api/auth/capture-status` | 查询签名采集进度 (是否完成、已采集数量) |
| | | - 🐛 修复 Python 脚本 UTF-8 编码问题 | Windows 环境 Rust 读取 stdout 报错 |
| | | - ⚡ 优化日志时区显示 | 从 UTC 改为本地时区 (默认 UTC+8) |
| **v1.1.0** | 2026-01-15 | **新增接口与模块重构** | |
| | | - 新增通知页采集: `/api/notification/mentions` 和 `/connections` | 评论/@、新增关注 |
| | | - 新增图文详情: `/api/note/page` | 笔记评论分页 |
| | | - 重构 API 公共模块 `XhsApiClient` | Header/Cookie/Signature 统一管理 |
| | | - 优化 Swagger 文档 | 详细列出 11 个 Feed 频道 |
| | | - 修复浏览器关闭时崩溃问题 | 捕获 TargetClosedError |
| **v1.0.0** | 2026-01-11 | **项目初始化 Release** | |
| | | - 实现 Playwright 扫码登录流程 (Python) | 解决 Headless 模式风控 |
| | | - 实现 Rust Axum API 服务端 | 提供 `/api/auth` 等接口 |
| | | - 集成 MongoDB 存储凭证与签名 | |
| | | - 实现 "11 频道" 拟人化自动采集策略 | 解决 "Illegal Request" |
| | | - 优化 Client Demo 演示脚本 | 终端可视化测试 |

## 🛠️ 快速开始

### 1. 环境准备
- Rust (Cargo)
- Python 3.8+
- MongoDB (Running on localhost:27017)
- Playwright (`playwright install chromium`)

### 2. 启动服务
```bash
# 1. 启动 Rust API 服务 (会自动调用 Python 脚本)
cargo run
```

### 3. 运行演示客户端
```bash
# 2. 在另一个终端运行测试脚本
python client_demo.py

# 第一次扫码登录完成，服务端（也就是api）会自动保存session，但采集发现页面各个频道的签名信息需要时间
# 通常情况下当看到【Session Update: {"success": true, "user_id": "19badc2722c0lpxxm2re19ws4onlft8quq3u", "cookie_count": 13, "signatures_captured": ["home_feed_recommend", "search_trending", "home_feed_fashion", "home_feed_food", "home_feed_cosmetics", "home_feed_movie_and_tv", "home_feed_career", "home_feed_love", "home_feed_household_product", "home_feed_gaming", "home_feed_travel", "home_feed_fitness"], "error": null}】这样的信息说明采集完成，再次运行python client_demo.py可以验证接口可用，页面信息可获取
```

## 🔌 已验证 API 列表 (Implemented APIs)

| Category | Endpoint | Status | Description |
| :--- | :--- | :--- | :--- |
| **Auth** | `/api/auth/login-session` | ✅ | 初始化登录会话 (流式响应) |
| **Auth** | `/api/auth/session` | ✅ | 检查 Session 有效性 |
| **Auth** | `/api/auth/qrcode-status` | ✅ | 实时查询扫码状态 (0:未扫码/1:已扫码/2:成功) |
| **Auth** | `/api/auth/capture-status` | ✅ | 查询签名采集进度 |
| **User** | `/api/user/me` | ✅ | 获取当前用户信息 |
| **Search** | `/api/search/trending` | ✅ | 获取实时热搜关键词 |
| **Feed** | `/api/feed/homefeed/{category}` | ✅ | 11 个垂直频道 (recommend/fashion/food/cosmetics/movie_and_tv/career/love/household_product/gaming/travel/fitness) |
| **Notification** | `/api/notification/mentions` | ✅ | 获取评论和 @ 通知 |
| **Notification** | `/api/notification/connections` | ✅ | 获取新增关注通知 |
| **Note** | `/api/note/page` | ✅ | 获取笔记评论 (支持分页) |

## 📚 接口文档 (API Docs)

本项目内置 Swagger UI，启动服务后即可访问：
- **地址**: `http://localhost:3005/swagger-ui/`
- **使用**: 可在网页上直接发起请求测试接口。

## 👨‍💻 作者自述 (Author's Note)

### 项目起源与转型
本项目的灵感来源于我对 **RPA (Robotic Process Automation)** 技术及浏览器自动化控制的深入研究。
在前作 `xhs_tools` (1.1k stars) 的维护过程中，我深刻体会到单纯依赖 Python 脚本进行自动化操作的局限性。随着 AI 技术（特别是像 Google Gemini 这样具备强大浏览器理解能力的模型）的崛起，传统的硬编码自动化正在被智能代理所取代。

因此，我决定暂停旧项目的更新，转向探索架构更先进、性能更强劲的解决方案：
*   **架构升级**: 核心网络层采用 **Rust** 重写，确保极高的并发性能与类型安全。
*   **自动化基座**: 保留 Python (Playwright) 作为浏览器控制层，专注于处理复杂的 DOM 交互与人机验证。
*   **API 化**: 将所有功能封装为标准 HTTP 接口，为未来接入 AI Agent 或其他上层应用提供坚实基础。

### 法律/合规声明 (Legal Statement)
**请务必仔细阅读：**

1.  **技术研究性质**: 本项目本质上是一个 **浏览器自动化框架** 的实践案例。所有的“采集”行为均基于模拟真实用户的常规浏览操作（点击、滚动、网络请求），**不包含** 任何破解加密算法、绕过身份验证或其他攻击目标服务器安全机制的代码。
2.  **数据安全**: 本项目**不提供**任何现成的账号或 Cookie。用户必须通过官方渠道（扫码）进行合法登录。项目仅作为数据的本地处理工具，不收集、不上传任何用户敏感信息。
3.  **合规使用**: 请使用者严格遵守《中华人民共和国网络安全法》及目标网站的《用户服务协议》。严禁将本项目用于数据爬取（Scraping）、批量账号控制（Botting）或任何侵犯他人隐私/知识产权的商业行为。
4.  **免责条款**: 开源作者不对任何因使用本项目而导致的法律纠纷或账号损失承担责任。代码仅作为技术交流用途，下载后请于 24 小时内删除。

---
*以技术探索为名，行守法合规之事。*

## 📄 开源协议
MIT License
