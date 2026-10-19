//! 翻译请求 journal（Python 写，`translation-request-journal.v1.jsonl`）的只读解析。
//!
//! 两处要读它：API 的「手动重试要不要确认重复风险」（retainpdf-jobs），和 runner 的
//! 「重启后能不能自动续跑」（retainpdf-runner）。两边必须用同一份判据，所以放在它们
//! 共同依赖的核心包里。
//!
//! 约定：dispatch 记录先落盘（fsync）再发请求，terminal 记录在拿到响应或确定失败后补上；
//! 只有 dispatch 没有 terminal 表示「结果不明」，绝不当作成功。

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde_json::Value;

const JOURNAL_SCHEMA: &str = "translation_request_journal_v1";
const JOURNAL_SCHEMA_VERSION: u64 = 1;

/// journal 的读取结论。`status` 取 `clean` / `ambiguous` / `corrupt`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationRequestJournalState {
    pub status: &'static str,
    pub unresolved_dispatches: u64,
    pub active_ambiguous_request_keys: u64,
    pub historical_ambiguous_request_keys: u64,
    /// 有结果不明的请求，或 journal 读不了：重发之前必须有人确认。
    pub requires_confirmation: bool,
    /// `corrupt` 时的原因。
    pub corrupt_detail: Option<&'static str>,
}

impl TranslationRequestJournalState {
    pub fn is_corrupt(&self) -> bool {
        self.status == "corrupt"
    }

    fn corrupt(detail: &'static str) -> Self {
        Self {
            status: "corrupt",
            unresolved_dispatches: 0,
            active_ambiguous_request_keys: 0,
            historical_ambiguous_request_keys: 0,
            requires_confirmation: true,
            corrupt_detail: Some(detail),
        }
    }
}

pub fn parse_translation_request_journal(path: &Path) -> TranslationRequestJournalState {
    let raw = match std::fs::read(path) {
        Ok(raw) => raw,
        Err(_) => return TranslationRequestJournalState::corrupt("translation request journal cannot be read"),
    };
    let complete_len = if raw.ends_with(b"\n") {
        raw.len()
    } else {
        raw.iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |index| index + 1)
    };
    let mut open_dispatches: HashMap<String, String> = HashMap::new();
    let mut active_ambiguous_keys: HashSet<String> = HashSet::new();
    let mut historical_ambiguous_keys: HashSet<String> = HashSet::new();

    for raw_line in raw[..complete_len].split(|byte| *byte == b'\n') {
        if raw_line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let event: Value = match serde_json::from_slice(raw_line) {
            Ok(event) => event,
            Err(_) => {
                return TranslationRequestJournalState::corrupt(
                    "translation request journal contains invalid JSON",
                )
            }
        };
        if event.get("schema").and_then(Value::as_str) != Some(JOURNAL_SCHEMA)
            || event.get("schema_version").and_then(Value::as_u64) != Some(JOURNAL_SCHEMA_VERSION)
        {
            return TranslationRequestJournalState::corrupt(
                "translation request journal contract is unsupported",
            );
        }
        let token = event
            .get("request_token")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let request_key = event
            .get("request_key")
            .and_then(Value::as_str)
            .unwrap_or_default();
        match event.get("event").and_then(Value::as_str) {
            Some("dispatch") if !token.is_empty() && !request_key.is_empty() => {
                open_dispatches.insert(token.to_string(), request_key.to_string());
            }
            Some("terminal") if !token.is_empty() => {
                let resolved_key = open_dispatches
                    .remove(token)
                    .unwrap_or_else(|| request_key.to_string());
                if resolved_key.is_empty() {
                    continue;
                }
                if event.get("outcome").and_then(Value::as_str) == Some("ambiguous") {
                    active_ambiguous_keys.insert(resolved_key.clone());
                    historical_ambiguous_keys.insert(resolved_key);
                } else {
                    // A later terminal result for the same deterministic request
                    // resolves the content-recovery ambiguity, while historical
                    // billing risk remains visible separately.
                    active_ambiguous_keys.remove(&resolved_key);
                    if open_dispatches.values().any(|key| key == &resolved_key) {
                        historical_ambiguous_keys.insert(resolved_key.clone());
                        open_dispatches.retain(|_, key| key != &resolved_key);
                    }
                }
            }
            _ => {}
        }
    }

    active_ambiguous_keys.extend(open_dispatches.values().cloned());
    historical_ambiguous_keys.extend(active_ambiguous_keys.iter().cloned());
    let requires_confirmation = !active_ambiguous_keys.is_empty();
    TranslationRequestJournalState {
        status: if requires_confirmation { "ambiguous" } else { "clean" },
        unresolved_dispatches: open_dispatches.len() as u64,
        active_ambiguous_request_keys: active_ambiguous_keys.len() as u64,
        historical_ambiguous_request_keys: historical_ambiguous_keys.len() as u64,
        requires_confirmation,
        corrupt_detail: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "retain-request-journal-{name}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn unmatched_dispatch_requires_confirmation_and_torn_tail_is_ignored() {
        let path = test_path("unmatched");
        std::fs::write(
            &path,
            concat!(
                "{\"schema\":\"translation_request_journal_v1\",\"schema_version\":1,",
                "\"event\":\"dispatch\",\"request_token\":\"t1\",\"request_key\":\"k1\"}\n",
                "{\"schema\":\"translation_request_journal_v1\""
            ),
        )
        .expect("write journal");
        let state = parse_translation_request_journal(&path);
        assert_eq!(state.status, "ambiguous");
        assert_eq!(state.unresolved_dispatches, 1);
        assert_eq!(state.active_ambiguous_request_keys, 1);
        assert!(state.requires_confirmation);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn later_success_resolves_active_ambiguity_but_preserves_history() {
        let path = test_path("resolved");
        std::fs::write(
            &path,
            concat!(
                "{\"schema\":\"translation_request_journal_v1\",\"schema_version\":1,\"event\":\"dispatch\",\"request_token\":\"t0\",\"request_key\":\"k1\"}\n",
                "{\"schema\":\"translation_request_journal_v1\",\"schema_version\":1,\"event\":\"dispatch\",\"request_token\":\"t1\",\"request_key\":\"k1\"}\n",
                "{\"schema\":\"translation_request_journal_v1\",\"schema_version\":1,\"event\":\"terminal\",\"request_token\":\"t1\",\"request_key\":\"k1\",\"outcome\":\"ambiguous\"}\n",
                "{\"schema\":\"translation_request_journal_v1\",\"schema_version\":1,\"event\":\"dispatch\",\"request_token\":\"t2\",\"request_key\":\"k1\"}\n",
                "{\"schema\":\"translation_request_journal_v1\",\"schema_version\":1,\"event\":\"terminal\",\"request_token\":\"t2\",\"request_key\":\"k1\",\"outcome\":\"succeeded\"}\n"
            ),
        )
        .expect("write journal");
        let state = parse_translation_request_journal(&path);
        assert_eq!(state.status, "clean");
        assert_eq!(state.active_ambiguous_request_keys, 0);
        assert_eq!(state.historical_ambiguous_request_keys, 1);
        assert_eq!(state.unresolved_dispatches, 0);
        assert!(!state.requires_confirmation);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn corrupt_complete_line_fails_closed() {
        let path = test_path("corrupt");
        std::fs::write(&path, b"{not-json}\n").expect("write journal");
        let state = parse_translation_request_journal(&path);
        assert!(state.is_corrupt());
        assert!(state.requires_confirmation);
        let _ = std::fs::remove_file(path);
    }
}
