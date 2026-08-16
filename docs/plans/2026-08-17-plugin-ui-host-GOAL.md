# Goal 规格 — 通用 Plugin UI Host（Host P0 + 夹具）

> **文档性质**：施工单一事实来源 + 可复制多 Agent `/goal` + 验收合同  
> **状态**：工作树已开，待按 WP 并行执行  
> **分支**：`feat/plugin-ui-host`  
> **工作树**：`~/.grok/worktrees/grok-app/plugin-ui-host`  
> **基线**：`main` @ `4be09f1a`  
> **产品设计**：`docs/plans/2026-08-17-x-creator-plugin-workbench.md`  
> **交接**：`docs/plans/2026-08-17-plugin-ui-host-HANDOFF.md`  
> **联审**：pi 只读审查，Verdict FLAG；执行以本文 D1–D10 为准，**不要按设计原文的 CSP / 同 server / 对象联合 mainPane 实现**

本期 = 通用宿主 + 夹具。X 博主工作台业务另开仓，不在本分支。

---

## 0. 决策锁定（不得擅自改）

| ID | 决策 |
|----|------|
| D1 | 独立 plugin-ui axum：`127.0.0.1:0` 新端口、新 token。禁止往 `media_server.rs` / `session_api.rs` 加路由 |
| D2 | CSP `frame-ancestors` = 宿主 origin 白名单（与 media_server 同一族：`tauri://localhost`、`https://tauri.localhost`、`http://localhost:1421` 等），不是 `'self'` |
| D3 | 路由键 / hash / iframe 路径用清单 `id`，不用 CLI `name` |
| D4 | 清单只认插件根目录 `grok-app-extension.json`。禁止放进 `.grok-plugin/`，禁止把 App 字段写入 `plugin.json` |
| D5 | P0 权限白名单 = `sessions.create` / `sessions.read` / `storage` / `dialog` / `toast` / `clipboard.write`。P1 名（license / automations.* / menu / picker / media.proxy / account.read / catalog.read / open）本轮声明即整份拒绝 |
| D6 | `run` = Host 侧 `create_session` + `connect` + `send_message(sessionId)`，学 `automation_runner.rs`。不改 `viewingSessionId`，不 `setMainPane("chat")`。`compose` P0 只做 `target:"new"`：建会话 + 写 draft，不发送、不切主栏 |
| D7 | `mainPane` 保持字符串 `"chat" \| "automations" \| "plugin"`，另存 `pluginRoute: {plugin, pane} \| null`。切走聊天时 iframe `display:none` 隐藏不卸载 |
| D8 | 插件数据 = `{app_data_root()}/plugin-data/{pluginId}/`。P0 `session.done` 以截断助手正文 32 KiB 为准，job JSON 尽力而为 |
| D9 | 推荐表只允许 marketplace 元数据：id + 展示名 + 一句描述 + git URL + 已装匹配。本期仍只挂 ChatCut。夹具标题不得进 i18n / 推荐表 / 生产 bundle |
| D10 | 契约文件只许 WP-A 写。其它 Agent 不得私自发明 invoke 名、hash、权限字符串 |
| D11 | `App.tsx` 冻结。不向 `App.tsx` 加 `useState` / 大块逻辑 |
| D12 | i18n 走 `createT`。宿主空态/错误用新域 `pluginHost.*`。对话框规范见 `docs/llm-wiki/dialogs.md` |
| D13 | Host P1 与 X 业务整列不做 |

---

## 1. Outcome（完成时必须为真）

