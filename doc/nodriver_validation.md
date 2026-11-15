# nodriver 架构与实际验收

## 架构

```text
AI / MCP client
    ↓ stdio
scripts/mcp_server.py（业务工具）
    ↓ loopback HTTP
scripts/agent_server.py
    ├─ LoginSessions：专用资料目录、登录验证、空闲回收
    ├─ DraftWorkflows：上传、输入、保存、任务幂等和结果核实
    ├─ BrowserService：现有访客/同步 Cookie 接口
    └─ NodriverBackend：进程生命周期 + CDP 操作
          ↓
        Chrome

原有 Rust HTTP API → Agent 的签名与 Cookie 接口
```

只有 Python Agent 拥有浏览器。MCP 不启动第二套浏览器，也不让模型拼接任意 JS、CDP 命令或逐步选择页面按钮。每个业务任务由程序按固定步骤执行。

nodriver 用于浏览器连接、发现与生命周期接入；CDP 操作采用有超时的连接，并重新发现当前页面。DOM 节点 ID 只在同一条 CDP 连接内使用。Chrome 进程 stdout/stderr 不进入 MCP stdio。

## 实测中发现并修复的问题

1. 登录后目标页面被替换，旧 Tab 引用会挂起：每次操作重新发现当前页面，读取设置超时，导航时只对读取做重试。
2. nodriver 0.47 的 `CookieJar.set_all` 覆盖输入参数：直接通过 `Storage.setCookies` 写入并做实际读取验证。
3. 首页图文入口携带 `openFilePicker=true`，会留下 macOS 文件选择框：使用不带该参数的已确认上传地址，直接设置文件 input。
4. “暂存离开”位于关闭的 Shadow DOM：用 CDP 的 DOM 树精确匹配按钮文字，通过鼠标事件点击；不选择相邻的“发布”按钮。
5. 草稿保存在浏览器本地：使用固定的专用 Chrome 资料目录，正常关闭后再重启验证，不能只保存 Cookie。
6. 缺少 Cookie 仍返回成功：改为等待必需 Cookie，否则失败。
7. Rust 自动启动 Agent 的输出管道无人消费：改为继承日志流，并等待健康检查，而非固定睡眠后假定成功。

8. 草稿箱默认视频分类，数据异步加载会产生“草稿丢失”误判：复验明确选择图文分类，并等待目标条目；缺失时返回 `missing`。最终复验已能找到较早和最新的两条测试草稿。
9. 打开编辑器时不做空闲回收，关闭工具返回 `editor_busy`，避免丢弃人工正在编辑的内容。

## 实际站点验收：2026-11-15（北京时间）

环境：Intel macOS 15.8.1、已安装 Google Chrome、Python 3.12.4、nodriver 0.47.0、MCP Python SDK 1.30.0。

通过真实 `ClientSession` 与 stdio 子进程执行：

| 步骤 | 结果 |
| --- | --- |
| MCP 初始化、发现 8 个业务工具 | 成功 |
| 人工登录后的平台身份验证 | 成功 |
| MCP 创建图文草稿：1 张 900×1200 PNG、中文标题和正文 | 成功 |
| 页面“暂存离开”与草稿列表核实 | 成功 |
| 关闭 Chrome，保留专用资料目录 | 成功 |
| 新 Chrome 进程恢复身份、重新打开草稿箱 | 成功 |
| 重复调用同一 request_id | 返回已有记录，无重复创建 |
| 重开草稿编辑页读取标题、正文、图片解码尺寸 | 内容一致，图片 naturalWidth=900 / naturalHeight=1200 |
| 普通站点与创作者站点访客 Cookie 获取 | 均成功 |

最终复验任务 ID：`nodriver-mcp-20261115-v3`（较早任务为 `nodriver-mcp-20261115-v2`）；标题：`nodriver重构验证1115`。

本机证据（不提交 Git，均位于 `.local/`）：

- `mcp-validation.json`：MCP 调用与返回记录。
- `tasks/nodriver-mcp-20261115-v3.json`：任务状态。
- `tasks/nodriver-mcp-20261115-v3-preview.png`：保存前编辑预览。
- `tasks/nodriver-mcp-20261115-v3-verified.png`：关闭重启后的草稿箱。
- `tasks/persistent-draft-editor.png`：恢复草稿后的编辑页面。
- `persistent-editor-check.json`：恢复后的标题、正文和图片尺寸读取。

首轮临时资料目录原型草稿在退出后被清除，随后切换为持久化目录并重新执行上表中的成功任务。早期导航失败任务仍作为调试记录保留，不计为成功。

## 当前边界

- 本轮只验收了图文草稿创建，未实现公开发布工具，未测试视频发布、定时发布、批量和多账号。
- 自动核实草稿列表目前按完整标题匹配，没有可靠的站点草稿 ID；同名草稿的精确区分需要后续增加。
- 任务 `saved` 是最近一次核实记录，不意味着用户手动删除草稿后记录会自动更新。
- 如果签名算法、平台接口、页面结构或账号状态变化，需要重新验证；本次成功不能证明自动化检测已被普遍解决。
- 当前平台草稿为本地资料，跨机器迁移需要保留整个专用 Chrome 资料目录。
- Docker 配置已更新，但容器运行未验收。原有 Rust API 仅完成编译检查，未逐个回归。

## 后续优先级

1. 用站点草稿标识替代标题核实，覆盖多图和更多文本格式。
2. 完善草稿预览到公开发布的任务状态、发布结果查询及避免重复发布。
3. 在已有真实用例基础上扩大浏览器与平台版本回归。

## 自动检查

单元与本地真实 Chrome 测试合计 16 项；Rust 使用 `cargo check --locked --all-targets` 检查。真实小红书验收与本地测试分开记录，模拟测试不计为实际站点成功。
