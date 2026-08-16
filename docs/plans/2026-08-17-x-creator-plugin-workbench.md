# X 博主工作台 · 插件宿主设计

| 字段 | 值 |
|------|-----|
| 日期 | 2026-08-17 |
| 状态 | 宿主 P0 已立项。pi 联审 FLAG；执行以 `2026-08-17-plugin-ui-host-GOAL.md` 的 D1–D10 为准 |
| 施工 | `docs/plans/2026-08-17-plugin-ui-host-GOAL.md` · `HANDOFF.md` · `PROGRESS.md` |
| 工作树 | `~/.grok/worktrees/grok-app/plugin-ui-host` · 分支 `feat/plugin-ui-host` |
| 产品 | 付费插件「X 博主工作台」：可视化面板 + 预置 Skill |
| 入口 | 设置 → 扩展 → 插件 → **推荐** 安装；安装并启用后侧栏才出现该项 |
| 商业 | App 开源（MIT）+ 独立付费插件仓 |
| 算法源 | [xai-org/x-algorithm](https://github.com/xai-org/x-algorithm)（2026-08-13/14 生产权重与可见性已开源） |

本文是单一事实来源。覆盖：需求从哪来、算法怎么用、功能怎么分期、**安装包为何不含业务 UI**、iframe 怎么和 App 往来、能开放/能回传什么、收费怎么挂。

---

## 0. 一句话

第一期不要做成 Typefully 的中文平替。做成：

> 对照开源 For You 权重的「写 / 检 / 归因」工作台。画面和文案全部活在插件里；App 只做一次通用宿主。没装插件时，安装包里看不到、也加载不到这项业务。

北极星（茄哥）：「核心就是怎么创作出爆款帖子，然后围绕这个去做适用性的改造。」Skill 必须预置好——「很多人不会改 Skill」。

---

## 1. 不可回退的约束

1. **未安装 = 安装包零业务。** 没有工作台 React、没有 X 专用 i18n、没有算法权重表、没有 Skill 正文、没有 `#/plugin/x-creator-…` 可用深链。侧栏没有那一项，不是 `if (installed)` 把预写组件藏起来。
2. **App 只实现一次通用宿主。** 下一个垂直插件走同一份契约，不必再改 App 业务组件。
3. **CLI 仍是插件真相源。** `grok plugin install / enable / uninstall`。不另造 `~/.grok-app/plugins` 商店。
4. **插件 UI 不准拿 Tauri IPC。** 只能打声明过的 Host SDK。
5. **安装须 GlassModal 确认 + `--trust`，从不自动装。** 确认文案写明：第三方代码以 Agent 权限跑，并可能向侧栏加入面板。
6. **不代发 X、不自动互动、不教刷 Qualified impressions。** 默认只出草稿，人去浏览器发。

和 wiki 里作废的「panel-host 试验」不是一回事。旧方向是往 App 里预埋业务面板。本设计是 **VS Code Webview 模型**：贡献点是数据，画面是插件静态页，宿主只负责发现、隔离、桥接。

---

## 2. 需求从哪来

参考目录 `/Users/ronglecat/Downloads/X 博主工作台/` 只有聊天记录和两张截图。帖子与视频另行拉取。

### 2.1 茄哥聊天（2026-08-16）

| 原话 | 产品含义 |
|------|----------|
| 社媒创作专属版，在内置 Skill 下功夫 | 垂直包，不是通用 Agent |
| Skill 让 Build 写，但很多人不会改 | 必须预置、可点、可视化 |
| 引用 / 回复；搜索调研；Imagine 配图转视频 | 创作闭环 |
| 抓热门、抓粉丝暴涨账号 | 热帖雷达 + 暴涨号雷达 |
| 最准确、最低消耗 | 快模型预检，Heavy 终稿 |
| 文案降权检测 | 对照 Phoenix 规则的可贴改写 |
| Grok 查爆款、暴涨归因 | 尖峰归因 |
| 最近刚需是上主页推荐 | 90 天 Verified Home 进度条 |
| 程序开源，Skill 插件收费 | 商业模型 |
| 先针对开源算法搞，我来测 | 算法蒸馏是验收标准 |
| 先做好 X 流量，GitHub 就起来了 | GTM |

### 2.2 截图：三个已在用的 Skill

| Skill | 作用 | 硬边界 |
|-------|------|--------|
| `x-for-you-post` | 按 For You / Phoenix 官方权重写、改 | 只出正文，不发帖，不调 X API |
| `x-copy-check` | 已有文案对照规则，给可贴改法 | 不从零写，不发帖 |
| `binance-square-traffic-copy` | 币安广场短帖 | 流量场不止 X；一期不做，信息架构留「场」 |

「写」和「检」是两个按钮，不是一个万能 prompt。

### 2.3 截图：原创内容奖励计划

X 于 2026-08-07 关闭旧分成新注册；9 月 8 日起 Original Content Rewards。茄哥卡在唯一未勾项：

**过去 90 天已验证首页时间线曝光 246.3K / 500K，回复不计。**

Qualified impression = Premium 用户在 Home Timeline、帖子至少 50% 可见。工作台首页第一块应是这根进度条，不是粉丝数。触达层（For You）和收益层（Qualified Home）必须分开算。

### 2.4 黄赟演示帖

https://x.com/huangyun_122/status/2088306916648972309  
视频 12.4s：左侧自建表（AI 原创分 / 合格）+ 右侧 Grok Build TUI 跑 `/twitter-ai-zh-authors`；每 2 小时刷粉；Dashboard 按**粉增量**排序；点曲线尖峰弹出「+238 粉 + 对应帖正文」。

这是「暴涨归因」的标准交互。有效打法：热帖入库、暴涨号雷达、尖峰归因、Heavy 花在分析、用「已安排」做刷新。没有人把自动发帖当成愿意公开承认的付费点。

### 2.5 和旧设计的关系

`docs/features/x-search.md` 写过「不服务运营党」——那是给 Agent 用的 X 证据轨。本工作台是另一条付费产品线。底层可共用搜索，不要把排期矩阵做进 App 内核。

---

## 3. 开源算法（写 Skill 的单一事实源）

仓库：`github.com/xai-org/x-algorithm`。2026-08-13 起生产权重在 `home-mixer/params/param.rs`。现有 `x-tweet-writer/algorithm-checklist.md` 若仍写「权重未公开」须改掉。

公式：

```
FinalScore = Σ (weight_i × P(action_i | viewer, post))
```

权重乘的是**该读者做出该动作的预测概率**，不是全网点赞数。不能说「1 个举报抵 468 个赞」。

### 3.1 官方权重（同步 2026-08-12）

| 动作 | 权重 | 人话 |
|------|------|------|
| 复制链接分享 | **20** | 值得被贴到群 / 笔记 |
| 互关者回复原创 | 5+15=**20** | 仅互关 + 原创帖 |
| 普通回复 / 引用 / 私信分享 | 5 | 留讨论口、带判断 QT |
| 被关注 | 4 | 主页值得被关注 |
| 分享按钮 / 转推 / 点赞 | 2 / 1 / 0.5 | 便宜 |
| 点进帖 | 0.4 | 首屏停人 |
| 点进主页 / 离散停留 | **0** | 已不加分 |
| 图/开视频/优质完播 | 0.05 | 视频 ≥ **10 秒** |
| 举报 / 静音 / 不感兴趣 / 拉黑 | -234 / -58.8 / -43.2 / -31.2 | 负向极重 |

后处理：同作者第 2 条 ×0.5（下限 0.25）；未关注者 ×0.75；关注者发的回复/转推同样打税；**>48h 硬删**；陌生人回复进不了 For You。

可见性是另一层：For You 推荐比关注时间线更狠（`SpamHighRecall`、`DoNotAmplify` 等对路人 DROP）。降权检测必须同时跑排序层、可见性层、收益层。用户可自查 https://x.com/i/under_the_hood。

---

## 4. 功能分期

完整清单按一天的时间排。图例：★ 用户原规划，☆ 调研补出。

**第一期（4–6 周）只做让「会写不会改 Skill」的人当天觉得比 TUI 强的东西：**

1. 通用 Plugin UI Host + 夹具插件验收（先于任何 X 功能）
2. 推荐页安装 X 插件 → 侧栏出现
3. 资格进度条（手贴数字也行）
4. 三个按钮：加热写帖 / 降权预检+查重 / 引用与回复
5. 热帖雷达（已登录 Grok 搜 48h）
6. 单帖爆文归因（贴链接，对照官方权重）
7. 低消耗：预检快模型，终稿 4.6 / Heavy

不做：常驻监控、收益金额预测、多账号、广场、自动发帖。

验收：草稿预检出「复制链接 / 回复 / 负向」；爆款链接能说出命中哪条官方权重；能看见离 50 万还差多少。全程不改 SKILL.md。

**第二期：** 尖峰归因（点曲线出帖）、暴涨号雷达、账号/关键词监控（复用「已安排」）、语料与声纹、Imagine 配图、收益区间预测。

**第三期：** 财经 / 币 / 广场垂直包、多账号审批、企业号。

红线：自动发帖、自动互动、互关农场、教人刷曝光。

---

## 5. 职责切分

```
安装包（MIT App）                            插件仓（安装后才落到本机）
─────────────────                            ────────────────────────
PluginContribution 发现                      grok-app-extension.json
侧栏按列表渲染                               ui/index.html + assets + icon
PluginPaneHost（一个 iframe）                skills / 权重钉扎 / 付费墙 / 图表
host.* postMessage SDK                       插件自己的 i18n
#/plugin/{name}/{pane}                       业务 100%
/plugin-host/chrome.css（通用壳样式）
pluginHost.* 空态/错误文案
推荐表一行：名字 + git URL（可选）
```

**允许留在 App 的 X 相关内容（仅此）：** 推荐表里一行 marketplace 元数据（和 ChatCut 一样）。不是面板。

**禁止留在 App 的：** `XCreatorWorkbenchPage`、`mainPane === "x-creator"`、`sidebar.xCreator`、权重表、资格条、归因图。

---

## 6. 插件契约

在 `{plugin.path}/grok-app-extension.json`（或 `.grok-plugin/` 下同名）。不塞进 `plugin.json`，避免 `plugin validate` 对未知字段过敏。

```json
{
  "schemaVersion": 1,
  "app": "grok-app",
  "minAppVersion": "0.2.16",
  "id": "x-creator-workbench",
  "contributes": {
    "sidebar": [
      {
        "id": "home",
        "title": {
          "en": "X Studio",
          "zh": "X 博主工作台",
          "zh-TW": "X 博主工作臺"
        },
        "icon": "ui/icon.svg",
        "entry": "ui/index.html",
        "placement": "nav"
      }
    ]
  },
  "permissions": [
    "sessions.create",
    "sessions.read",
    "account.read",
    "catalog.read",
    "automations.readwrite",
    "storage",
    "clipboard.write",
    "dialog",
    "menu",
    "picker",
    "toast",
    "open",
    "license"
  ],
  "license": {
    "productId": "xcw-pro"
  }
}
```

| 字段 | 约束 |
|------|------|
| `schemaVersion` | 不认识的主版本整份忽略，设置页 warn |
| `minAppVersion` | 不够则侧栏不出现 |
| `sidebar[].id` | 插件内唯一，`[a-z0-9-]{1,32}` |
| `title` | 必须有 `en`；缺 `zh` 回退 `en`。App i18n **不收录** |
| `icon` / `entry` | 相对路径，只许 `ui/`；拒绝 `..`、绝对路径、符号链接逃逸 |
| `placement` | `nav` = 主侧栏（连接设备下方）；`more` = 「插件」分组。主侧栏最多 3 个 `nav`，多的降到 `more` |
| `permissions` | 只许 §10 白名单；未知权限 = 整份贡献拒绝 |

插件目录：

```
x-creator-workbench/
  .grok-plugin/plugin.json
  grok-app-extension.json
  ui/index.html          # 静态 SPA，插件自己打包
  ui/icon.svg
  ui/assets/…
  skills/…
```

App 不编译、不对齐 React 版本。插件用任意栈打出静态盘即可。

---

## 7. 发现与路由

```
启动 / 插件变更
  → plugin_list（name, enabled, path）
  → 对 enabled && path 存在 的项读 grok-app-extension.json
  → 校验 schema / 逃逸 / 权限 / minAppVersion
  → PluginContribution[]
  → 前端只按这份列表画侧栏
```

重扫时机：启动；install / enable / disable / uninstall / update 成功；设置页「刷新插件」。

| 事件 | 侧栏 | 若正在看该面板 |
|------|------|----------------|
| 安装并启用 | 出现 | — |
| 禁用 / 卸载 | 消失 | 退回 `#/workbench` |
| 更新 | 按新清单重建 | iframe 按新 entry 重载 |
| 清单损坏 / 越权 | 不出现，设置页 warn | — |

```ts
type MainPane =
  | { kind: "chat" }
  | { kind: "automations" }
  | { kind: "plugin"; plugin: string; pane: string };

// hash: #/plugin/{plugin}/{pane}
```

`AppWorkbench.tsx` 只加：贡献列表循环 + 一个 `PluginPaneHost`。禁止写 X 文案。状态进 `src/providers/PluginContributionsProvider.tsx`。`App.tsx` 继续冻结。

---

## 8. iframe 宿主

### 8.1 为什么是 iframe

| 做法 | 不用的原因 |
|------|------------|
| App 预写页面，安装后 unhide | 装包已含业务 |
| 动态 `import()` 插件 React | 和主 bundle 绑死，等于交出渲染进程 |
| Resources 原生子 WebView | 画在 DOM 之上，和浮动输入框、Modal 抢层 |

主栏用同窗口 iframe，层叠跟现有 GlassModal 一致。

### 8.2 加载

独立 loopback 前缀（不要把 media token 发给 iframe）：

```
GET http://127.0.0.1:{port}/plugin-ui/{plugin}/{sessionToken}/{rel}

{rel} 必须落在 {plugin.path}/ui/**
禁止 ..、符号链接逃出 ui/、禁止读 skills/ 与 .env
```

`sessionToken` 进程内随机、按插件一份。iframe：

```html
<iframe
  title="{清单 title}"
  src="{entryUrl}"
  sandbox="allow-scripts allow-forms allow-same-origin"
  allow=""
  style="border:0;width:100%;height:100%;background:transparent"
></iframe>
```

CSP（宿主加，插件改不掉）：

```
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';
img-src 'self' data: blob: http://127.0.0.1:*;
connect-src 'self';
frame-ancestors 'self';
```

第一期可放宽 `img-src https:`。`connect-src` 仍不许插件直连任意 API。

### 8.3 和原 UI 能融到什么程度

**能做到「同一套壳」**：侧栏、顶栏、壁纸、主题是 App 的；主区颜色、字号、圆角、按钮、卡片和「已安排」同一家族。

必须做，缺一就会一眼假：

| 项 | 做法 |
|----|------|
| 去框 / 透明 | iframe 无边框；`html, body { background: transparent }`，壁纸才能透出来 |
| 单滚动条 | 宿主主区 `overflow: hidden`，只让 iframe 内部滚 |
| Token | `host.ready` + `theme` 事件把 `docs/design-tokens.md` 全套写进 iframe `html`，含 `data-theme` |
| 字体 | 同一套 `--font-sans` / `--font-mono` |
| 宿主样式包 | `/plugin-host/chrome.css`（按钮、输入、卡片、空态、表、tabs）。通用壳，不是 X 业务 |
| 皮肤 | 切亮暗或换皮肤再推一次 token；插件禁止写死 `#111` |

**融不掉的缝，产品上避开：**

| 缝 | 对策 |
|----|------|
| Select / 右键 / Tooltip 画出 iframe 会被切 | 弹出层一律 `host.dialog` / `host.menu` / `host.picker`，App 画在 iframe **外面** |
| 系统 `<select>` / `alert` | 契约禁止 |
| Tab 进出 iframe 顿一下 | 可接受 |
| Finder 拖文件进 iframe | 附件走宿主 |
| 跨 iframe 多选复制 | `host.clipboard.writeText` |

原则：**插件只画页面内部；一切会越界的东西回宿主。** 一体感来自共用弹出层 + 同一套 token。

夹具验收：并排截图「已安排」和夹具页，色、字、圆角一致。

---

## 9. 会话往返（iframe → 建会话 → 跑完回传）

**可以实现。** App 已有 `sessionCreate` / `sessionSend`、`session://state`（`ready` = 本轮结束）、`session://turn_marker`、`session://turn_error`。宿主按 job 转给开过它的 iframe。

`postMessage` 立刻回 id，**不要**挂到模型跑完。

```ts
// 只插入，人点发送
host.sessions.compose({
  title: "预检草稿",
  skill: "x-copy-check",
  prompt: compiledText,
  target: "new",            // current 须二次确认
});

// 创建并立刻跑
host.sessions.run({
  title: "爆文归因",
  skill: "x-viral-attribution",
  prompt: compiledText,
  open: "background",       // 默认：人留在工作台
});
// → { jobId, sessionId }
```

随后事件：

```
session.started    { jobId, sessionId }
session.progress   { jobId, phase }          // P1，不传 token 流
session.needsUser  { jobId, kind }           // permission | ask_user
session.done       { jobId, sessionId, ok, reason, text?, artifact? }
```

iframe 重挂后用 `host.sessions.poll(jobId)` 拉终态。

**结果从哪来：** Skill 写 `~/.grok-app/plugin-data/{pluginId}/jobs/{jobId}.json`。宿主塞进 `artifact`。没有则退回最后一条助手正文（截断 32 KiB）。不要灌 tool timeline。

**`open`：** `background` = 主栏仍是插件，侧栏会话转圈；`focus` = 切到聊天，iframe **只隐藏不卸载**，`done` 仍能送到插件页。

**权限：** iframe 不重做权限 UI。App 用已有 GlassModal / AskUser 盖在 iframe 上面，同时发 `needsUser`。拒绝或超时 → `done` 且 `ok: false`。

`run` 只建新会话。禁止默认 `sessionSend` 到用户正在看的闲聊。

```
iframe                         宿主                          Agent
  │ run({ skill, prompt })       │                             │
  │  { jobId } ◄──────────────── │ sessionCreate + sessionSend │
  │  session.started ◄────────── │                             │
  │  session.needsUser ◄──────── │          GlassModal         │
  │                              │          turn ready         │
  │                              │ 读 jobs/{jobId}.json        │
  │  session.done { artifact } ◄─│                             │
  │  渲染预检卡 / 归因卡          │                             │
```

---

## 10. 能力与回传总表

通道只有 `postMessage`。未列即没有。单条消息 **≤ 256 KiB**。更大内容写 storage / jobs，只传 id。

```ts
// iframe → parent
{ v: 1, id: string, type: "req", method: string, params?: unknown }

// parent → iframe
{ v: 1, id: string, type: "res", ok: true, result: unknown }
{ v: 1, id: string, type: "res", ok: false, error: { code, message } }
{ v: 1, type: "event", event: string, payload: unknown }
```

校验：`event.source === iframe.contentWindow`，origin 必须是 plugin-ui loopback。方法不在该插件 `permissions` 里 → `E_FORBIDDEN`。

插件仓带一份薄 `host-client.js`，不要每个插件手写握手。

### 10.1 宿主 → iframe（事件）

| 事件 | 何时 | payload |
|------|------|---------|
| `host.ready` | iframe load 后首次握手 | `pluginId, paneId, locale, theme, tokens, appVersion, permissions[]` |
| `theme` | 亮暗 / 皮肤变 | `{ dataTheme, tokens }` |
| `locale` | 界面语言变 | `{ locale }` |
| `account` | 登录 / 登出 / 切号 | `{ signedIn, displayName, planLabel }`；email 需 `account.read` |
| `session.started` | 本插件 `run` 的会话已建 | `{ jobId, sessionId }` |
| `session.progress` | P1 | `{ jobId, phase }` |
| `session.needsUser` | 权限或提问 | `{ jobId, kind }` |
| `session.done` | 该 job 一轮结束 | `{ jobId, sessionId, ok, reason, text?, artifact? }` |
| `automation.ran` | 本插件建的已安排跑完 | `{ automationId, sessionId, ok, at }` |
| `license` | 激活状态变 | `{ state, productId }` |
| `contrib.revoke` | 禁用 / 卸载 | `{ reason }` |

不能订阅别人的会话，不能拿原始 `session://stream`。

### 10.2 iframe → 宿主（请求）

**默认就有：**

| 方法 | 传入 | 返回 |
|------|------|------|
| `host.getInfo` | — | 同 `host.ready` |
| `host.theme.get` | — | 当前 tokens |
| `host.locale.get` | — | locale |
| `host.sessions.poll` | `{ jobId }` | 该 job 终态；不是自己的 → 禁止 |
| `host.focus.pane` | — | 主栏切回本插件 |

**声明后才有：**

| permission | 方法 | 传入 | 返回 / 副作用 |
|------------|------|------|----------------|
| `sessions.create` | `host.sessions.compose` | `{ title?, skill?, prompt, target }` | `{ sessionId }`；填框不发送 |
| `sessions.create` | `host.sessions.run` | `{ title?, skill?, prompt, open }` | `{ jobId, sessionId }` |
| `sessions.create` | `host.sessions.open` | `{ sessionId }` | 切到该聊天（仅本插件创建的） |
| `sessions.read` | `host.sessions.get` | `{ sessionId }` | `{ title, state, updatedAt }`；无全文 |
| `account.read` | `host.account.summary` | — | 登录态、展示名、套餐、额度%；无 key |
| `catalog.read` | `host.catalog.models` | — | 可选模型 id + 展示名 |
| `automations.readwrite` | `host.automations.list` | — | 仅本插件创建的任务 |
| 同上 | `host.automations.upsert` | 标题 / prompt / 日程 / 模型 | `{ id }`；打插件来源标签 |
| 同上 | `host.automations.setEnabled` | `{ id, enabled }` | ok |
| `storage` | `get/set/list/delete` | 相对键；值 JSON ≤ 1 MiB/键 | 只动 `plugin-data/{pluginId}/` |
| `clipboard.write` | `writeText` | `{ text }` ≤ 256 KiB | ok |
| `dialog` | `notice` / `confirm` / `prompt` | title / body | 走 GlassModal，不是 `alert` |
| `menu` | `open` | `{ items, x, y }` 坐标相对 iframe | `{ id \| null }` |
| `picker` | `select` | `{ options, value? }` | `{ id }`；用 App `Select` |
| `toast` | `toast` | `{ message, tone? }` | — |
| `open` | `openExternal` | `{ url }` 仅 https | 系统浏览器 |
| `license` | `status` / `activate` | key | `{ state }` |
| `media.proxy` | `fetch` | `{ url }` https | `{ storageKey }` |
| `projects.read` | `list` | — | `{ id, name }[]`；默认无绝对路径 |

### 10.3 永远不开放

Tauri `invoke`；读 `auth.json` / API key / refresh token；任意文件系统、读别的插件、读项目源码；`sessionSend` 到不是自己建的会话；订阅全站 stream / tool timeline；代发 X；`clipboard.read`；装/卸插件；改 `config.toml` / 皮肤；`eval` 主窗口；打本机其它 loopback；原生 `<select>` / `alert` / `confirm`。

### 10.4 iframe 允许传出的口袋

没有「DOM 冒泡到 App」。能出去的只有方法参数：

| 口袋 | 适合 | 限制 |
|------|------|------|
| `prompt` | Skill 提示词、帖子草稿 | 256 KiB；更大先 storage |
| `artifact` | 预检分、归因卡、改写数组 | JSON，宿主不解释业务字段 |
| `storage` | 语料索引、监控名单、上次表单 | 每键 1 MiB；目录配额约 200 MiB |
| `dialog` / `toast` / `menu` 文案 | 短 UI 字符串 | title 80，body 2 KiB |
| `license.key` | 激活码 | 不进日志 |
| `url` | 给人去 X 的链接 | 仅 https |
| `clipboard` | 用户点「复制草稿」 | 应跟一次点击 |

---

## 11. 收费

激活 UI 画在插件 iframe 里。App 只提供通用 `host.license.status / activate`。

- 密钥：`~/.grok-app/plugin-data/{pluginId}/license.json`（0600）
- 绑定已登录 Grok user id
- 校验 URL 在清单 `license.verifyEndpoint`（https only）
- 离线宽限 7 天，所有付费插件共用
- 默认：没钥匙侧栏仍出现（内嵌付费墙）。若要「没钥匙当没装」，清单加 `sidebarRequiresLicense: true`

建议档位（可再调）：Creator ¥69 / Pro ¥169（主力） / Studio ¥499。不按发帖量抽成。免费钩子（每天 3 次预检）由插件用 storage 自己数。

---

## 12. App 落点（全是通用代码）

| 位置 | 做什么 |
|------|--------|
| `src-tauri/.../plugin_contributions.rs` | 扫 path、校验清单、plugin-ui 路由 |
| `src/lib/pluginContributions.ts` | 清单 → 侧栏 item；权限检查 |
| `src/providers/PluginContributionsProvider.tsx` | 拉列表、听插件变更 |
| `src/components/plugin-host/PluginPaneHost.tsx` | iframe + postMessage |
| `src/components/plugin-host/hostBridge.ts` | SDK → 已有 api |
| `src/lib/pluginRecommended.ts` | 推荐表多项 |
| `AppWorkbench.tsx` | `mainPane` 联合类型 + 侧栏 map + `#/plugin/…` |
| i18n `pluginHost.*` | 仅宿主空态 / 错误 |
| 静态 | `/plugin-host/chrome.css` |

**没有** `src/components/x-creator/`。

数据：`{app_data_root()}/plugin-data/{pluginId}/`（macOS 常见为 Application Support，不要字面量写 `~/.grok-app/...`）。不要 `x-creator/` 专目录，不要改 `~/.grok`。

施工覆盖（pi 2026-08-17 FLAG，详见 GOAL D1–D10）：独立 plugin-ui 端口，禁止接到 `media_server`；CSP `frame-ancestors` 用宿主 origin 白名单不是 `'self'`；hash 用清单 `id`；清单只放插件根目录；`run` 学已安排后台（create+connect+send），不改 `viewingSessionId`；`mainPane` 保持字符串 `"plugin"` + `pluginRoute`。

---

## 13. 宿主分期与验收

**Host P0（先于任何 X 功能）：**

1. 读清单 + 侧栏 map + `#/plugin/…`
2. plugin-ui loopback + iframe
3. Token 注入 + `chrome.css`
4. SDK：`getInfo` / `sessions.compose` / `sessions.run` / `poll` / `dialog` / `storage` / `toast`
5. `session.started` / `needsUser` / `done`
6. 安装 / 卸载摘挂
7. 推荐表可多项

**Host P1：** automations、license、theme 热更新、menu/picker、nav 超额降级、developer-mode vite。

**X 插件 P0：** 在独立仓实现资格条 / 写 / 检 / 归因。App 仓库零业务文件。

**夹具验收（不依赖 X）：**

- `fixtures/plugin-ui-hello/`：标题来自清单，按钮调 `host.dialog.notice`，再调 `host.sessions.compose` 能建会话。
- 未安装：侧栏无此项；bundle 里搜不到夹具标题。
- 安装启用：侧栏出现；主区无框、token 与「已安排」同色。
- `run` 一个短 Skill：iframe 收到 `session.done`。
- 禁用 / 卸载：项消失，hash 回聊天。
- 清单声明不存在的 permission：整份贡献拒绝，设置页 warn。

有了夹具，X 插件只是第二个消费者。

---

## 14. 下一步

1. 先落地 Host P0 + 夹具，再开 X 插件仓。
2. 和茄哥确认第一期三个按钮是否就是「写 / 检 / 归因」，资格条手贴数字能否接受。
3. 改 `x-tweet-writer` 过时的「权重未公开」（即使工作台未做）。
4. 插件仓先私有，推荐页用 git URL。

---

## 附录 A. 官方权重钉扎

来源：`home-mixer/params/param.rs` 2026-08-12T04:09:22Z

```
share_via_copy_link = 20
bidirectional_follow_reply_boost = 15   # 加在 reply 上，仅互关+原创
reply = 5
quote = 5
share_via_dm = 5
follow_author = 4
share = 2
retweet = 1
favorite = 0.5
click = 0.4
open_link = 0.2
photo_expand = video_open = vqv = 0.05
profile_click = 0
dwell = 0
cont_dwell_time = 0.004
report = -234
mute_author = -58.8
not_interested = -43.2
block_author = -31.2
author_diversity_decay = 0.5
author_diversity_floor = 0.25
oon_weight_factor = 0.75
min_video_duration_ms = 10000
age_limit = 48h
```

## 附录 B. 链接

- 算法：https://github.com/xai-org/x-algorithm
- 原创奖励：https://help.x.com/en/using-x/original-content-rewards
- Under the Hood：https://x.com/i/under_the_hood
- 黄赟演示：https://x.com/huangyun_122/status/2088306916648972309
- 插件市场：`docs/llm-wiki/plugins-marketplace.md`
- ChatCut 模式：`docs/llm-wiki/chatcut.md`
- 旧「不做运营党」：`docs/features/x-search.md`
- Design tokens：`docs/design-tokens.md`
- Dialogs：`docs/llm-wiki/dialogs.md`