1. 未安装夹具时：侧栏没有该项；生产 bundle / `src` 搜不到夹具专用标题。
2. 安装并启用夹具后：侧栏出现清单 `title`；主区 iframe 无框、token 与「已安排」同色家族；`host.dialog.notice` 弹出 App `GlassModal`（不是 `window.alert`）。
3. `host.sessions.compose` 建出新会话并填 draft，不发送，不切走工作台。
4. `host.sessions.run` 后台跑短 Skill / 短 prompt：iframe 收到 `session.started` 与 `session.done`；用户当前闲聊 `viewingSessionId` 不变。
5. 禁用 / 卸载：侧栏项消失，hash 回到 `#/workbench`。
6. 清单声明不在 D5 白名单的 permission：整份贡献拒绝，设置页 warn。
7. 推荐区是可多项的数据表，ChatCut 仍在；安装确认文案写明第三方以 Agent 权限跑，并可能向侧栏加入面板。
8. `pnpm typecheck` 与本改动相关 vitest / cargo test 绿。无 `App.tsx` 膨胀。

---

## 2. 依赖图

```
A 契约 ──先合──► B Rust server
              ├► D Host UI / bridge / Provider
              └► F 夹具静态文件（真机验收要 B）
C 推荐表 可与 A 同天并行
B + D 绿 ──最后──► E AppWorkbench 接线
```

同一工作树、同一分支。并行前必须 rebase 到已含 WP-A 的 `feat/plugin-ui-host`。

---

## 3. 施工工作包

### WP-A — 契约先行（先合，阻塞 B/D/F）

**目标**：类型、hash、权限表、清单校验纯函数落地，其它 Agent 只 import。

**步骤**

1. 新建 `src/lib/pluginHost/`：`types.ts` `hash.ts` `permissions.ts` `manifest.ts` 及对应 `*.test.ts`
2. `parsePluginHash` / `buildPluginHash`：`#/plugin/{id}/{pane}`；拒绝 `settings` / `session` / `automations` / `workbench` 冲突
3. `P0_PERMISSIONS` 按 D5；未知或 P1 名 → 整份清单 invalid
4. `parseExtensionManifest`：`schemaVersion===1`、`id` / `sidebar[].id` 正则、`entry`/`icon` 必须相对且只许 `ui/`、拒 `..`
5. 可选短文 `docs/plans/plugin-ui-host-schema.md` 列出 invoke 名与 postMessage envelope，供 B/D 对齐

**验收**

- `pnpm exec vitest run src/lib/pluginHost`
- hash 往返；未知 permission 拒绝；路径逃逸拒绝

**禁止**：改 `AppWorkbench.tsx` `App.tsx` `lib.rs` `media_server.rs` `ExtensionsPanel.tsx` i18n

---

### WP-B — Rust 发现 + plugin-ui server（可与 C/D/F 并行）

**依赖**：A 已合（权限字符串与清单字段对齐）

**步骤**

1. 新文件 `src-tauri/src/plugin_contributions.rs`：对 `plugins_list` 里 `enabled && path` 读根目录 `grok-app-extension.json`，校验同 A
2. 新文件 `src-tauri/src/plugin_ui_server.rs`：独立 `127.0.0.1:0`；`GET /plugin-ui/{id}/{sessionToken}/{rel}`；`GET /plugin-host/chrome.css` `GET /plugin-host/host-client.js`
3. rel 必须落在 `canonical(plugin.path/ui)` 下；`..` / symlink 逃出 / `skills/` / `.env` → 404/403
4. 每插件一份进程内随机 token（24 字节级）；跨插件 token → 401；禁日志打全 URL
5. CSP：D2 origin 白名单；`script-src 'self'`；`connect-src 'self'`；不要 `allow-modals`
6. `lib.rs` **只**加 `mod` + `start()` + 注册 command（建议 `plugin_contributions_list`、`plugin_ui_endpoint`、可选 `plugin_host_warns`）
7. `paths.rs` / `ensure_app_dirs` 增加 `plugin-data` 目录

**验收**

- `cd src-tauri && cargo test plugin_contrib -- --nocapture`
- `cd src-tauri && cargo test plugin_ui -- --nocapture`
- 覆盖：路径逃逸、未知权限、跨插件 token、非 `ui/` 404

