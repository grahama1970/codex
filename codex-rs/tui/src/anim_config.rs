use codex_core::config::find_codex_home;
use codex_core::config::load_config_as_toml;
use std::sync::OnceLock;
use std::sync::RwLock;
use toml::Value as TomlValue;

// Session override for tips enabled/disabled.
static TIPS_OVERRIDE: OnceLock<RwLock<Option<bool>>> = OnceLock::new();

pub fn set_tips_override(v: Option<bool>) {
    let lock = TIPS_OVERRIDE.get_or_init(|| RwLock::new(None));
    if let Ok(mut g) = lock.write() {
        *g = v;
    }
}

fn get_tips_override() -> Option<bool> {
    if let Some(lock) = TIPS_OVERRIDE.get()
        && let Ok(g) = lock.read()
    {
        return *g;
    }
    None
}

pub fn tips_enabled() -> bool {
    // Highest precedence: in-session override via slash command.
    if let Some(v) = get_tips_override() {
        return v;
    }
    // Env override
    if let Ok(v) = std::env::var("CXPLUS_SPINNER_TIPS") {
        if v == "0" || v.eq_ignore_ascii_case("false") {
            return false;
        } else if v == "1" || v.eq_ignore_ascii_case("true") {
            return true;
        }
    }
    // config.toml: [anim] spinner_tips = true|false (default false)
    if let Ok(home) = find_codex_home()
        && let Ok(root) = load_config_as_toml(&home)
        && let Some(TomlValue::Table(anim)) = root.get("anim")
        && let Some(TomlValue::Boolean(b)) = anim.get("spinner_tips")
    {
        return *b;
    }
    false
}

pub fn tips_interval_secs() -> u64 {
    // config.toml: [anim] tip_interval_secs = 2
    if let Ok(home) = find_codex_home()
        && let Ok(root) = load_config_as_toml(&home)
        && let Some(TomlValue::Table(anim)) = root.get("anim")
        && let Some(TomlValue::Integer(i)) = anim.get("tip_interval_secs")
    {
        return (*i).max(1) as u64;
    }
    2
}

pub fn tips_list() -> Vec<String> {
    if let Ok(home) = find_codex_home()
        && let Ok(root) = load_config_as_toml(&home)
        && let Some(TomlValue::Table(anim)) = root.get("anim")
        && let Some(TomlValue::Array(arr)) = anim.get("tips")
    {
        let mut out = Vec::new();
        for v in arr {
            if let TomlValue::String(s) = v {
                out.push(s.clone());
            }
        }
        if !out.is_empty() {
            return out;
        }
    }
    vec![
        "Analyzing…".to_string(),
        "Indexing…".to_string(),
        "Linking…".to_string(),
        "Optimizing…".to_string(),
        "Syncing…".to_string(),
    ]
}
