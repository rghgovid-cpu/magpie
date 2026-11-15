use crate::error::{MagpieError, Result};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Where Magpie stores its own config (skills cache, preferences).
const MAGPIE_CONFIG_DIR: &str = "magpie";
/// OpenCode's config location (read/extended by Magpie).
const OPENCODE_CONFIG_DIR: &str = "opencode";
const OPENCODE_CONFIG_FILE: &str = "opencode.jsonc";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MagpieConfig {
    pub skill_sources: Vec<String>,
    pub default_paper_size: PaperSize,
    pub default_format: PrintFormat,
    pub auto_install: bool,
    pub llm_provider: LLMProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LLMProviderConfig {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum PaperSize {
    #[default]
    A4,
    Letter,
}

impl std::fmt::Display for PaperSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaperSize::A4 => write!(f, "a4"),
            PaperSize::Letter => write!(f, "letter"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum PrintFormat {
    #[default]
    Pdf,
    Docx,
}

impl std::fmt::Display for PrintFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrintFormat::Pdf => write!(f, "pdf"),
            PrintFormat::Docx => write!(f, "docx"),
        }
    }
}

/// Returns the directory to store Magpie config.
pub fn magpie_config_dir() -> Result<PathBuf> {
    let base = BaseDirs::new().ok_or_else(|| MagpieError::Custom("Could not determine user dirs".into()))?;
    let magpie_dir = base.config_dir().to_path_buf().join(MAGPIE_CONFIG_DIR);
    fs::create_dir_all(&magpie_dir)?;
    Ok(magpie_dir)
}

/// Returns the OpenCode config directory.
pub fn opencode_config_dir() -> Result<PathBuf> {
    let base = BaseDirs::new().ok_or_else(|| MagpieError::Custom("Could not determine user dirs".into()))?;
    let opencode_dir = base.config_dir().to_path_buf().join(OPENCODE_CONFIG_DIR);
    fs::create_dir_all(&opencode_dir)?;
    Ok(opencode_dir)
}

/// Returns the OpenCode skills directory.
pub fn opencode_skills_dir() -> Result<PathBuf> {
    Ok(opencode_config_dir()?.join("skills"))
}

/// Returns the path to opencode.jsonc.
pub fn opencode_config_path() -> Result<PathBuf> {
    Ok(opencode_config_dir()?.join(OPENCODE_CONFIG_FILE))
}

/// Returns the path to magpie's config file.
pub fn magpie_config_path() -> Result<PathBuf> {
    Ok(magpie_config_dir()?.join("config.toml"))
}

/// Load Magpie config, creating defaults if missing.
pub fn load_magpie_config() -> Result<MagpieConfig> {
    let path = magpie_config_path()?;
    if path.exists() {
        let contents = fs::read_to_string(&path)?;
        let config: MagpieConfig = toml::from_str(&contents)?;
        Ok(config)
    } else {
        let config = MagpieConfig::default();
        save_magpie_config(&config)?;
        Ok(config)
    }
}

/// Save Magpie config.
pub fn save_magpie_config(config: &MagpieConfig) -> Result<()> {
    let path = magpie_config_path()?;
    let contents = toml::to_string_pretty(config)?;
    fs::write(&path, contents)?;
    Ok(())
}

/// Read OpenCode's opencode.jsonc (tolerant to comments) and extract skill sources.
pub fn load_opencode_skill_sources() -> Result<Vec<String>> {
    let path = opencode_config_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let contents = fs::read_to_string(&path)?;
    // Strip JSONC comments tolerantly.
    let stripped = strip_jsonc_comments(&contents);
    let parsed: serde_json::Value = serde_json::from_str(&stripped)?;
    let mut sources = Vec::new();
    if let Some(arr) = parsed.get("skills").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                sources.push(s.to_string());
            }
        }
    }
    Ok(sources)
}

