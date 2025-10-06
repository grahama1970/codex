use crate::color::blend;
use crate::color::is_light;
use crate::color::perceptual_distance;
use crate::terminal_palette::terminal_palette;
use codex_core::config::find_codex_home;
use codex_core::config::load_config_as_toml;
use ratatui::style::Color;
use ratatui::style::Style;
use std::sync::OnceLock;
use std::sync::RwLock;
use toml::Value as TomlValue;

/// Returns the style for a user-authored message using the provided terminal background.
pub fn user_message_style(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    match terminal_bg {
        Some(bg) => Style::default().bg(user_message_bg(bg)),
        None => Style::default(),
    }
}

#[allow(clippy::disallowed_methods)]
pub fn user_message_bg(terminal_bg: (u8, u8, u8)) -> Color {
    let top = if is_light(terminal_bg) {
        (0, 0, 0)
    } else {
        (255, 255, 255)
    };
    let bottom = terminal_bg;
    let Some(color_level) = supports_color::on_cached(supports_color::Stream::Stdout) else {
        return Color::default();
    };

    let target = blend(top, bottom, 0.1);
    if color_level.has_16m {
        let (r, g, b) = target;
        Color::Rgb(r, g, b)
    } else if color_level.has_256
        && let Some(palette) = terminal_palette()
        && let Some((i, _)) = palette.into_iter().enumerate().min_by(|(_, a), (_, b)| {
            perceptual_distance(*a, target)
                .partial_cmp(&perceptual_distance(*b, target))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    {
        Color::Indexed(i as u8)
    } else {
        Color::default()
    }
}

// Optional in-process theme override (for live /light and /dark switching).
static THEME_OVERRIDE: OnceLock<RwLock<Option<String>>> = OnceLock::new();

fn theme_override() -> Option<String> {
    if let Some(lock) = THEME_OVERRIDE.get()
        && let Ok(guard) = lock.read()
    {
        return guard.clone();
    }
    None
}

pub fn set_theme_override(theme: Option<&str>) {
    let lock = THEME_OVERRIDE.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = lock.write() {
        *guard = theme.map(std::string::ToString::to_string);
    }
}

fn parse_color_name_or_hex(s: &str) -> Option<Color> {
    let name = s.trim().to_ascii_lowercase();
    match name.as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" | "purple" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "white" => Some(Color::White),
        _ => {
            // Support #RRGGBB
            let hex = name.strip_prefix('#').unwrap_or(name.as_str());
            if hex.len() == 6
                && let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                )
            {
                return Some(Color::Rgb(r, g, b));
            }
            None
        }
    }
}

fn read_brand_color(key: &str, default: Color) -> Color {
    // Read ~/.codex/config.toml as a generic TOML value so we don't need to
    // change core config types for optional theming.
    if let Ok(home) = find_codex_home()
        && let Ok(root) = load_config_as_toml(&home)
        && let Some(TomlValue::Table(tui)) = root.get("tui")
        && let Some(TomlValue::Table(brand)) = tui.get("brand")
        && let Some(TomlValue::String(s)) = brand.get(key)
        && let Some(c) = parse_color_name_or_hex(s)
    {
        return c;
    }
    default
}

/// Title style for the cxplus fork banner.
pub fn brand_title_style() -> Style {
    let def = match theme_override()
        .or_else(|| std::env::var("CXPLUS_THEME").ok())
        .as_deref()
    {
        Some("light") => Color::Blue,
        Some("dark-dim") => Color::Magenta,
        _ => Color::Magenta,
    };
    Style::default().fg(read_brand_color("title_color", def))
}

/// Accent style for small labels in the header/help.
pub fn brand_accent_style() -> Style {
    let def = match theme_override()
        .or_else(|| std::env::var("CXPLUS_THEME").ok())
        .as_deref()
    {
        Some("light") => Color::Blue,
        Some("dark-dim") => Color::Magenta,
        _ => Color::Magenta,
    };
    Style::default().fg(read_brand_color("accent_color", def))
}

fn themed_color(dark_hex: &str, light_hex: &str) -> Color {
    match theme_override()
        .or_else(|| std::env::var("CXPLUS_THEME").ok())
        .as_deref()
    {
        Some("light") => parse_color_name_or_hex(light_hex).unwrap_or(Color::Blue),
        _ => parse_color_name_or_hex(dark_hex).unwrap_or(Color::Magenta),
    }
}

pub fn state_success_style() -> Style {
    Style::default().fg(themed_color("#19B28E", "#167E6B"))
}

pub fn state_warning_style() -> Style {
    Style::default().fg(themed_color("#E6B450", "#B7791F"))
}

pub fn state_error_style() -> Style {
    Style::default().fg(themed_color("#E16D76", "#C5394A"))
}

pub fn state_info_style() -> Style {
    Style::default().fg(themed_color("#4DB9F7", "#2563EB"))
}

pub fn link_style() -> Style {
    // Preserve existing snapshots by keeping cyan as default on dark.
    let color = match theme_override()
        .or_else(|| std::env::var("CXPLUS_THEME").ok())
        .as_deref()
    {
        Some("light") => themed_color("#6CB2FF", "#2952CC"),
        _ => Color::Cyan,
    };
    Style::default().fg(color)
}