**禁止**：改 `media_server.rs` `session_api.rs` `AppWorkbench.tsx` `ExtensionsPanel.tsx` 前端 i18n

---

### WP-C — 推荐表多项（可与 A 并行）

**目标**：推荐区从写死 ChatCut 变成 `RECOMMENDED_PLUGINS.map`。本期仍只挂 ChatCut，不发明 X git URL。

**步骤**

1. `src/lib/pluginRecommended.ts` 抽出 `RecommendedPlugin` 与列表；保留现有 ChatCut 匹配函数
2. `ExtensionsPanel.tsx` **仅**推荐 section（约 1873–1910）与对应 GlassModal 改成循环
3. 通用安装确认补「可能向侧栏加入面板」（改 `ext.market.installTrustNote` 或通用 recommended confirm，三语同步）
4. `settingsCatalog/entries/extensions.ts` 搜索词保持 ChatCut，不要加夹具标题

**验收**

- `pnpm exec vitest run src/lib/pluginRecommended.test.ts src/lib/settingsCatalog.test.ts`
- `pnpm typecheck`
- ChatCut 安装 / 已装匹配回归仍绿

**禁止**：`AppWorkbench.tsx` `lib.rs` `src/lib/pluginHost/**` 三个 `i18n/messages/*/index.ts`；不要加 X 业务文案或夹具标题

---

### WP-D — iframe Host + bridge + Provider（可与 B/C/F 并行）

**依赖**：A 已合

**步骤**

1. `src/providers/PluginContributionsProvider.tsx`：拉 list、听插件变更、暴露贡献数组 + warns
2. `src/components/plugin-host/PluginPaneHost.tsx`：iframe `sandbox="allow-scripts allow-forms allow-same-origin"` `allow=""`；校验 `event.source` 与 loopback origin
3. `src/components/plugin-host/PluginNavItems.tsx`：只渲染 nav 贡献，不写 X 文案
4. `src/lib/api/pluginHost.ts` + `src/lib/pluginHost/bridge.ts` + `jobs.ts`：P0 方法与事件
5. `run` / `compose` 走 D6，**不要**调 `AppWorkbench` 的 `executeSend` / `newChat`
6. 权限 / AskUser：Host 自己听该 job 的 `session://permission` / `ask_user`，用已有 `GlassModal` / `AskUserModal` 盖在 iframe 上。**不要改** `useSessionHostEvents.ts`
7. 新 i18n 域：`src/i18n/messages/{en,zh,zh-TW}/pluginHost.ts`，并在三个 `index.ts` 加一行 import
8. storage 只动 `{app_data}/plugin-data/{pluginId}/`

**验收**

- `pnpm exec vitest run src/lib/pluginHost`
- `pnpm typecheck`
- `pnpm exec vitest run src/i18n/messages.test.ts`（或项目现有 i18n key 对齐测）

**禁止**：改 `AppWorkbench.tsx` `ExtensionsPanel.tsx` `extensions.ts` i18n `lib.rs` `media_server.rs`

---

### WP-E — AppWorkbench 接线（最后，单独一人）

**依赖**：B + D 绿

**步骤**

1. **只改** `src/app/AppWorkbench.tsx`（必要时 `src/styles/workbench*.css` 给 plugin 主区 `overflow: hidden`）
2. `mainPane` 加 `"plugin"`；增加 `pluginRoute` state
3. `syncFromHash` 识别 `#/plugin/{id}/{pane}`；禁用/卸载写回 `#/workbench`
4. 侧栏在「连接设备」下方 map `PluginNavItems`
5. 主区：`mainPane==="plugin"` 时渲染一个 `PluginPaneHost`；切走则 CSS 隐藏不卸载
6. 现有 `mainPane==="chat"` 分支保持：composer / 权限条 / 空态不在 plugin 主区冒出来
7. 副窗保持 `mainPane="chat"`，忽略 plugin hash
8. Provider 挂在 Workbench 内部，不进 `App.tsx`

