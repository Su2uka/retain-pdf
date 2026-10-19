use std::path::{Path, PathBuf};

use serde_json::Value;

use retainpdf_api_base::error::AppError;
use crate::views::TranslationDebugIndexView;
use retainpdf_core::models::domain::JobSnapshot;

use crate::translations::common::{read_json_value, value_string};
use crate::{stored_path, StoredFile};

pub(super) fn read_translation_debug_index_file(
    data_root: &Path,
    job: &JobSnapshot,
) -> Result<Option<TranslationDebugIndexView>, AppError> {
    let Some(path) = stored_path(job, data_root, StoredFile::TranslationDebugIndex) else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(&path)?;
    let payload: TranslationDebugIndexView = serde_json::from_str(&text).map_err(|err| {
        AppError::internal(format!("parse debug index {}: {err}", path.display()))
    })?;
    Ok(Some(payload))
}

pub(super) fn translation_manifest_path(
    data_root: &Path,
    job: &JobSnapshot,
) -> Result<PathBuf, AppError> {
    // 任务在、还没翻译完：404 + `reason: not_ready`。列表接口接住它退回空列表。
    stored_path(job, data_root, StoredFile::TranslationManifest)
        .ok_or_else(|| AppError::not_ready("translation manifest"))
}

pub(super) fn load_manifest_pages(
    manifest_path: &Path,
) -> Result<Vec<(i64, String, Vec<Value>)>, AppError> {
    let manifest = read_json_value(manifest_path)?;
    let pages = manifest
        .get("pages")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AppError::internal(format!(
                "invalid translation manifest: {}",
                manifest_path.display()
            ))
        })?;
    let base_dir = manifest_path.parent().unwrap_or(manifest_path);
    let mut result = Vec::new();
    for page in pages {
        let page_idx = page
            .get("page_index")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        let rel_path = value_string(page.get("path"));
        if rel_path.is_empty() {
            continue;
        }
        let payload_path = if Path::new(&rel_path).is_absolute() {
            PathBuf::from(&rel_path)
        } else {
            base_dir.join(&rel_path)
        };
        let page_payload = read_json_value(&payload_path)?;
        let items = page_payload.as_array().cloned().unwrap_or_default();
        result.push((page_idx, rel_path, items));
    }
    Ok(result)
}
