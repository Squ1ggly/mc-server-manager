//! Reading and writing `server.properties` while preserving comments and
//! line order, so hand-edited files stay recognisable.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

const PROPERTIES_FILE_NAME: &str = "server.properties";

/// One `key=value` pair from `server.properties`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    pub key: String,
    pub value: String,
}

/// Reads all properties from a server directory. Returns an empty list when
/// the file doesn't exist yet (the server writes it on first start).
pub fn read(server_dir: &Path) -> AppResult<Vec<Property>> {
    let file_path = server_dir.join(PROPERTIES_FILE_NAME);
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let contents = std::fs::read_to_string(&file_path)?;
    let properties = parse(&contents);
    Ok(properties)
}

/// A complete default `server.properties`, so the file is fully populated
/// the moment a server is created. Without this the file is generated on
/// first start, which can clobber edits the user made in between.
const DEFAULT_PROPERTIES: &str = "\
allow-flight=false
allow-nether=true
difficulty=easy
enable-command-block=false
enforce-secure-profile=true
enforce-whitelist=false
force-gamemode=false
gamemode=survival
generate-structures=true
hardcore=false
hide-online-players=false
level-name=world
level-seed=
level-type=minecraft\\:normal
max-players=20
max-world-size=29999984
motd=A Minecraft Server
online-mode=true
pvp=true
simulation-distance=10
spawn-monsters=true
spawn-protection=16
view-distance=10
white-list=false
";

/// Writes the default `server.properties` if none exists yet. Called at
/// creation for game servers.
pub fn ensure_defaults(server_dir: &Path) -> AppResult<()> {
    let file_path = server_dir.join(PROPERTIES_FILE_NAME);
    if file_path.exists() {
        return Ok(());
    }
    std::fs::write(file_path, DEFAULT_PROPERTIES)?;
    Ok(())
}

/// Applies updated values to the file, keeping comments, ordering, and any
/// keys the update doesn't mention. Unknown new keys are appended.
pub fn write(server_dir: &Path, updates: &[Property]) -> AppResult<()> {
    let file_path = server_dir.join(PROPERTIES_FILE_NAME);
    let existing_contents = if file_path.exists() {
        std::fs::read_to_string(&file_path)?
    } else {
        String::new()
    };

    let merged = merge(&existing_contents, updates);
    std::fs::write(&file_path, merged)?;
    Ok(())
}

fn parse(contents: &str) -> Vec<Property> {
    let mut properties = Vec::new();
    for line in contents.lines() {
        let Some(property) = parse_line(line) else {
            continue;
        };
        properties.push(property);
    }
    properties
}

fn parse_line(line: &str) -> Option<Property> {
    let trimmed = line.trim();
    let is_comment_or_blank =
        trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!');
    if is_comment_or_blank {
        return None;
    }

    let (raw_key, raw_value) = split_key_value(trimmed)?;
    let property = Property {
        key: unescape(raw_key),
        value: unescape(raw_value),
    };
    Some(property)
}

/// Splits a property line into its raw (still-escaped) key and value. Like
/// Java's Properties loader, the separator is the first unescaped `=`, `:`, or
/// run of whitespace — an escaped `\=`/`\:` inside the value is not mistaken
/// for one.
fn split_key_value(line: &str) -> Option<(&str, &str)> {
    let mut escaped = false;
    let mut separator_start: Option<usize> = None;
    for (index, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character == '=' || character == ':' || character.is_whitespace() {
            separator_start = Some(index);
            break;
        }
    }

    let separator_start = separator_start?;
    let key = &line[..separator_start];
    // After the key, skip whitespace, then at most one `=`/`:`, then the
    // whitespace that follows it — whatever remains is the value.
    let after_key = line[separator_start..].trim_start();
    let after_separator = after_key.strip_prefix(['=', ':']).unwrap_or(after_key);
    let value = after_separator.trim_start();
    Some((key, value))
}

/// Turns Java `.properties` escapes (`\n`, `\t`, `\uXXXX`, `\:`, `\\`, …) into
/// the characters they represent, so the UI shows real values.
fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        let Some(escaped) = chars.next() else {
            // A trailing lone backslash isn't an escape — keep it literally.
            out.push('\\');
            break;
        };
        match escaped {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            'f' => out.push('\u{000C}'),
            'u' => push_unicode_escape(&mut out, &mut chars),
            other => out.push(other),
        }
    }
    out
}

/// Reads the four hex digits of a `\uXXXX` escape (the `\u` is already
/// consumed) and appends the character. Keeps the sequence literal if it isn't
/// a valid four-digit code point.
fn push_unicode_escape(out: &mut String, chars: &mut std::str::Chars) {
    let hex: String = chars.by_ref().take(4).collect();
    let code_point = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
    if hex.len() == 4 {
        if let Some(character) = code_point {
            out.push(character);
            return;
        }
    }
    out.push('\\');
    out.push('u');
    out.push_str(&hex);
}