**验收**

- `pnpm typecheck`
- 手测：`#/plugin/{id}/{pane}`、`#/settings`、`#/session/{id}`、`#/automations`、`#/workbench` 不互吞

**禁止**：改 `lib.rs` `ExtensionsPanel.tsx` `pluginRecommended.ts` `src/i18n/messages/en/extensions.ts`；禁止在 Workbench 里新写业务页面

---

### WP-F — 夹具 + 同源壳（可与 B/C/D 并行）

**依赖**：A 的清单字段；真机要 B 挂静态路由

**步骤**

1. `fixtures/plugin-ui-hello/`：根目录 `grok-app-extension.json` + `.grok-plugin/plugin.json` + `ui/index.html` + 外链 JS（`script-src 'self'` 禁止 inline）
2. 夹具标题用独特英文串 `Plugin UI Hello`（不要进 i18n）
3. 按钮：`host.dialog.notice` → `host.sessions.compose` → `host.sessions.run` 短 prompt
4. 静态壳：`src-tauri/plugin_host_static/chrome.css` 与 `host-client.js`（B 挂路由，F 写文件；若 B 未合则先放文件）
5. 夹具 html/body 透明；用 token，不写死 `#111`

**验收**

- 未安装：侧栏无此项；`rg "Plugin UI Hello" src src/i18n` 为空（fixture 目录除外）
- 安装启用：侧栏 title 来自清单；dialog / compose / run 如 Outcome
- 禁用回 `#/workbench`
- 未知 permission 夹具变体：无侧栏 + 设置 warn

**禁止**：任何 `src/i18n/**`；推荐表；`AppWorkbench.tsx`；X 文案

---

## 4. 验收合同

| 编号 | 项 | 如何证明 |
|------|----|----------|
| V1 | 未安装零业务 | 侧栏无夹具项；src/i18n 无夹具标题 |
| V2 | 安装启用出现 | 清单 title；`#/plugin/plugin-ui-hello/home` 打开 iframe |
| V3 | 视觉同族 | 无框、透明底、token / chrome.css；禁止夹具写死颜色 |
| V4 | dialog | `host.dialog.notice` → GlassModal |
| V5 | compose | 新会话 + draft；未 send；mainPane 仍 plugin |
| V6 | run | `session.done`；`viewingSessionId` 仍是原闲聊 |
| V7 | 摘挂 | 禁用/卸载项消失，hash=`#/workbench` |
| V8 | 未知权限 | 贡献拒绝 + 设置 warn |
| V9 | 路径/token | cargo test：逃逸 403、跨插件 401 |
| V10 | 类型与单测 | `pnpm typecheck`；相关 vitest / cargo 绿 |
| V11 | 推荐表 | ChatCut 仍可装；确认文案含侧栏面板提示 |
| V12 | 冻结 | `App.tsx` 不加状态；无 `window.confirm`；无 X 业务文件 |

---

## 5. 文件地图与写权限

