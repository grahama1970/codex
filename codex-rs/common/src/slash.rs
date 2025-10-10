//! Minimal slash-commands parser for Codex.
//! A slash command is a single line starting with `/` followed by a verb and optional flags.
//! Parsing is intentionally simple and defensive.

#[derive(Debug, Clone, PartialEq)]
pub enum SlashCommand {
    Help,
    Status,
    /// Switch to light theme (best-effort; session/process scope).
    Light,
    /// Switch to dark theme (best-effort; session/process scope).
    Dark,
    Model {
        id: String,
    },
    Provider {
        id: String,
    },
    Profile {
        name: String,
    },
    Discover(DiscoverArgs),
    Grep {
        pattern: String,
        path: Option<String>,
    },
    Open {
        path: String,
        line: Option<usize>,
    },
    Fmt,
    Build,
    Test,
    Warmup {
        secs: Option<u64>,
    },
    Unknown {
        raw: String,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
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
    let Some(verb) = parts.first().cloned() else {
        return Some(SlashCommand::Unknown {
            raw: raw.to_string(),
        });
    };
    match verb.as_str() {
        "help" => Some(SlashCommand::Help),
        "status" => Some(SlashCommand::Status),
        "light" => Some(SlashCommand::Light),
        "dark" => Some(SlashCommand::Dark),
        "model" => parts
            .get(1)
            .cloned()
            .map(|id| SlashCommand::Model { id })
            .or_else(|| {
                Some(SlashCommand::Unknown {
                    raw: raw.to_string(),
                })
            }),
        "provider" => parts
            .get(1)
            .cloned()
            .map(|id| SlashCommand::Provider { id })
            .or_else(|| {
                Some(SlashCommand::Unknown {
                    raw: raw.to_string(),
                })
            }),
        "profile" => parts
            .get(1)
            .cloned()
            .map(|name| SlashCommand::Profile { name })
            .or_else(|| {
                Some(SlashCommand::Unknown {
                    raw: raw.to_string(),
                })
            }),
        "discover" => {
            let (args, errs) = parse_discover_flags(&parts[1..]);
            if !errs.is_empty() {
                let joined = errs.join("; ");
                return Some(SlashCommand::Unknown {
                    raw: format!("parse-error: {joined} (use /help for flags)"),
                });
            }
            Some(SlashCommand::Discover(args))
        }
        "grep" => {
            let pat = parts.get(1).cloned();
            let pth = parts.get(2).cloned();
            pat.map(|pattern| SlashCommand::Grep { pattern, path: pth })
                .or_else(|| {
                    Some(SlashCommand::Unknown {
                        raw: raw.to_string(),
                    })
                })
        }
        "open" => {
            // /open path[:line]
            if let Some(spec) = parts.get(1) {
                let (path, line) = parse_path_line(spec);
                Some(SlashCommand::Open { path, line })
            } else {
                Some(SlashCommand::Unknown {
                    raw: raw.to_string(),
                })
            }
        }
        "warmup" => {
            // Optional integer seconds: /warmup 8
            let secs = parts.get(1).and_then(|s| s.parse::<u64>().ok());
            Some(SlashCommand::Warmup { secs })
        }
        "fmt" => Some(SlashCommand::Fmt),
        "build" => Some(SlashCommand::Build),
        "test" => Some(SlashCommand::Test),
        _ => Some(SlashCommand::Unknown {
            raw: raw.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_unknown_flag_surfaces_parse_error() {
        let Some(SlashCommand::Unknown { raw }) = parse("/discover --bogus 1") else {
            panic!("expected Unknown for bad flag");
        };
        assert!(
            raw.contains("parse-error"),
            "unknown flag should surface parse-error message: {raw}"
        );
    }
}

#[cfg(test)]
mod tests_unknown_flag {
    use super::*;
    #[test]
    fn discover_unknown_flag_surfaces_parse_error() {
        let Some(SlashCommand::Unknown { raw }) = parse("/discover --bogus 1") else {
            panic!("expected Unknown for bad flag");
        };
        assert!(raw.contains("parse-error"), "unknown flag should surface parse-error message: {raw}");
    }
}
fn parse_discover_flags(args: &[String]) -> (DiscoverArgs, Vec<String>) {
    let mut out = DiscoverArgs::default();
    let mut errors = Vec::new();
    let mut i = 0;
    let is_flag = |s: &str| s.starts_with("--");
    while i < args.len() {
        match args[i].as_str() {
            "--min-params" if i + 1 < args.len() => {
                match args[i + 1].parse::<i64>() {
                    Ok(v) => out.min_params = Some(v),
                    Err(_) => errors.push(format!("invalid --min-params '{}'", args[i + 1])),
                }
                i += 2;
            }
            "--max-params" if i + 1 < args.len() => {
                match args[i + 1].parse::<i64>() {
                    Ok(v) => out.max_params = Some(v),
                    Err(_) => errors.push(format!("invalid --max-params '{}'", args[i + 1])),
                }
                i += 2;
            }
            "--max-output-ppm" if i + 1 < args.len() => {
                match args[i + 1].parse::<f64>() {
                    Ok(v) => out.max_output_ppm = Some(v),
                    Err(_) => errors.push(format!("invalid --max-output-ppm '{}'", args[i + 1])),
                }
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
            _ => {
                // Tighten: treat unknown --flags as parse errors
                if is_flag(&args[i]) {
                    errors.push(format!("unknown flag '{}'", args[i]));
                }
                i += 1;
            }
        }
    }
    (out, errors)
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

fn parse_path_line(spec: &str) -> (String, Option<usize>) {
    if let Some((p, l)) = spec.rsplit_once(':')
        && let Ok(n) = l.parse::<usize>()
    {
        return (p.to_string(), Some(n));
    }
    (spec.to_string(), None)
}

#[cfg(test)]
mod tests_basic {
    use super::*;
    #[test]
    fn parses_basic_help() {
        assert!(matches!(parse("/help"), Some(SlashCommand::Help)));
    }
    #[test]
    fn parses_discover_flags() {
        let cmd = parse("/discover --min-params 10 --require-modalities text,image").unwrap();
        match cmd {
            SlashCommand::Discover(d) => {
                assert_eq!(d.min_params, Some(10));
                assert_eq!(d.require_modalities.as_deref(), Some("text,image"));
            }
            _ => panic!("wrong kind"),
        }
    }

    #[test]
    fn flags_reject_bad_number() {
        let cmd = parse("/discover --min-params nope");
        assert!(matches!(cmd, Some(SlashCommand::Unknown { raw }) if raw.contains("parse-error")));
    }
}
