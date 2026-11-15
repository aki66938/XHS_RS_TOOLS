# XHS_RS_TOOLS — nodriver 重构实验

面向 AI 的小红书业务工具：通过 MCP 调用确定的浏览器流程，完成登录、图文草稿创建与结果核实。

本分支已移除 Playwright、Selenium、undetected-chromedriver 执行路径。浏览器只采用 **nodriver + Chrome DevTools Protocol（CDP）**。Rust 保留已有业务 API；图文创作流程在 Python 浏览器服务内执行，不经过 Rust 签名接口。

## 已验证的范围

2026-11-15 在 Intel macOS、真实 Chrome、实际小红书账号上完成：

- 交互登录，平台身份接口确认用户身份。
- 通过真实 stdio MCP 客户端上传一张图片、填写中文标题和正文。
- 点击“暂存离开”，在草稿箱中确认标题存在。
- 关闭并重新启动 Chrome，重新找到该草稿；重开编辑页后图片、标题和正文仍可读取。
- 相同任务重复调用返回记录，不重复上传和保存。
- `explore` 与 `creator` 两个访客 Cookie 获取入口实际成功。

这是**图文草稿闭环**，当前没有公开发布工具；视频、多账号、批量任务、跨设备草稿和原有全部业务 API 尚未完成本轮验证。该结果也不能证明长期免受平台风控。

详见 [架构与验收记录](doc/nodriver_validation.md)。

## 本地启动

需要 Python 3.12、Chrome/Chromium 和 Rust 1.88（仓库的 `rust-toolchain.toml` 会选择该版本）。

在仓库根目录运行：

```bash
uv venv --python 3.12
uv pip install --python .venv/bin/python -r scripts/requirements.txt
.venv/bin/python -m scripts.agent_server
```

浏览器服务默认地址为 `http://127.0.0.1:8765`。首次登录使用它打开的专用 Chrome 窗口，完成扫码或短信登录。不会接管日常浏览器资料目录。

如需旧有 Rust API，另开终端：

```bash
XHS_AGENT_URL=http://127.0.0.1:8765 cargo run
```

Rust API 的 Swagger 为 `http://127.0.0.1:3005/swagger-ui/`。也可直接 `cargo run` 自动启动 Python 服务；它会优先选择项目 `.venv`，并复用已运行的本地 Agent。

## 接入 MCP

先启动上述浏览器服务，再在支持 stdio MCP 的客户端中添加以下配置。将路径替换为本机仓库的**绝对路径**：

```json
{
  "mcpServers": {
    "xhs-rs-tools": {
      "command": "/absolute/path/XHS_RS_TOOLS/.venv/bin/python",
      "args": ["-m", "scripts.mcp_server"],
      "cwd": "/absolute/path/XHS_RS_TOOLS",
      "env": {"XHS_AGENT_URL": "http://127.0.0.1:8765"}
    }
  }
}
```

| 工具 | 作用 |
| --- | --- |
| `browser_health` | 检查服务和浏览器后端 |
| `start_login` | 打开/恢复 `creator` 或 `explore` 会话 |
| `get_login_status` | 向平台确认身份并保存本地凭据；不返回 Cookie |
| `get_browser_view` | 当前页面摘要与本地截图路径 |
| `create_image_note_draft` | 上传图片、填写内容、暂存草稿、核实草稿标题 |
| `get_draft_task` | 读取持久化任务记录 |
| `verify_saved_draft` | 重新打开草稿箱核实已有任务 |
| `close_browser_session` | 关闭 Chrome，保留资料目录和草稿 |

示例给 AI 的请求：

> 打开 creator 登录会话，等待我完成登录并验证身份。使用 uploads 中的图片，创建标题为“我的测试草稿”的图文草稿，正文为“这是一条测试内容”，request_id 使用 demo-001。保存后核实结果。

图片必须位于 `XHS_UPLOAD_DIR`（默认仓库的 `uploads/`）中。MCP 使用服务器端的图片绝对路径。标题最多 20 字符，正文最多 1000 字符，1–18 张 PNG/JPEG/WebP 图片，每张最多 32 MB。图片数量上限与多图上传代码已实现，本轮实际站点只验证一张图片。