| 路径 | 谁写 |
|------|------|
| `src/lib/pluginHost/types.ts` hash.ts permissions.ts manifest.ts `*.test.ts` | 仅 A |
| `src-tauri/src/plugin_contributions.rs` `plugin_ui_server.rs` | 仅 B |
| `src-tauri/src/lib.rs`（mod + start + 2–3 command） | 仅 B |
| `src-tauri/src/paths.rs`（plugin-data dir） | 仅 B |
| `src/lib/pluginRecommended.ts` `pluginRecommended.test.ts` | 仅 C |
| `src/components/ExtensionsPanel.tsx` 推荐区 | 仅 C |
| `src/i18n/messages/{en,zh,zh-TW}/extensions.ts` | 仅 C |
| `src/lib/settingsCatalog/entries/extensions.ts` | 仅 C |
| `src/providers/PluginContributionsProvider.tsx` | 仅 D |
| `src/components/plugin-host/**` | 仅 D |
| `src/lib/api/pluginHost.ts` | 仅 D |
| `src/lib/pluginHost/bridge.ts` `jobs.ts` | 仅 D |
| `src/i18n/messages/{en,zh,zh-TW}/pluginHost.ts` + 三个 `index.ts` 一行 | 仅 D |
| `src/app/AppWorkbench.tsx` | 仅 E |
| `src/styles/workbench*.css`（仅 overflow） | 仅 E |
| `fixtures/plugin-ui-hello/**` | 仅 F |
| `src-tauri/plugin_host_static/**` | 仅 F（B 只挂路由） |
| `docs/llm-wiki/plugins-marketplace.md` 及新 `docs/llm-wiki/plugin-host.md` | E 或收尾一人，勿并行 |
| `docs/plans/2026-08-17-plugin-ui-host-PROGRESS.md` | 各 WP 只追加自己一节 |

**任何 WP 禁止写**：`src/App.tsx`、`src-tauri/src/media_server.rs`、`src/hooks/useSessionHostEvents.ts`、`~/.grok/**`、X 业务组件。

---

## 6. 关键现网锚点（执行时去读，不要凭记忆改）

| 点 | 位置 |
|----|------|
| `mainPane` | `src/app/AppWorkbench.tsx` ~1742 |
| hash | 同文件 ~4283–4327 |
| 侧栏 nav | 同文件 ~18216–18254 |
| 主区 automations 分支 | 同文件 ~19309 |
| 副窗强制 chat | 同文件 ~3080 |
| 已安排后台 run | `src-tauri/src/automation_runner.rs` ~373–411 |
| media loopback | `src-tauri/src/media_server.rs` |
| 插件 list/install | `src-tauri/src/commands/extensions_p2.rs`、`src/lib/api/extensions.ts` |
| ChatCut 推荐 | `src/lib/pluginRecommended.ts`、`ExtensionsPanel.tsx` ~1873 |
| app data | `src-tauri/src/paths.rs` `app_data_root()` |
| 对话框 | `docs/llm-wiki/dialogs.md`、`GlassModal`、`useAppDialogs.ts` |

---

## 7. 可复制启动 Goal

工作树 cwd 必须是 `~/.grok/worktrees/grok-app/plugin-ui-host`。先跑 A 并 commit，再开 B/C/D/F，最后 E。

### 7.0 编排（一个人串行也可）

```text
/goal 在工作树完成通用 Plugin UI Host P0：夹具可安装后出现在侧栏，iframe 走独立 plugin-ui loopback，compose/run/dialog/storage/toast 按 D1-D13 可用，未安装时安装包零业务。
验证：按 docs/plans/2026-08-17-plugin-ui-host-GOAL.md 的 V1-V12；pnpm typecheck；vitest 跑 src/lib/pluginHost 与 pluginRecommended；cargo test plugin_contrib 与 plugin_ui；夹具手测 compose/run/摘挂。
约束：严格 D1-D13。不要按设计原文的 frame-ancestors self、同 media_server、对象联合 mainPane 实现。不做 Host P1 与 X 业务。
边界：只在 feat/plugin-ui-host 工作树写 GOAL 地图内文件。禁止 App.tsx、media_server.rs、useSessionHostEvents.ts、~/.grok。
迭代策略：严格 A 然后并行 B/C/D/F 然后 E。每 WP 测绿再进下一 WP。同一错误连续失败 2 次必须换证据来源。最多 3 轮聚焦修。
完成条件：V1-V12 有命令或手测证据；PROGRESS 写到各 WP 状态。
暂停条件：需要改 CLI 插件协议、付费/license、自动发帖、或 main 上同文件大冲突时暂停并写 HANDOFF。
```

### 7.1 Agent A — 契约

