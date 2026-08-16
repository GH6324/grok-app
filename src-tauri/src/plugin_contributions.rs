//! Scan enabled CLI plugins for `grok-app-extension.json` at the package root.
//! Mirrors `src/lib/pluginHost` (GOAL D4 / D5). Does not read `.grok-plugin/`.

use std::fs;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::plugin_ui_server::PluginUiHandle;

const P0_PERMISSIONS: &[&str] = &[
    "sessions.create",
    "sessions.read",
    "storage",
    "dialog",
    "toast",
    "clipboard.write",
];

const PLUGIN_ID_MAX: usize = 32;

#[derive(Debug, Clone)]
pub struct PluginScanInput {
    pub name: String,
    pub enabled: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SidebarContributionDto {
    pub id: String,
    pub title_en: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_zh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_zh_tw: Option<String>,
    pub icon: String,
    pub entry: String,
    pub placement: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginContributionDto {
    pub id: String,
    pub cli_name: String,
    pub path: String,
    pub min_app_version: Option<String>,
    pub permissions: Vec<String>,
    pub sidebar: Vec<SidebarContributionDto>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginHostWarn {
    pub plugin: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginScanResult {
    pub contributions: Vec<PluginContributionDto>,
    pub warns: Vec<PluginHostWarn>,
}

fn is_plugin_id(s: &str) -> bool {
    let b = s.as_bytes();
    (1..=PLUGIN_ID_MAX).contains(&b.len())
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

pub fn is_p0_permission(name: &str) -> bool {
    P0_PERMISSIONS.contains(&name)
}

pub fn is_p1_permission(name: &str) -> bool {
    name.starts_with("automations.")
        || matches!(
            name,
            "license"
                | "menu"
                | "picker"
                | "media.proxy"
                | "account.read"
                | "catalog.read"
                | "open"
                | "projects.read"
                | "automations.readwrite"
                | "automations.read"
                | "automations.write"
        )
}

/// Relative `ui/…` only. Rejects `..`, absolute, backslash, empty segments.
pub fn is_safe_ui_rel_path(raw: &str) -> bool {
    let p = raw.trim();
    if p.is_empty() || p.starts_with('/') || p.starts_with('\\') {
        return false;
    }
    if p.contains('\\') || p.contains('\0') {
        return false;
    }
    if !p.starts_with("ui/") {
        return false;
    }
    let segs: Vec<&str> = p.split('/').collect();
    if segs.len() < 2 {
        return false;
    }
    segs.iter()
        .all(|s| !s.is_empty() && *s != "." && *s != "..")
}

fn parse_semver_triple(raw: &str) -> Option<(u64, u64, u64)> {
    let mut parts = raw.trim().split('.');
    let maj = parts.next()?.parse().ok()?;
    let min = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let pat = parts
        .next()
        .unwrap_or("0")
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0);
    Some((maj, min, pat))
}

pub fn app_meets_min_version(app: &str, min: &str) -> bool {
    match (parse_semver_triple(app), parse_semver_triple(min)) {
        (Some(a), Some(m)) => a >= m,
        _ => true,
    }
}

fn warn(plugin: &str, code: &str, message: impl Into<String>) -> PluginHostWarn {
    PluginHostWarn {
        plugin: plugin.to_string(),
        code: code.to_string(),
        message: message.into(),
    }
}

/// Parse a deserialized `grok-app-extension.json` object.
pub fn parse_extension_manifest(
    raw: &Value,
    cli_name: &str,
    plugin_path: &str,
) -> Result<PluginContributionDto, Vec<PluginHostWarn>> {
    let mut warns = Vec::new();
    let obj = match raw.as_object() {
        Some(o) => o,
        None => {
            return Err(vec![warn(
                cli_name,
                "not_object",
                "manifest must be a JSON object",
            )]);
        }
    };

    if obj.get("schemaVersion").and_then(|v| v.as_u64()) != Some(1) {
        warns.push(warn(
            cli_name,
            "schema_version",
            "unknown schemaVersion; only 1 is accepted",
        ));
    }
    if let Some(app) = obj.get("app").and_then(|v| v.as_str()) {
        if app != "grok-app" {
            warns.push(warn(cli_name, "app", "app must be grok-app"));
        }
    }

    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if !is_plugin_id(&id) {
        warns.push(warn(
            cli_name,
            "id",
            "id must match [a-z0-9-]{1,32}",
        ));
    }

    let min_app_version = obj
        .get("minAppVersion")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    if let Some(min) = min_app_version.as_deref() {
        if !app_meets_min_version(env!("CARGO_PKG_VERSION"), min) {
            warns.push(warn(
                cli_name,
                "min_app_version",
                format!("requires app {min}"),
            ));
        }
    }

    let sidebar_raw = obj
        .get("contributes")
        .and_then(|v| v.get("sidebar"))
        .and_then(|v| v.as_array());
    let mut sidebar = Vec::new();
    let mut seen = std::collections::HashSet::new();
    match sidebar_raw {
        Some(items) if !items.is_empty() => {
            for item in items {
                let Some(row) = item.as_object() else {
                    warns.push(warn(cli_name, "sidebar_item", "sidebar item must be an object"));
                    continue;
                };
                let pane = row
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !is_plugin_id(&pane) {
                    warns.push(warn(
                        cli_name,
                        "sidebar_id",
                        "sidebar id must match [a-z0-9-]{1,32}",
                    ));
                    continue;
                }
                if !seen.insert(pane.clone()) {
                    warns.push(warn(
                        cli_name,
                        "sidebar_dup",
                        "sidebar id must be unique inside the plugin",
                    ));
                    continue;
                }
                let title = row.get("title").and_then(|v| v.as_object());
                let title_en = title
                    .and_then(|t| t.get("en"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if title_en.is_empty() {
                    warns.push(warn(cli_name, "title_en", "title.en is required"));
                    continue;
                }
                let icon = row
                    .get("icon")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let entry = row
                    .get("entry")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !is_safe_ui_rel_path(&icon) {
                    warns.push(warn(
                        cli_name,
                        "sidebar_icon",
                        "icon must be a relative path under ui/",
                    ));
                    continue;
                }
                if !is_safe_ui_rel_path(&entry) {
                    warns.push(warn(
                        cli_name,
                        "sidebar_entry",
                        "entry must be a relative path under ui/",
                    ));
                    continue;
                }
                let placement = match row.get("placement").and_then(|v| v.as_str()) {
                    Some("more") => "more",
                    _ => "nav",
                };
                let title_zh = title
                    .and_then(|t| t.get("zh"))
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());
                let title_zh_tw = title
                    .and_then(|t| t.get("zh-TW"))
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());
                sidebar.push(SidebarContributionDto {
                    id: pane,
                    title_en,
                    title_zh,
                    title_zh_tw,
                    icon,
                    entry,
                    placement: placement.to_string(),
                });
            }
        }
        _ => warns.push(warn(
            cli_name,
            "sidebar",
            "at least one sidebar pane is required",
        )),
    }

    let mut permissions = Vec::new();
    let mut p1 = Vec::new();
    let mut unknown = Vec::new();
    if let Some(arr) = obj.get("permissions").and_then(|v| v.as_array()) {
        let mut seen = std::collections::HashSet::new();
        for p in arr {
            let name = p.as_str().unwrap_or("").trim();
            if name.is_empty() || !seen.insert(name.to_string()) {
                continue;
            }
            if is_p0_permission(name) {
                permissions.push(name.to_string());
            } else if is_p1_permission(name) {
                p1.push(name.to_string());
            } else {
                unknown.push(name.to_string());
            }
        }
    }
    if !p1.is_empty() {
        warns.push(warn(
            cli_name,
            "permission_p1",
            format!("P1 permissions reject the contribution: {}", p1.join(", ")),
        ));
    }
    if !unknown.is_empty() {
        warns.push(warn(
            cli_name,
            "permission_unknown",
            format!(
                "unknown permissions reject the contribution: {}",
                unknown.join(", ")
            ),
        ));
    }

    if !warns.is_empty() {
        return Err(warns);
    }

    Ok(PluginContributionDto {
        id,
        cli_name: cli_name.to_string(),
        path: plugin_path.to_string(),
        min_app_version,
        permissions,
        sidebar,
    })
}

/// Scan installed plugin rows. Missing manifest = skip (not every plugin is a UI host).
pub fn scan_plugin_contributions(inputs: &[PluginScanInput]) -> PluginScanResult {
    let mut out = PluginScanResult::default();
    for p in inputs {
        if !p.enabled {
            continue;
        }
        let Some(path) = p.path.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
            continue;
        };
        let root = Path::new(path);
        if !root.is_dir() {
            continue;
        }
        let manifest_path = root.join("grok-app-extension.json");
        if !manifest_path.is_file() {
            continue;
        }
        let text = match fs::read_to_string(&manifest_path) {
            Ok(t) => t,
            Err(e) => {
                out.warns.push(warn(
                    &p.name,
                    "read_error",
                    format!("cannot read grok-app-extension.json: {e}"),
                ));
                continue;
            }
        };
        let value: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => {
                out.warns.push(warn(
                    &p.name,
                    "parse_error",
                    "manifest is not valid JSON",
                ));
                continue;
            }
        };
        match parse_extension_manifest(&value, &p.name, path) {
            Ok(c) => out.contributions.push(c),
            Err(ws) => out.warns.extend(ws),
        }
    }
    out
}

fn inputs_from_plugins_json(value: &Value) -> Vec<PluginScanInput> {
    let Some(arr) = value.get("plugins").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|item| {
            let name = item.get("name")?.as_str()?.trim().to_string();
            if name.is_empty() {
                return None;
            }
            let enabled = item.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            let path = item
                .get("path")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            Some(PluginScanInput {
                name,
                enabled,
                path,
            })
        })
        .collect()
}

fn apply_scan_to_ui(handle: &PluginUiHandle, scan: &PluginScanResult) {
    let ids: Vec<String> = scan.contributions.iter().map(|c| c.id.clone()).collect();
    handle.retain(&ids);
    for c in &scan.contributions {
        handle.upsert(c.id.clone(), std::path::PathBuf::from(&c.path));
    }
}

#[tauri::command]
pub async fn plugin_contributions_list(
    ui: State<'_, PluginUiHandle>,
) -> Result<Value, String> {
    let listed = crate::commands::plugins_list().await?;
    let inputs = inputs_from_plugins_json(&listed);
    let scan = scan_plugin_contributions(&inputs);
    apply_scan_to_ui(&ui, &scan);
    serde_json::to_value(&scan).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn plugin_host_warns(ui: State<'_, PluginUiHandle>) -> Result<Value, String> {
    let listed = crate::commands::plugins_list().await?;
    let scan = scan_plugin_contributions(&inputs_from_plugins_json(&listed));
    apply_scan_to_ui(&ui, &scan);
    Ok(serde_json::json!({ "warns": scan.warns }))
}

fn storage_file(plugin_id: &str) -> Result<std::path::PathBuf, String> {
    if !is_plugin_id(plugin_id) {
        return Err("invalid plugin id".into());
    }
    Ok(crate::paths::plugin_data_dir(plugin_id).join("storage.json"))
}

fn read_storage_bag(plugin_id: &str) -> Result<serde_json::Map<String, Value>, String> {
    let path = storage_file(plugin_id)?;
    if !path.is_file() {
        return Ok(serde_json::Map::new());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(m)) => Ok(m),
        _ => Ok(serde_json::Map::new()),
    }
}

fn write_storage_bag(
    plugin_id: &str,
    bag: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    let path = storage_file(plugin_id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, serde_json::to_vec_pretty(bag).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn plugin_storage_get(plugin_id: String, key: String) -> Result<Value, String> {
    let bag = read_storage_bag(&plugin_id)?;
    Ok(bag.get(&key).cloned().unwrap_or(Value::Null))
}

#[tauri::command]
pub async fn plugin_storage_set(
    plugin_id: String,
    key: String,
    value: Value,
) -> Result<Value, String> {
    let mut bag = read_storage_bag(&plugin_id)?;
    bag.insert(key, value);
    write_storage_bag(&plugin_id, &bag)?;
    Ok(serde_json::json!({ "ok": true }))
}

#[tauri::command]
pub async fn plugin_storage_list(plugin_id: String) -> Result<Value, String> {
    let bag = read_storage_bag(&plugin_id)?;
    Ok(serde_json::json!({ "keys": bag.keys().cloned().collect::<Vec<_>>() }))
}

#[tauri::command]
pub async fn plugin_storage_delete(plugin_id: String, key: String) -> Result<Value, String> {
    let mut bag = read_storage_bag(&plugin_id)?;
    bag.remove(&key);
    write_storage_bag(&plugin_id, &bag)?;
    Ok(serde_json::json!({ "ok": true }))
}

#[cfg(test)]
mod plugin_contrib_tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;

    fn write_manifest(dir: &Path, body: &str) {
        fs::create_dir_all(dir).unwrap();
        let mut f = fs::File::create(dir.join("grok-app-extension.json")).unwrap();
        f.write_all(body.as_bytes()).unwrap();
    }

    fn valid_json(id: &str, extra_perm: &str) -> String {
        format!(
            r#"{{
  "schemaVersion": 1,
  "app": "grok-app",
  "id": "{id}",
  "contributes": {{
    "sidebar": [{{
      "id": "home",
      "title": {{ "en": "Hello Host" }},
      "icon": "ui/icon.svg",
      "entry": "ui/index.html"
    }}]
  }},
  "permissions": ["sessions.create", "dialog"{extra}]
}}"#,
            extra = if extra_perm.is_empty() {
                String::new()
            } else {
                format!(", \"{extra_perm}\"")
            }
        )
    }

