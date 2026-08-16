# Plugin UI Host — invoke + postMessage (P0)

Contract source: `src/lib/pluginHost/`. WP-B / WP-D must import these names; do not invent new ones.

## Tauri commands

| Command | Purpose |
|---------|---------|
| `plugin_contributions_list` | Scan enabled plugins; return valid contributions + warn rows |
| `plugin_ui_endpoint` | `{ baseUrl, tokens: { [pluginId]: token } }` for iframe src |
| `plugin_host_warns` | Optional refresh of reject/warn reasons |

## postMessage envelope

```
req:   { v: 1, id, type: "req", method, params? }
res:   { v: 1, id, type: "res", ok: true, result }
       { v: 1, id, type: "res", ok: false, error: { code, message } }
event: { v: 1, type: "event", event, payload }
```

## P0 methods

Always: `host.getInfo` `host.theme.get` `host.locale.get` `host.sessions.poll` `host.focus.pane`

Declared: `host.sessions.compose` / `run` / `open` (`sessions.create`); `host.sessions.get` (`sessions.read`); `host.storage.*` (`storage`); `host.dialog.*` (`dialog`); `host.toast` (`toast`); `host.clipboard.writeText` (`clipboard.write`)

P1 methods are not registered. A manifest that lists a P1 or unknown permission is rejected whole.
