//! Sanitization filters and validation logic.

use blake3::Hasher;
use ingestion_planning::PlannedChunk;
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct SanitizationConfig {
    pub redact_patterns: Vec<String>,
    pub script_indicators: Vec<String>,
}

impl Default for SanitizationConfig {
    fn default() -> Self {
        Self {
            redact_patterns: vec![
                r"SECRET[-_A-Z0-9]+".into(),
                r"API_KEY[=:][A-Za-z0-9_-]+".into(),
                r#"(?i)password\s*[:=]\s*['"][^'"]+['"]"#.into(),
                r#"(?i)token\s*[:=]\s*['"][^'"]+['"]"#.into(),
            ],
            script_indicators: vec!["#!/bin".into(), "#!/usr/bin/env".into()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct SanitizedChunk {
    pub plan_id: String,
    pub scrubbed_payload: String,
    pub redaction_log: Vec<String>,
    pub validation_status: String,
}

#[derive(Debug, Error)]
pub enum SanitizationError {
    #[error("invalid redaction pattern: {0}")]
    InvalidPattern(String),
}

#[derive(Debug, Clone)]
pub struct Sanitizer {
    config: SanitizationConfig,
}

impl Sanitizer {
    #[must_use]
    pub const fn new(config: SanitizationConfig) -> Self {
        Self { config }
    }

    pub fn apply(&self, chunk: &PlannedChunk) -> Result<SanitizedChunk, SanitizationError> {
        let mut scrubbed = chunk.payload().to_string();
        let mut redaction_log = Vec::new();
        for pattern in &self.config.redact_patterns {
            let regex = Regex::new(pattern)
                .map_err(|_| SanitizationError::InvalidPattern(pattern.clone()))?;
            let matches: Vec<String> = regex
                .find_iter(&scrubbed)
                .map(|mat| mat.as_str().to_string())
                .collect();
            if !matches.is_empty() {
                let mut hasher = Hasher::new();
                for capture in &matches {
                    hasher.update(capture.as_bytes());
                    hasher.update(&[0u8]);
                }
                let digest = hasher.finalize().to_hex().to_string();
                let count = matches.len();
                redaction_log.push(format!("{pattern} => count={count}, digest={digest}"));
                scrubbed = regex.replace_all(&scrubbed, "[REDACTED]").into_owned();
            }
        }

        let mut validation_status = String::from("clean");
        for indicator in &self.config.script_indicators {
            if chunk.payload().starts_with(indicator) {
                validation_status = String::from("script-reviewed");
                break;
            }
        }

        Ok(SanitizedChunk {
            plan_id: chunk.plan().plan_id.clone(),
            scrubbed_payload: scrubbed,
            redaction_log,
            validation_status,
        })
    }
}
