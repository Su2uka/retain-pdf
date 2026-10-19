use std::path::{Path, PathBuf};

use retainpdf_api_base::error::AppError;
use retainpdf_core::models::domain::JobSnapshot;

use super::{cached_output_is_fresh, job_artifacts_dir, DerivedArtifactDeps};

#[derive(Clone, Copy)]
pub enum BookImageKind {
    Cover,
    Thumbnail,
}

impl BookImageKind {
    pub fn file_name(self) -> &'static str {
        match self {
            Self::Cover => "cover.jpg",
            Self::Thumbnail => "thumbnail.jpg",
        }
    }

    pub fn width_px(self) -> u32 {
        match self {
            Self::Cover => 900,
            Self::Thumbnail => 360,
        }
    }
}

pub fn ensure_book_image(
    deps: DerivedArtifactDeps<'_>,
    data_root: &Path,
    job: &JobSnapshot,
    source_pdf: &Path,
    kind: BookImageKind,
) -> Result<PathBuf, AppError> {
    let output_dir = job_artifacts_dir(data_root, job)?;
    let output_path = output_dir.join(kind.file_name());
    ensure_book_image_at_path(deps, source_pdf, &output_path, kind)
}

/// 文档级封面/缩略图：从源 PDF 首页渲染，缓存到 documents/<id>/。
pub fn ensure_document_book_image(
    deps: DerivedArtifactDeps<'_>,
    data_root: &Path,
    document_id: &str,
    source_pdf: &Path,
    kind: BookImageKind,
) -> Result<PathBuf, AppError> {
    let output_dir = super::document_artifacts_dir(data_root, document_id)?;
    let output_path = output_dir.join(kind.file_name());
    ensure_book_image_at_path(deps, source_pdf, &output_path, kind)
}

fn ensure_book_image_at_path(
    deps: DerivedArtifactDeps<'_>,
    source_pdf: &Path,
    output_path: &Path,
    kind: BookImageKind,
) -> Result<PathBuf, AppError> {
    if output_path.exists() && output_path.is_file() {
        return Ok(output_path.to_path_buf());
    }
    render_book_image(deps.python_bin, source_pdf, output_path, kind.width_px())?;
    Ok(output_path.to_path_buf())
}

/// 单页预览图。缓存文件名只由页码、宽度、dpi 决定，所以要和对照 PDF、Word 一样比修改时间：
/// 源 PDF 比缓存图新（译文重渲染、原地重跑覆盖了 `translated.pdf`），就重新渲染，
/// 不能把旧页图当成新的发出去。
///
/// 页码超出 PDF 页数：渲染脚本以 [`PAGE_OUT_OF_RANGE_EXIT`] 退出，这里变成 404。
pub fn ensure_page_preview(
    deps: DerivedArtifactDeps<'_>,
    output_path: &Path,
    source_pdf: &Path,
    page_index: u32,
    width_px: u32,
    dpi: u32,
) -> Result<PathBuf, AppError> {
    if cached_output_is_fresh(output_path, &[source_pdf])? {
        return Ok(output_path.to_path_buf());
    }
    render_pdf_page_preview(
        deps.python_bin,
        source_pdf,
        output_path,
        page_index,
        width_px,
        dpi,
    )?;
    Ok(output_path.to_path_buf())
}

/// 渲染脚本发现页码超出 PDF 页数时的退出码，和别的失败区分开：客户端写错页码是 404，不是 500。
const PAGE_OUT_OF_RANGE_EXIT: i32 = 3;

fn render_book_image(
    python_bin: &str,
    source_pdf: &Path,
    output_path: &Path,
    width_px: u32,
) -> Result<(), AppError> {
    let script = r#"
import sys
from pathlib import Path
import fitz

source = Path(sys.argv[1])
output = Path(sys.argv[2])
width_px = int(sys.argv[3])

with fitz.open(source) as doc:
    if doc.page_count < 1:
        raise RuntimeError("source pdf has no pages")
    page = doc[0]
    scale = width_px / max(float(page.rect.width), 1.0)
    pix = page.get_pixmap(matrix=fitz.Matrix(scale, scale), alpha=False)
    output.parent.mkdir(parents=True, exist_ok=True)
    pix.save(output)
"#;
    let status = std::process::Command::new(python_bin)
        .arg("-c")
        .arg(script)
        .arg(source_pdf)
        .arg(output_path)
        .arg(width_px.to_string())
        .status()
        .map_err(|error| AppError::internal(format!("failed to render book image: {error}")))?;
    if !status.success() || !output_path.exists() {
        return Err(AppError::internal("failed to render book image"));
    }
    Ok(())
}