/// Add a source URL/path to OpenCode's skills array.
pub fn register_opencode_skill_source(source: &str) -> Result<()> {
    let path = opencode_config_path()?;
    let mut parsed: serde_json::Value = if path.exists() {
        let contents = fs::read_to_string(&path)?;
        let stripped = strip_jsonc_comments(&contents);
        serde_json::from_str(&stripped)?
    } else {
        serde_json::json!({ "$schema": "https://opencode.ai/config.json" })
    };

    let sources = match parsed.get_mut("skills") {
        Some(arr) if arr.is_array() => arr.as_array_mut().unwrap(),
        _ => {
            let arr = serde_json::Value::Array(vec![]);
            parsed["skills"] = arr;
            parsed["skills"].as_array_mut().unwrap()
        }
    };

    if !sources.iter().any(|v| v.as_str() == Some(source)) {
        sources.push(serde_json::Value::String(source.to_string()));
    }

    let contents = serde_json::to_string_pretty(&parsed)?;
    fs::write(&path, contents)?;
    Ok(())
}

/// Remove a source from OpenCode's skills array.
pub fn remove_opencode_skill_source(source: &str) -> Result<()> {
    let path = opencode_config_path()?;
    if !path.exists() {
        return Ok(());
    }
    let contents = fs::read_to_string(&path)?;
    let stripped = strip_jsonc_comments(&contents);
    let mut parsed: serde_json::Value = serde_json::from_str(&stripped)?;
    if let Some(arr) = parsed.get_mut("skills").and_then(|v| v.as_array_mut()) {
        arr.retain(|v| v.as_str() != Some(source));
    }
    let contents = serde_json::to_string_pretty(&parsed)?;
    fs::write(&path, contents)?;
    Ok(())
}

/// Strip JSONC comments (// and /* */) so we can parse as JSON.
/// String-aware: does not treat // or /* inside string literals as comments.
fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_string {
            if escaped {
                out.push(ch);
                escaped = false;
            } else if ch == '\\' {
                out.push(ch);
                escaped = true;
            } else if ch == '"' {
                out.push(ch);
                in_string = false;
            } else {
                out.push(ch);
            }
            continue;
        }

        // Not in a string.
        if ch == '"' {
            out.push(ch);
            in_string = true;
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            // Line comment until newline.
            chars.next(); // consume second /
            while let Some(c) = chars.next() {
                if c == '\n' {
                    out.push('\n');
                    break;
                }
            }
        } else if ch == '/' && chars.peek() == Some(&'*') {
            // Block comment.
            chars.next(); // consume *
            let mut closed = false;
            while let Some(c) = chars.next() {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next(); // consume /
                    closed = true;
                    break;
                }
            }
            if !closed {
                out.push_str("/*");
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Ensure the OpenCode skills directory exists.
pub fn ensure_opencode_skills_dir() -> Result<PathBuf> {
    let dir = opencode_skills_dir()?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Copy a built-in skill directory into OpenCode's skills directory.
pub fn copy_builtin_skill(skill_id: &str, source_dir: &Path) -> Result<PathBuf> {
    let dest_dir = ensure_opencode_skills_dir()?.join(skill_id);
    if dest_dir.exists() {
        fs::remove_dir_all(&dest_dir)?;
    }
    copy_dir_recursive(source_dir, &dest_dir)?;
    Ok(dest_dir)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// List installed skills in OpenCode's skills directory.
pub fn list_installed_skills() -> Result<Vec<PathBuf>> {
    let dir = opencode_skills_dir()?;
    let mut skills = Vec::new();
    if dir.exists() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                skills.push(path);
            }
        }
    }
    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_jsonc_comments() {
        let input = r#"{
  // a comment
  "skills": [
    "https://example.com/skills/" /* block */
  ]
}"#;
        let stripped = strip_jsonc_comments(input);
        assert!(!stripped.contains("// a comment"));
        assert!(!stripped.contains("/* block */"));
        // URLs with // should be preserved inside strings.
        assert!(stripped.contains("https://example.com/skills/"));
        // Should be valid JSON.
        serde_json::from_str::<serde_json::Value>(&stripped).unwrap();
    }

    #[test]
    fn test_strip_preserves_urls() {
        let input = r#"{ "url": "https://opencode.ai/config.json" }"#;
        let stripped = strip_jsonc_comments(input);
        assert_eq!(stripped, input);
        serde_json::from_str::<serde_json::Value>(&stripped).unwrap();
    }
}
