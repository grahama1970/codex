//! Minimal slash-commands parser for Codex.
//! A slash command is a single line starting with `/` followed by a verb and optional flags.
//! Parsing is intentionally simple and defensive.

#[derive(Debug, Clone, PartialEq)]
pub enum SlashCommand {
    Help,
    Status,
    Model { id: String },
    Provider { id: String },
    Profile { name: String },
    Discover(DiscoverArgs),
    Unknown { raw: String },
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Default)]
pub struct DiscoverArgs {
    pub min_params: Option<i64>,
    pub max_params: Option<i64>,
    pub max_output_ppm: Option<f64>,
    pub require_modalities: Option<String>,
    pub require_capabilities: Option<String>,
}


/// Parse a single-line slash command. Returns None if the line does not begin with '/'.
pub fn parse(line: &str) -> Option<SlashCommand> {
    let raw = line.trim();
    if !raw.starts_with('/') {
        return None;
    }
    let parts = shellish_split(&raw[1..]);
    let Some(verb) = parts.first().cloned() else { return Some(SlashCommand::Unknown { raw: raw.to_string() }); };
    match verb.as_str() {
        "help" => Some(SlashCommand::Help),
        "status" => Some(SlashCommand::Status),
        "model" => parts.get(1).cloned().map(|id| SlashCommand::Model { id }).or_else(|| Some(SlashCommand::Unknown { raw: raw.to_string() })),
        "provider" => parts.get(1).cloned().map(|id| SlashCommand::Provider { id }).or_else(|| Some(SlashCommand::Unknown { raw: raw.to_string() })),
        "profile" => parts.get(1).cloned().map(|name| SlashCommand::Profile { name }).or_else(|| Some(SlashCommand::Unknown { raw: raw.to_string() })),
        "discover" => Some(SlashCommand::Discover(parse_discover_flags(&parts[1..]))),
        _ => Some(SlashCommand::Unknown { raw: raw.to_string() }),
    }
}

fn parse_discover_flags(args: &[String]) -> DiscoverArgs {
    let mut out = DiscoverArgs::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--min-params" if i + 1 < args.len() => {
                out.min_params = args[i + 1].parse::<i64>().ok();
                i += 2;
            }
            "--max-params" if i + 1 < args.len() => {
                out.max_params = args[i + 1].parse::<i64>().ok();
                i += 2;
            }
            "--max-output-ppm" if i + 1 < args.len() => {
                out.max_output_ppm = args[i + 1].parse::<f64>().ok();
                i += 2;
            }
            "--require-modalities" if i + 1 < args.len() => {
                out.require_modalities = Some(args[i + 1].clone());
                i += 2;
            }
            "--require-capabilities" if i + 1 < args.len() => {
                out.require_capabilities = Some(args[i + 1].clone());
                i += 2;
            }
            _ => i += 1,
        }
    }
    out
}

fn shellish_split(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut quote_char = '\0';
    for ch in s.chars() {
        if in_quote {
            if ch == quote_char {
                in_quote = false;
            } else {
                cur.push(ch);
            }
        } else {
            match ch {
                '\'' | '"' => {
                    in_quote = true;
                    quote_char = ch;
                }
                ' ' | '\t' => {
                    if !cur.is_empty() {
                        out.push(std::mem::take(&mut cur));
                    }
                }
                _ => cur.push(ch),
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_basic_help() {
        assert!(matches!(parse("/help"), Some(SlashCommand::Help)));
    }
    #[test]
    fn parses_discover_flags() {
        let cmd = parse("/discover --min-params 10 --require-modalities text,image").unwrap();
        match cmd { SlashCommand::Discover(d) => {
            assert_eq!(d.min_params, Some(10));
            assert_eq!(d.require_modalities.as_deref(), Some("text,image"));
        }, _ => panic!("wrong kind") }
    }
}