fn render_pdf_page_preview(
    python_bin: &str,
    source_pdf: &Path,
    output_path: &Path,
    page_index: u32,
    width_px: u32,
    dpi: u32,
) -> Result<(), AppError> {
    let script = r#"
import os
import sys
from pathlib import Path
import fitz

source = Path(sys.argv[1])
output = Path(sys.argv[2])
page_index = int(sys.argv[3])
width_px = int(sys.argv[4])
dpi = int(sys.argv[5])

with fitz.open(source) as doc:
    if page_index < 0 or page_index >= doc.page_count:
        print(f"page out of range: {page_index + 1}/{doc.page_count}", file=sys.stderr)
        sys.exit(3)
    page = doc[page_index]
    if dpi > 0:
        scale = dpi / 72.0
    else:
        scale = width_px / max(float(page.rect.width), 1.0)
    pix = page.get_pixmap(matrix=fitz.Matrix(scale, scale), alpha=False)
    output.parent.mkdir(parents=True, exist_ok=True)
    # 先写临时文件再换名：重新渲染旧缓存时，别的请求不会读到写了一半的图。
    partial = output.with_name(output.stem + ".part.jpg")
    pix.save(partial, jpg_quality=82)
    os.replace(partial, output)
"#;
    let status = std::process::Command::new(python_bin)
        .arg("-c")
        .arg(script)
        .arg(source_pdf)
        .arg(output_path)
        .arg(page_index.to_string())
        .arg(width_px.to_string())
        .arg(dpi.to_string())
        .status()
        .map_err(|error| AppError::internal(format!("failed to render page preview: {error}")))?;
    if status.code() == Some(PAGE_OUT_OF_RANGE_EXIT) {
        return Err(AppError::not_found(format!(
            "no such file: pages/{}.jpg",
            page_index + 1
        )));
    }
    if !status.success() || !output_path.exists() {
        return Err(AppError::internal("failed to render page preview"));
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, SystemTime};

    use super::*;

    /// 假 python：`python -c <脚本> <源> <输出> …`，按 `$exit` 退出，退出 0 时往输出里写 `new`。
    fn fake_python(dir: &Path, exit_code: i32) -> PathBuf {
        let path = dir.join(format!("fake-python-{exit_code}.sh"));
        std::fs::write(
            &path,
            format!("#!/bin/sh\nif [ {exit_code} -eq 0 ]; then printf new > \"$4\"; fi\nexit {exit_code}\n"),
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn set_mtime(path: &Path, time: SystemTime) {
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(time)
            .unwrap();
    }

    struct Case {
        dir: PathBuf,
        source: PathBuf,
        cached: PathBuf,
    }

    fn case(name: &str) -> Case {
        let dir = std::env::temp_dir().join(format!("retainpdf-preview-{name}-{}", fastrand::u64(..)));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("translated.pdf");
        let cached = dir.join("preview-translated-p0001-w1200-d0.jpg");
        std::fs::write(&source, b"%PDF").unwrap();
        std::fs::write(&cached, b"old").unwrap();
        Case { dir, source, cached }
    }

    #[test]
    fn cached_preview_older_than_its_pdf_is_rendered_again() {
        let case = case("stale");
        let now = SystemTime::now();
        set_mtime(&case.source, now);
        set_mtime(&case.cached, now - Duration::from_secs(60));
        let python = fake_python(&case.dir, 0);
        let deps = DerivedArtifactDeps::new(python.to_str().unwrap());
        let path = ensure_page_preview(deps, &case.cached, &case.source, 0, 1200, 0).unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"new", "译文重渲染以后不能再发旧页图");
    }

    #[test]
    fn cached_preview_newer_than_its_pdf_is_reused_without_rendering() {
        let case = case("fresh");
        let now = SystemTime::now();
        set_mtime(&case.source, now - Duration::from_secs(60));
        set_mtime(&case.cached, now);
        let python = fake_python(&case.dir, 1);
        let deps = DerivedArtifactDeps::new(python.to_str().unwrap());
        let path = ensure_page_preview(deps, &case.cached, &case.source, 0, 1200, 0).unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"old");
    }

    #[test]
    fn page_beyond_the_pdf_is_not_found_not_an_internal_error() {
        let case = case("range");
        std::fs::remove_file(&case.cached).unwrap();
        let python = fake_python(&case.dir, PAGE_OUT_OF_RANGE_EXIT);
        let deps = DerivedArtifactDeps::new(python.to_str().unwrap());
        let error = ensure_page_preview(deps, &case.cached, &case.source, 8, 1200, 0).unwrap_err();
        assert!(matches!(&error, AppError::NotFound(message) if message == "no such file: pages/9.jpg"), "{error:?}");
        let python = fake_python(&case.dir, 1);
        let deps = DerivedArtifactDeps::new(python.to_str().unwrap());
        let error = ensure_page_preview(deps, &case.cached, &case.source, 0, 1200, 0).unwrap_err();
        assert!(matches!(error, AppError::Internal(_)), "别的渲染失败仍然是内部错误");
    }
}
