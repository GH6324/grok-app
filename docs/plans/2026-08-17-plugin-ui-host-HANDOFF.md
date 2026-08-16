# Handoff — 通用 Plugin UI Host（Host P0）

> **文档性质**：联审纪要 + 开工交接  
> **状态**：工作树已开，契约 WP-A 待新会话执行  
> **分支**：`feat/plugin-ui-host`  
> **工作树**：`~/.grok/worktrees/grok-app/plugin-ui-host`  
> **基线**：`main` @ `4be09f1a`（`chore: sync Cargo.lock package version to 0.2.20`）+ 设计稿 `894f1d5a`  
> **产品设计**：`docs/plans/2026-08-17-x-creator-plugin-workbench.md`  
> **施工 SoT**：`docs/plans/2026-08-17-plugin-ui-host-GOAL.md`  
> **联审**：pi 只读审查（2026-08-17），Verdict **FLAG**（方向对，原文有 4 处不改就会做错）

本期只做 **通用宿主 + 夹具**。不做 X 博主工作台业务 UI / Skill / 付费墙，不做 Host P1。

---

## 1. 为什么要先改设计再写代码

原文（设计 §8.2 / §9 / §6 / §12）有四处按字面实现会失败或出洞：

| 原文 | 问题 | 锁定 |
|------|------|------|
| CSP `frame-ancestors 'self'` | iframe 源是 `127.0.0.1:pluginPort`，宿主是 `tauri://localhost` / `http://localhost:1421`，`'self'` 会拒嵌 | D2：白名单宿主 origin |
| plugin-ui 容易被接到 `media_server` | 同 origin 下 iframe 能打 `/v1/media`；`<img>` 无 Origin 时 media 会放行 | D1：独立端口 + 独立 token |
| `sessionCreate` + `sessionSend` | 漏 `sessionConnect`；Workbench 现有 create 会改 `viewingSessionId`；权限条只绑正在看的会话 | D6：学 `automation_runner.rs` 后台跑 |
| `mainPane` 改对象联合 | `AppWorkbench.tsx` 约 25k 行、30+ 处 `=== "chat"` | D7：字符串 `"plugin"` + `pluginRoute` |

完整审查见本文件 §4 与 GOAL 决策表。

---

## 2. 范围

**做**

1. 读 `grok-app-extension.json` → 侧栏贡献列表 + `#/plugin/{id}/{pane}`
2. 独立 plugin-ui loopback + iframe（token、`ui/` 前缀、CSP）
3. Token 注入 + 同源 `/plugin-host/chrome.css` + `/plugin-host/host-client.js`
4. SDK P0：`getInfo` / `sessions.compose` / `sessions.run` / `poll` / `dialog` / `storage` / `toast` / `clipboard.write`
5. 事件：`host.ready` / `session.started` / `needsUser` / `done`
6. 安装 / 启用 / 禁用 / 卸载摘挂
7. 推荐表从「写死 ChatCut」改成可多项的 marketplace 元数据表（本期仍只挂 ChatCut 一行；不发明 X git URL）
8. 夹具 `fixtures/plugin-ui-hello/` 验收

**不做**

- license / automations SDK / menu / picker / media.proxy / vite dev / nav 超额降 `more`
- X 业务组件、权重表、资格条、Skill 正文、付费墙
- 改 `App.tsx` 加状态、改 `media_server.rs`、写 `~/.grok`、另造 `~/.grok-app/plugins` 商店

---

## 3. 多 Agent 怎么一起跑

```
WP-A 契约  ──必须先合──┐
                      ├─ WP-B Rust 发现 + plugin-ui server
                      ├─ WP-C 推荐表多项
                      ├─ WP-D iframe Host + bridge + Provider
                      └─ WP-F 夹具 + chrome.css / host-client.js
WP-B + WP-D 绿 ──最后── WP-E AppWorkbench 接线
```

- **同一工作树、同一分支**。禁止各 Agent 再开新 worktree。
- 写路径互斥，见 GOAL §5。冲突热点 `AppWorkbench.tsx` 只许 WP-E；`lib.rs` 只许 WP-B；`pluginHost` 契约只许 WP-A。
- 并行 Agent 开工前必须 `git pull`（或 rebase）到已含 WP-A commit 的 `feat/plugin-ui-host`。

---

## 4. pi 审查摘要

**Verdict：FLAG**

宿主模型（贡献点是数据、画面在插件静态页、App 只发现/隔离/桥接）方向对，和 CLI 插件真相源、ChatCut 推荐先例、已安排后台会话对齐。按原文直接并行开干会做错。

安全与实现要点：

- 推荐表可以有名字 / 一句描述 / git URL / 已装匹配；禁止未安装时出现侧栏 title、权重、深链。
- plugin-ui 路径必须 `canonical({plugin.path}/ui)` 前缀，禁止复用 `path_scope`（那会放行整个项目和 `~/.grok`）。
- hash `{plugin}` 用清单 `id`，不用 CLI `name`。
- 清单只认插件根目录 `grok-app-extension.json`，不放进 `.grok-plugin/`。
- P0 权限白名单收口：未实现的 P1 名声明即整份拒绝。
- 插件数据目录 = `{app_data_root()}/plugin-data/{pluginId}/`，不要字面量写 `~/.grok-app/...`。
- `host-client.js` / `chrome.css` 由 plugin-ui 服务器同源提供；token 只走 postMessage。
- 信任文案 `ext.market.installTrustNote` 要补「可能向侧栏加入面板」。
- 副窗强制 `mainPane="chat"`；plugin hash 只属于主窗。

---

## 5. 进度

见 `2026-08-17-plugin-ui-host-PROGRESS.md`。

| WP | 状态 | 谁 |
|----|------|-----|
| 工作树 / 分支 / 文档 | DONE | 主会话 + pi |
| A 契约 | 待执行 | Agent A |
| B Rust server | 待 A 合入 | Agent B |
| C 推荐表 | 可与 A 并行 | Agent C |
| D Host UI | 待 A 合入 | Agent D |
| F 夹具 | 待 A 合入（真机要 B） | Agent F |
| E Workbench | 待 B+D | Agent E |

---

## 6. 启动

把工作树设为会话 cwd，按 GOAL §7 / §8 粘贴对应 `/goal`。不要在主仓 `grok-app` 的 `main` 上改产品代码。