```text
/goal 在 feat/plugin-ui-host 落地 src/lib/pluginHost 契约：清单解析、P0 权限白名单、hash 往返，供后续 Agent 只 import 不改。
验证：pnpm exec vitest run src/lib/pluginHost 全绿；未知权限拒绝；ui/ 外路径拒绝；hash 与 settings/session/automations 不冲突。
约束：D3 D4 D5 D10。权限集合不可擅自扩大。
边界：只写 src/lib/pluginHost/ 下 types hash permissions manifest 及测试，可选 docs/plans/plugin-ui-host-schema.md。禁止 AppWorkbench、lib.rs、i18n、ExtensionsPanel。
迭代策略：先写类型与失败用例再实现。测红再改。最多 3 轮。
完成条件：测试绿并 commit，PROGRESS 标记 A。
暂停条件：与设计 D5 白名单冲突且无法在不扩大权限下解析示例清单。
```

### 7.2 Agent B — Rust server

```text
/goal 在已含 WP-A 的 feat/plugin-ui-host 实现独立 plugin-ui loopback 与贡献扫描：enabled 插件根目录 grok-app-extension.json，token 隔离，CSP 用宿主 origin 白名单。
验证：cargo test plugin_contrib 与 plugin_ui；覆盖路径逃逸、跨插件 token、未知权限、非 ui/ 404。
约束：D1 D2 D4 D8。不要改 media_server.rs 或 session_api.rs。
边界：新文件 plugin_contributions.rs 与 plugin_ui_server.rs；lib.rs 只加 mod/start/2-3 个 command；paths.rs 只加 plugin-data。禁止前端与 AppWorkbench。
迭代策略：先红测路径与 token，再接线 start。最多 3 轮。
完成条件：cargo 相关测试绿并 commit，PROGRESS 标记 B。
暂停条件：axum 已有绑定策略与独立端口冲突且无法在不改 media_server 下解决。
```

### 7.3 Agent C — 推荐表

```text
/goal 把设置扩展页推荐区从写死 ChatCut 改成可多项 RecommendedPlugin 列表，本期仍只挂 ChatCut，确认文案补上可能向侧栏加入面板。
验证：pnpm exec vitest run src/lib/pluginRecommended.test.ts src/lib/settingsCatalog.test.ts；pnpm typecheck；ChatCut 已装匹配与安装源不变。
约束：D9。不发明 X git URL，不把夹具写进推荐表。
边界：pluginRecommended.ts 及其测试、ExtensionsPanel 推荐 section 与对应 GlassModal、extensions 三语文案、settingsCatalog entries/extensions.ts。禁止 AppWorkbench、pluginHost、三个 i18n index.ts。
迭代策略：先抽类型与单测，再改面板循环。最多 3 轮。
完成条件：测试绿并 commit，PROGRESS 标记 C。
暂停条件：安装确认文案产品措辞需要人工拍板且现有 trust note 无法扩一句时暂停。
```

### 7.4 Agent D — Host UI

```text
/goal 实现 PluginContributionsProvider、PluginPaneHost iframe、hostBridge P0 方法与 pluginHost i18n，不接线 AppWorkbench。
验证：pnpm typecheck；vitest 跑 src/lib/pluginHost；i18n key 三语对齐。compose 不 send；run 的单元/契约测证明不碰 viewingSessionId。
约束：D5 D6 D8 D12。不要改 useSessionHostEvents.ts。P1 方法不要做。
边界：providers/PluginContributionsProvider.tsx、components/plugin-host/、lib/api/pluginHost.ts、pluginHost/bridge.ts 与 jobs.ts、i18n pluginHost.ts 加三个 index 一行。禁止 AppWorkbench、ExtensionsPanel、lib.rs、media_server。
迭代策略：先握手 getInfo/ready，再 dialog/storage/toast，最后 compose/run/poll。每步补测。最多 3 轮。
完成条件：类型与单测绿并 commit，PROGRESS 标记 D。
暂停条件：sessionCreate/connect/send 在不改 SessionManager 公开 API 时无法后台跑，写清缺口到 HANDOFF。
```

