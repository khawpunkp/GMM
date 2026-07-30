use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{KeybindInfo, PersistVar};
use crate::mods::find_mod_ini_paths;

/// Parses a single INI file's content for keybinds: looks for a `; Constants` comment marker,
/// then collects `key = value` lines inside `[Key...]` sections that appear after it. Ports the
/// old app's `get_ini_keybinds` line-scanning algorithm.
pub fn parse_keybinds_from_ini(content: &str) -> Vec<KeybindInfo> {
    let mut current_section_title: Option<String> = None;
    let mut found_constants_tag = false;
    let mut keybinds = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if !found_constants_tag {
            if line.starts_with(';') && line[1..].trim_start().to_lowercase().contains("constants") {
                found_constants_tag = true;
            }
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let section_name = line[1..line.len() - 1].trim().to_string();
            current_section_title = if section_name.to_lowercase().starts_with("key") {
                Some(section_name)
            } else {
                None
            };
        } else if let Some(title) = &current_section_title {
            if line.to_lowercase().starts_with("key") && line.contains('=') {
                if let Some(value_part) = line.splitn(2, '=').nth(1) {
                    let keybind_value = value_part.trim().to_string();
                    if !keybind_value.is_empty() {
                        keybinds.push(KeybindInfo { title: title.clone(), key: keybind_value });
                    }
                }
            }
        }
    }

    keybinds
}

/// Finds the mod's INI file(s) on disk (via the same dual enabled/DISABLED_ path check used
/// everywhere else) and returns the first one with any keybinds found after its `; Constants`
/// marker.
pub fn get_keybinds(base_mods_path: &Path, folder_name: &str) -> Vec<KeybindInfo> {
    for ini_path in find_mod_ini_paths(base_mods_path, folder_name) {
        let Ok(content) = fs::read_to_string(&ini_path) else {
            continue;
        };
        let keybinds = parse_keybinds_from_ini(&content);
        if !keybinds.is_empty() {
            return keybinds;
        }
    }
    Vec::new()
}

/// Scans for `global persist $name = value` lines anywhere in the file (3DMigoto writes the
/// current value back into this line itself as the user cycles it in-game — GMM just reads and,
/// on request, overwrites this same line), then cross-references every `[Key...]` section for a
/// `type = cycle` + `$name = a,b,c,...` line to recover that var's valid range.
pub fn parse_persist_vars_from_ini(content: &str) -> Vec<PersistVar> {
    let mut vars: Vec<PersistVar> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("global persist") else { continue };
        let Some(dollar_pos) = rest.find('$') else { continue };
        let after_dollar = &rest[dollar_pos + 1..];
        let Some(eq_pos) = after_dollar.find('=') else { continue };
        let name = after_dollar[..eq_pos].trim();
        let value_str = after_dollar[eq_pos + 1..].trim();
        let Ok(value) = value_str.parse::<i64>() else { continue };
        if name.is_empty() {
            continue;
        }
        vars.push(PersistVar { name: name.to_string(), value, options: Vec::new() });
    }

    // Cross-reference [Key...] sections for this var's cycle range.
    let mut current_section: Option<String> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let section_name = trimmed[1..trimmed.len() - 1].trim().to_string();
            current_section = if section_name.to_lowercase().starts_with("key") {
                Some(section_name)
            } else {
                None
            };
            continue;
        }
        if current_section.is_none() {
            continue;
        }
        let Some(dollar_pos) = trimmed.find('$') else { continue };
        let after_dollar = &trimmed[dollar_pos + 1..];
        let Some(eq_pos) = after_dollar.find('=') else { continue };
        let name = after_dollar[..eq_pos].trim();
        let values_str = after_dollar[eq_pos + 1..].trim();
        if let Some(var) = vars.iter_mut().find(|v| v.name == name) {
            let options: Vec<i64> = values_str.split(',').filter_map(|s| s.trim().parse::<i64>().ok()).collect();
            if !options.is_empty() {
                var.options = options;
            }
        }
    }

    vars
}

pub fn get_persist_vars(base_mods_path: &Path, folder_name: &str) -> Vec<PersistVar> {
    for ini_path in find_mod_ini_paths(base_mods_path, folder_name) {
        let Ok(content) = fs::read_to_string(&ini_path) else {
            continue;
        };
        let vars = parse_persist_vars_from_ini(&content);
        if !vars.is_empty() {
            return vars;
        }
    }
    Vec::new()
}

fn find_ini_containing_var(base_mods_path: &Path, folder_name: &str, var_name: &str) -> Option<PathBuf> {
    let needle = format!("${}", var_name);
    for ini_path in find_mod_ini_paths(base_mods_path, folder_name) {
        if let Ok(content) = fs::read_to_string(&ini_path) {
            if content.lines().any(|l| {
                let t = l.trim();
                t.starts_with("global persist") && t.contains(&needle)
            }) {
                return Some(ini_path);
            }
        }
    }
    None
}

