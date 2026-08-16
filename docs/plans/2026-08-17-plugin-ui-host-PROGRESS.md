# Progress — 通用 Plugin UI Host（Host P0）

> 实施时更新。每个 WP 合入后改状态并记下命令输出。  
> 规格：`2026-08-17-plugin-ui-host-GOAL.md`

| WP | 名称 | 状态 | 证据 |
|----|------|------|------|
| 0 | 工作树 + 设计 + pi 联审 + Goal | DONE | 分支 `feat/plugin-ui-host` @ 工作树；HANDOFF 已写 |
| A | 契约：类型 / hash / 权限 / 清单 | DONE | vitest 13/13 + typecheck；pi PASS |
| B | Rust 发现 + plugin-ui server | DONE | cargo plugin_contrib 4 + plugin_ui 5；pi PASS |
| C | 推荐表多项 | DONE | vitest pluginRecommended+settingsCatalog+i18n；typecheck；pi FLAG 已修 |
| D | iframe Host + bridge + Provider | DONE | vitest pluginHost+i18n；typecheck；pi PASS after FLAG |
| E | AppWorkbench 接线 | PENDING | |
| F | 夹具 + chrome.css / host-client.js | PENDING | |

## 基线

- 工作树：`~/.grok/worktrees/grok-app/plugin-ui-host`
- 分支：`feat/plugin-ui-host`
- 基于：`main` @ `4be09f1a` + 设计稿

## 日志

- 2026-08-17：开工作树；pi FLAG；D1–D10 锁定；Goal 文档落地。未改产品代码。