## 重跑真实验收

先生成一张纯色测试图：

```bash
.venv/bin/python -m scripts.create_test_image
```

通过真实 MCP 协议创建草稿、重启 Chrome 并验证：

```bash
.venv/bin/python -m scripts.verify_draft \
  --request-id demo-001 \
  --image uploads/nodriver-validation.png \
  --restart
```

只检查本机已经验证过的草稿，不生成新草稿：

```bash
.venv/bin/python -m scripts.verify_draft \
  --request-id nodriver-mcp-20261115-v3 \
  --verify-only --restart
```

草稿箱核实时会明确切换到图文分类，等待本地数据加载；找不到时返回 `missing`，不会继续报告成功。

任务相同且内容相同会返回已有记录；同一个 ID 配不同内容会返回 `idempotency_conflict`。保存后超时或进程在执行中重启会标记 `uncertain`，不会自动重复操作。已有编辑内容时返回 `editor_busy`，防止覆盖尚未保存的内容。

## 状态与持久化

- `.local/profiles/creator/`：登录状态、Local Storage、IndexedDB 和小红书本地草稿。
- `.local/profiles/explore/`：普通站点登录资料。
- `.local/tasks/`：任务步骤、结果、预览和核实截图。
- `cookie.json` / `cookie-creator.json`：供 Rust 业务 API 使用的凭据。

**草稿不是云端笔记。删除对应浏览器资料目录会丢失草稿。** Cookie JSON 不能代替 IndexedDB 备份。每个目标当前使用一个专用资料目录，不支持同时切换多个账号。空闲 10 分钟后关闭 Chrome，资料仍保留；检测到打开的图文编辑器时不会自动回收。显式关闭工具也会拒绝丢弃打开的编辑器。

上述本地数据和上传文件都已加入 Git 忽略规则。

## 配置

| 环境变量 | 默认值 | 说明 |
| --- | --- | --- |
| `XHS_BROWSER_HEADLESS` | `0` | `1` 使用无界面 Chrome；首次交互登录建议有界面 |
| `XHS_BROWSER_EXECUTABLE` | 自动发现 | Chrome/Chromium 可执行文件路径 |
| `XHS_BROWSER_PROXY` | 不额外设置 | 显式浏览器代理，不自动复用包安装代理 |
| `XHS_BROWSER_TIMEOUT` | `25` | 单次浏览器操作超时，秒 |
| `XHS_DATA_DIR` | 仓库 `.local` | 浏览器资料、任务和截图目录 |
| `XHS_UPLOAD_DIR` | 仓库 `uploads` | 允许上传的文件目录 |
| `XHS_AGENT_URL` | `http://127.0.0.1:8765` | MCP/Rust 连接的浏览器服务地址 |
| `XHS_PYTHON` | 项目 `.venv` 优先 | Rust 自动启动 Agent 时使用的 Python |

## 检查

```bash
.venv/bin/python -m unittest discover -s tests -v
XHS_LIVE_BROWSER_TEST=1 .venv/bin/python -m unittest discover -s tests -p test_browser_live.py -v
cargo check --locked --all-targets
```

本地真实浏览器测试只访问临时的本机页面，覆盖上传、中文输入、关闭的 Shadow DOM 按钮、Cookie 写入以及重启后的持久化。实际小红书验收使用上面的 `scripts.verify_draft`。

Docker 配置已改为 Chromium + nodriver，尚未在本机验证容器运行。原有分页 API 说明仍在 `doc/` 中。

## 致谢与许可

浏览器生命周期和业务层分离的方向参考了 [stealth-browser-mcp](https://github.com/vibheksoni/stealth-browser-mcp)。本项目直接依赖 [nodriver](https://github.com/ultrafunkamsterdam/nodriver)，保留 [xhshow](https://github.com/Cloxl/xhshow) 签名服务用于已有 Rust API。MIT 许可见 [LICENSE](LICENSE)。