    #[test]
    fn plugin_contrib_accepts_p0_root_manifest() {
        let dir = std::env::temp_dir().join(format!("grok-pc-ok-{}", std::process::id()));
        write_manifest(&dir, &valid_json("hello-host", ""));
        let scan = scan_plugin_contributions(&[PluginScanInput {
            name: "cli-hello".into(),
            enabled: true,
            path: Some(dir.to_string_lossy().into_owned()),
        }]);
        assert!(scan.warns.is_empty(), "{:?}", scan.warns);
        assert_eq!(scan.contributions.len(), 1);
        assert_eq!(scan.contributions[0].id, "hello-host");
        assert_eq!(scan.contributions[0].cli_name, "cli-hello");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn plugin_contrib_rejects_unknown_and_p1_permissions() {
        let dir = std::env::temp_dir().join(format!("grok-pc-p1-{}", std::process::id()));
        write_manifest(&dir, &valid_json("hello-host", "license"));
        let scan = scan_plugin_contributions(&[PluginScanInput {
            name: "cli-hello".into(),
            enabled: true,
            path: Some(dir.to_string_lossy().into_owned()),
        }]);
        assert!(scan.contributions.is_empty());
        assert!(scan.warns.iter().any(|w| w.code == "permission_p1"));
        let _ = fs::remove_dir_all(&dir);

        let dir2 = std::env::temp_dir().join(format!("grok-pc-unk-{}", std::process::id()));
        write_manifest(&dir2, &valid_json("hello-host", "not-a-real-perm"));
        let scan2 = scan_plugin_contributions(&[PluginScanInput {
            name: "cli-hello".into(),
            enabled: true,
            path: Some(dir2.to_string_lossy().into_owned()),
        }]);
        assert!(scan2.contributions.is_empty());
        assert!(scan2.warns.iter().any(|w| w.code == "permission_unknown"));
        let _ = fs::remove_dir_all(&dir2);
    }

    #[test]
    fn plugin_contrib_rejects_entry_escape() {
        let raw = json!({
            "schemaVersion": 1,
            "app": "grok-app",
            "id": "hello-host",
            "contributes": { "sidebar": [{
                "id": "home",
                "title": { "en": "Hello" },
                "icon": "ui/icon.svg",
                "entry": "ui/../skills/SKILL.md"
            }]},
            "permissions": []
        });
        let err = parse_extension_manifest(&raw, "x", "/tmp/x").unwrap_err();
        assert!(err.iter().any(|w| w.code == "sidebar_entry"));
        assert!(!is_safe_ui_rel_path("ui/../secret"));
        assert!(!is_safe_ui_rel_path("skills/SKILL.md"));
        assert!(is_safe_ui_rel_path("ui/index.html"));
    }

    #[test]
    #[test]
    fn plugin_contrib_storage_roundtrip_under_plugin_data() {
        let _g = crate::paths::APP_HOME_ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!("grok-pc-store-{}", std::process::id()));
        std::env::set_var("GROK_APP_HOME", &tmp);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            plugin_storage_set(
                "hello-host".into(),
                "k".into(),
                json!("v"),
            )
            .await
            .unwrap();
            let got = plugin_storage_get("hello-host".into(), "k".into())
                .await
                .unwrap();
            assert_eq!(got, json!("v"));
            let listed = plugin_storage_list("hello-host".into()).await.unwrap();
            assert_eq!(listed["keys"], json!(["k"]));
        });
        std::env::remove_var("GROK_APP_HOME");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn plugin_contrib_skips_disabled_and_missing_manifest() {
        let dir = std::env::temp_dir().join(format!("grok-pc-skip-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let scan = scan_plugin_contributions(&[
            PluginScanInput {
                name: "off".into(),
                enabled: false,
                path: Some(dir.to_string_lossy().into_owned()),
            },
            PluginScanInput {
                name: "on".into(),
                enabled: true,
                path: Some(dir.to_string_lossy().into_owned()),
            },
        ]);
        assert!(scan.contributions.is_empty());
        assert!(scan.warns.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