### 7.5 Agent E — Workbench 接线

```text
/goal 仅在 AppWorkbench 把贡献列表、hash、侧栏、一个 PluginPaneHost 接上，mainPane 增加 plugin 字符串与 pluginRoute，iframe 隐藏不卸载。
验证：pnpm typecheck；手测 plugin/settings/session/automations/workbench hash 不互吞；副窗仍是 chat。
约束：D7 D11。禁止把 job/bridge/X 页面写进 Workbench。
边界：只改 src/app/AppWorkbench.tsx，必要时 workbench CSS 的 overflow。禁止 lib.rs、ExtensionsPanel、pluginRecommended、extensions i18n。
迭代策略：先 hash 与类型，再侧栏，再主区。每步 typecheck。最多 3 轮。
完成条件：typecheck 绿、hash 手测记录写进 PROGRESS，并 commit。
暂停条件：必须大拆 AppWorkbench 才能接线时暂停，不要趁机重构 25k 文件。
```

### 7.6 Agent F — 夹具

```text
/goal 新增 fixtures/plugin-ui-hello 夹具与 plugin_host_static 的 chrome.css、host-client.js，标题 Plugin UI Hello 不进 i18n 与推荐表。
验证：运行 parseExtensionManifest 测试接受夹具清单文件；检查 html 无 inline script；运行 rg 确认 src 与 i18n 无 Plugin UI Hello；本地安装后打开侧栏，手测 GOAL V2-V7。
约束：D2 D9。透明底，不用写死颜色。script-src self。
边界：只写 fixtures/plugin-ui-hello/ 与 src-tauri/plugin_host_static/。禁止 i18n、推荐表、AppWorkbench、X 文案。
迭代策略：先清单与静态页，再对 host-client API。最多 3 轮。
完成条件：文件齐、PROGRESS 标记 F，并 commit。
暂停条件：B 尚未挂 /plugin-host 静态路由导致无法真机验时，仍提交静态文件并在 PROGRESS 注明待 B。
```

---

## 8. 启动命令（人类）

工作树已经建好。不要再 `git worktree add`。

```bash
# 进入施工树（所有 Agent 的 cwd）
cd ~/.grok/worktrees/grok-app/plugin-ui-host
git status -sb
# 应显示：## feat/plugin-ui-host
```

在 Grok App / CLI 里把该目录设为项目 cwd，然后：

1. **先开 1 个会话**，粘贴 §7.1（Agent A）。等它 commit。
2. `git log -1 --oneline` 确认 A 在。
3. **再开 4 个会话**（同一 cwd），分别粘贴 §7.2 B、§7.3 C、§7.4 D、§7.6 F。C 也可与 A 同时开。
4. B 与 D 都绿后，**开第 6 个会话** 粘贴 §7.5 E。
5. 用夹具走一遍 V1–V8，把证据写进 PROGRESS。

只想一个人干完，粘贴 §7.0。

可选：从主仓核对工作树

```bash
git -C /Users/ronglecat/Documents/self/tools/desktop-app/grok-app worktree list
```

合入 main 之后再删树（现在不要删）：

```bash
# 全部合完后才执行
# cd /Users/ronglecat/Documents/self/tools/desktop-app/grok-app
# git worktree remove ~/.grok/worktrees/grok-app/plugin-ui-host
```

---

## 9. wiki（收尾，勿并行抢）

E 或最后一人更新：

- `docs/llm-wiki/plugins-marketplace.md`：推荐表可多项；安装确认含侧栏面板
- 新建 `docs/llm-wiki/plugin-host.md`：发现、iframe、P0 SDK、夹具怎么装
- `docs/llm-wiki/README.md` 加一条索引
- 不要在 wiki 写 X 业务细节（仍指向设计稿）