/// Escapes a value for `server.properties` the way Java's Properties writer
/// does, so characters like `:` `=` `\` and a leading space survive being
/// read back by the server. UTF-8 text (accents, emoji) is left as-is; only
/// control characters need `\uXXXX` form.
fn escape_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for (index, character) in value.char_indices() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '=' => out.push_str("\\="),
            ':' => out.push_str("\\:"),
            ' ' if index == 0 => out.push_str("\\ "),
            control if (control as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", control as u32))
            }
            other => out.push(other),
        }
    }
    out
}

fn merge(existing_contents: &str, updates: &[Property]) -> String {
    let mut remaining_updates: Vec<&Property> = updates.iter().collect();
    let mut output_lines: Vec<String> = Vec::new();

    for line in existing_contents.lines() {
        let rewritten = rewrite_line(line, &mut remaining_updates);
        output_lines.push(rewritten);
    }

    for new_property in remaining_updates {
        output_lines.push(format!(
            "{}={}",
            new_property.key,
            escape_value(&new_property.value)
        ));
    }

    let mut merged = output_lines.join("\n");
    merged.push('\n');
    merged
}

/// Replaces a line's value if an update targets its key; removes the used
/// update from `remaining_updates`. Non-property lines pass through as-is.
fn rewrite_line(line: &str, remaining_updates: &mut Vec<&Property>) -> String {
    let Some(existing) = parse_line(line) else {
        return line.to_string();
    };

    let update_position = remaining_updates
        .iter()
        .position(|update| update.key == existing.key);
    let Some(position) = update_position else {
        return line.to_string();
    };

    let update = remaining_updates.remove(position);
    format!("{}={}", update.key, escape_value(&update.value))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "#Minecraft server properties\n#Wed Jul 15 2026\nmax-players=20\npvp=true\nmotd=A Minecraft Server\n";

    #[test]
    fn parses_keys_and_skips_comments() {
        let properties = parse(SAMPLE);
        assert_eq!(properties.len(), 3);
        assert_eq!(properties[0].key, "max-players");
        assert_eq!(properties[0].value, "20");
    }

    #[test]
    fn merge_preserves_comments_and_order() {
        let updates = vec![Property {
            key: "pvp".to_string(),
            value: "false".to_string(),
        }];
        let merged = merge(SAMPLE, &updates);

        assert!(merged.starts_with("#Minecraft server properties\n"));
        assert!(merged.contains("max-players=20"));
        assert!(merged.contains("pvp=false"));
        assert!(merged.contains("motd=A Minecraft Server"));
    }

    #[test]
    fn merge_appends_new_keys() {
        let updates = vec![Property {
            key: "view-distance".to_string(),
            value: "12".to_string(),
        }];
        let merged = merge(SAMPLE, &updates);
        assert!(merged.ends_with("view-distance=12\n"));
    }

    #[test]
    fn unescapes_java_escaped_values_on_read() {
        // `minecraft\:normal` and a `\u` sequence should decode to real chars.
        let properties = parse("level-type=minecraft\\:normal\ncafe=caf\\u00e9\n");
        assert_eq!(properties[0].value, "minecraft:normal");
        assert_eq!(properties[1].value, "café");
    }

    #[test]
    fn reads_colon_and_whitespace_separators() {
        let properties = parse("max-players:20\nmotd A Minecraft Server\n");
        assert_eq!(properties[0].key, "max-players");
        assert_eq!(properties[0].value, "20");
        assert_eq!(properties[1].key, "motd");
        assert_eq!(properties[1].value, "A Minecraft Server");
    }

    #[test]
    fn escapes_special_characters_on_write() {
        let updates = vec![Property {
            key: "level-type".to_string(),
            value: "minecraft:normal".to_string(),
        }];
        let merged = merge("level-type=default\n", &updates);
        assert!(merged.contains("level-type=minecraft\\:normal"));
    }

    #[test]
    fn round_trips_values_that_need_escaping() {
        // A backslash, a colon and a leading space would all be lost or
        // misread by the server without escaping.
        let original = " C:\\worlds:main";
        let updates = vec![Property {
            key: "motd".to_string(),
            value: original.to_string(),
        }];
        let merged = merge("motd=A Minecraft Server\n", &updates);
        let reparsed = parse(&merged);
        let motd = reparsed
            .iter()
            .find(|property| property.key == "motd")
            .expect("motd should survive the round-trip");
        assert_eq!(motd.value, original);
    }
}