/// Surgically replaces just the matching `global persist $name = OLD` line's value, leaving every
/// other byte of the file untouched (these files are hand-written 3DMigoto scripts with comments
/// and exact formatting that a generic INI-rewrite pass would risk mangling).
pub fn set_persist_var(base_mods_path: &Path, folder_name: &str, var_name: &str, new_value: i64) -> Result<(), String> {
    let ini_path = find_ini_containing_var(base_mods_path, folder_name, var_name)
        .ok_or_else(|| format!("Could not find a 'global persist ${}' line for this mod.", var_name))?;

    let content = fs::read_to_string(&ini_path).map_err(|e| e.to_string())?;
    let needle = format!("${}", var_name);
    let mut replaced = false;

    let new_lines: Vec<String> = content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if !replaced && trimmed.starts_with("global persist") && trimmed.contains(&needle) {
                if let Some(dollar_pos) = line.find('$') {
                    if let Some(eq_pos) = line[dollar_pos..].find('=') {
                        let prefix = &line[..dollar_pos + eq_pos + 1];
                        replaced = true;
                        return format!("{}{}", prefix, format!(" {}", new_value));
                    }
                }
            }
            line.to_string()
        })
        .collect();

    if !replaced {
        return Err(format!("Could not locate the 'global persist ${}' line to update.", var_name));
    }

    let newline = if content.contains("\r\n") { "\r\n" } else { "\n" };
    fs::write(&ini_path, new_lines.join(newline)).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_keybinds_after_constants_marker() {
        let ini = r#"
[Mod]
Name = Test Mod

; Constants
[Constants]
global persist $active = 0

[KeySwap]
key = ]
type = cycle
$active = 0,1

[KeyToggleHat]
key = h
type = toggle
$hat_on = 0,1
"#;
        let keybinds = parse_keybinds_from_ini(ini);
        assert_eq!(
            keybinds,
            vec![
                KeybindInfo { title: "KeySwap".to_string(), key: "]".to_string() },
                KeybindInfo { title: "KeyToggleHat".to_string(), key: "h".to_string() },
            ]
        );
    }

    #[test]
    fn ignores_key_sections_before_constants_marker() {
        let ini = r#"
[KeyBeforeConstants]
key = x

; Constants
[KeyAfter]
key = y
"#;
        let keybinds = parse_keybinds_from_ini(ini);
        assert_eq!(keybinds, vec![KeybindInfo { title: "KeyAfter".to_string(), key: "y".to_string() }]);
    }

    #[test]
    fn returns_empty_when_no_constants_marker() {
        let ini = "[KeySomething]\nkey = z\n";
        assert!(parse_keybinds_from_ini(ini).is_empty());
    }

    const REAL_WORLD_INI: &str = r#"
[Constants]
global $active
global persist $body = 2
global persist $legs = 0

[KeySwapBody]
key = no_modifiers UP
condition = $active == 1
type = cycle
$body = 0,1,2,3

[KeySwapLegs]
key = no_modifiers DOWN
type = cycle
$legs = 0,1,2
"#;

    #[test]
    fn parses_persist_vars_with_cycle_ranges() {
        let vars = parse_persist_vars_from_ini(REAL_WORLD_INI);
        assert_eq!(
            vars,
            vec![
                PersistVar { name: "body".to_string(), value: 2, options: vec![0, 1, 2, 3] },
                PersistVar { name: "legs".to_string(), value: 0, options: vec![0, 1, 2] },
            ]
        );
    }

    #[test]
    fn ignores_non_persist_globals() {
        let vars = parse_persist_vars_from_ini(REAL_WORLD_INI);
        assert!(!vars.iter().any(|v| v.name == "active"));
    }

    #[test]
    fn set_persist_var_replaces_only_the_matching_line() {
        let base = std::env::temp_dir().join(format!("gmm_persist_test_{}", std::process::id()));
        let mod_dir = base.join("TestMod");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(mod_dir.join("mod.ini"), REAL_WORLD_INI).unwrap();

        set_persist_var(&base, "TestMod", "body", 3).expect("write should succeed");

        let updated = fs::read_to_string(mod_dir.join("mod.ini")).unwrap();
        assert!(updated.contains("global persist $body = 3"));
        assert!(updated.contains("global persist $legs = 0"), "unrelated var must be untouched");
        assert!(updated.contains("condition = $active == 1"), "unrelated lines must be untouched");

        let vars = parse_persist_vars_from_ini(&updated);
        let body = vars.iter().find(|v| v.name == "body").unwrap();
        assert_eq!(body.value, 3);
        assert_eq!(body.options, vec![0, 1, 2, 3]);

        fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn set_persist_var_errors_when_var_not_found() {
        let base = std::env::temp_dir().join(format!("gmm_persist_test_missing_{}", std::process::id()));
        let mod_dir = base.join("TestMod");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(mod_dir.join("mod.ini"), REAL_WORLD_INI).unwrap();

        let result = set_persist_var(&base, "TestMod", "does_not_exist", 1);
        assert!(result.is_err());

        fs::remove_dir_all(&base).ok();
    }
}
