//! 文件夹级的 agent 工作区。
//!
//! 起因：到这一版为止 agent 的工作区钉死在单个 job 上，所以「这几篇里哪些互相
//! 矛盾」这类问题**在结构上问不出来** —— 不是模型不行，是它看不见第二本书。
//!
//! 这里守四件事，每一件反过来都是一种安静的错：
//! - 成员书能被看见（不然这个功能等于没做）
//! - **跳过的书要被说出来**（沉默地少几本 = agent 以为它看全了，结论是错的）
//! - 重建 books/ 不能顺手删掉 agent 的产出
//! - 重译换了 active_job 之后，链接要跟着换（指着旧译文不会报错，只是错）

use std::fs;
use std::path::PathBuf;

use retainpdf_data::db::Db;
use retainpdf_core::models::domain::{CreateJobInput, JobArtifacts, JobSnapshot, UploadRecord};

use axum::http::StatusCode;
use axum::response::IntoResponse;
use retainpdf_core::models::api::DocumentRecord;

use super::collection_workspace::{
    collection_workspace_dir, ensure_collection_workspace, is_valid_collection_id, link_name,
    materialize_collection_workspace, remove_collection_dir, ReadingBook,
};

/// `new_collection_id` 生成的形状（`col-` + 14 位时间 + `-` + 6 位小写十六进制）。
const COL: &str = "col-20260923000000-0c0c01";

struct Fixture {
    root: PathBuf,
    db: Db,
}

impl Fixture {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("retain-collection-ws-{}", fastrand::u64(..)));
        fs::create_dir_all(root.join("uploads")).unwrap();
        let db = Db::new(root.join("jobs.db"), root.clone());
        db.init().unwrap();
        Self { root, db }
    }

    /// 建一本书：upload → document → job，并把 job 设成 active。
    ///
    /// 全程走公开 API（`save_upload_with_document` / `link_job_to_document` /
    /// `set_document_active_job`），不直接 UPDATE 表 —— 直接写表的话，哪天归属
    /// 换个存法这些测试会继续绿着骗人。
    fn book(&self, document_id: &str, title: &str, job_id: &str) -> PathBuf {
        let job_root = self.root.join("jobs").join(job_id);
        fs::create_dir_all(job_root.join("ocr")).unwrap();
        fs::write(job_root.join("ocr").join("document.v1.json"), b"{}").unwrap();

        let upload_id = self.upload(document_id, title, job_id);
        let mut job = JobSnapshot::new(job_id.into(), CreateJobInput::default(), vec![]);
        job.artifacts = Some(JobArtifacts {
            job_root: Some(format!("jobs/{job_id}")),
            ..Default::default()
        });
        self.db.save_job(&job).unwrap();
        self.db.link_job_to_document(job_id, &upload_id).unwrap();
        self.db
            .set_document_active_job(document_id, job_id, Some(7))
            .unwrap();
        job_root
    }

    /// 一本还没翻译完的书：有档案，但 `active_job_id` 始终是 NULL。
    fn book_without_translation(&self, document_id: &str, title: &str) {
        self.upload(document_id, title, "never-translated");
    }

    fn upload(&self, document_id: &str, title: &str, tag: &str) -> String {
        let upload_id = format!("upload-{document_id}-{tag}");
        self.db
            .save_upload_with_document(&UploadRecord {
                upload_id: upload_id.clone(),
                filename: format!("{document_id}.pdf"),
                stored_path: self
                    .root
                    .join("uploads")
                    .join(format!("{document_id}.pdf"))
                    .to_string_lossy()
                    .into_owned(),
                bytes: 1,
                page_count: 7,
                uploaded_at: "2026-09-23T00:00:00Z".into(),
                developer_mode: false,
                content_hash: document_id.into(),
            })
            .unwrap();
        self.db
            .update_document_fields(document_id, Some(title), None)
            .unwrap();
        upload_id
    }

    fn collect(&self, collection_id: &str, document_ids: &[&str]) {
        self.db
            .create_collection(collection_id, "测试文件夹", None)
            .unwrap();
        self.db
            .add_documents_to_collection(
                collection_id,
                &document_ids.iter().map(|id| (*id).to_string()).collect::<Vec<_>>(),
            )
            .unwrap();
    }

    fn workspace(&self, collection_id: &str) -> PathBuf {
        collection_workspace_dir(&self.root, collection_id).unwrap()
    }

    /// 测试里的「阅读入口」：书卡的 active_job 就是可读的译文。真的阅读入口（`workspace_reading_book`）
    /// 看哪个任务覆盖了整本，最后一条用例单独测它。
    fn reading_book(&self, document: &DocumentRecord) -> Option<ReadingBook> {
        let job_id = document.active_job_id.clone().filter(|id| !id.is_empty())?;
        Some(ReadingBook { root: self.root.join("jobs").join(&job_id), job_id })
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn every_member_book_shows_up_under_books() {
    let fx = Fixture::new();
    fx.book("doc-a", "注意力机制综述", "job-a");
    fx.book("doc-b", "扩散模型的采样加速", "job-b");
    fx.collect(COL, &["doc-a", "doc-b"]);

    let manifest = ensure_collection_workspace(&fx.db, &fx.root, COL, &|d| fx.reading_book(d)).unwrap();
    assert_eq!(manifest.books.len(), 2);
    assert!(manifest.skipped.is_empty());

    // 链接名用标题 —— agent `ls books/` 要能一眼看出有哪几本。
    let books = fx.workspace(COL).join("books");
    assert!(books.join("注意力机制综述").join("ocr").join("document.v1.json").exists());
    assert!(books.join("扩散模型的采样加速").join("ocr").join("document.v1.json").exists());

    // 清单也要落盘：Python 那边不认识 collection，只会读这份 JSON。
    let raw = fs::read_to_string(fx.workspace(COL).join("collection.v1.json")).unwrap();
    assert!(raw.contains("retainpdf_collection_workspace_v1"));
    assert!(raw.contains("注意力机制综述"));
}

#[test]
fn books_without_a_translation_are_reported_not_silently_dropped() {
    // 沉默地少几本最糟：agent 以为自己看全了这个文件夹，然后给出一个
    // 「这几篇都没提到 X」的结论 —— 而提到 X 的那篇正好还没翻译完。
    let fx = Fixture::new();
    fx.book("doc-a", "翻译好的", "job-a");
    fx.book_without_translation("doc-b", "还没翻译的");
    fx.collect(COL, &["doc-a", "doc-b"]);

    let manifest = ensure_collection_workspace(&fx.db, &fx.root, COL, &|d| fx.reading_book(d)).unwrap();
    assert_eq!(manifest.books.len(), 1);
    assert_eq!(manifest.skipped.len(), 1, "被跳过的书没进清单");
    assert_eq!(manifest.skipped[0].title, "还没翻译的");
    assert!(!manifest.skipped[0].reason.is_empty(), "没说为什么跳过");

    let raw = fs::read_to_string(fx.workspace(COL).join("collection.v1.json")).unwrap();
    assert!(raw.contains("还没翻译的"), "跳过的书没写进清单，agent 看不见");
}

#[test]
fn refreshing_keeps_agent_output_and_repoints_links() {
    // 重译之后 active_job 会变。链接还指着旧 job 的话，agent 读到的是上一次的
    // 译文 —— 不报错，只是错。
    let fx = Fixture::new();
    fx.book("doc-a", "会被重译的书", "job-old");
    fx.collect(COL, &["doc-a"]);
    ensure_collection_workspace(&fx.db, &fx.root, COL, &|d| fx.reading_book(d)).unwrap();

    // agent 在工作区里干了活。
    let notes = fx.workspace(COL).join("notes-across-books.md");
    fs::write(&notes, "跨书笔记").unwrap();

    // 重译：新 job 成为 active。
    fx.book("doc-a", "会被重译的书", "job-new");
    let manifest = ensure_collection_workspace(&fx.db, &fx.root, COL, &|d| fx.reading_book(d)).unwrap();

    assert_eq!(manifest.books[0].job_id, "job-new", "还指着旧 job");
    let link = fx.workspace(COL).join("books").join("会被重译的书");
    assert_eq!(fs::read_link(&link).unwrap(), fx.root.join("jobs").join("job-new"));
    // 重建 books/ 不能顺手把 agent 的产出删了 —— 那是它攒下来的全部东西。
    assert_eq!(fs::read_to_string(&notes).unwrap(), "跨书笔记");
}

#[test]
fn same_titled_books_do_not_overwrite_each_other() {
    // 同一篇论文的两个版本标题一样。后一条盖掉前一条的话，agent `ls books/`
    // 看到一本，而清单里写着两本 —— 它会以为有个文件丢了。
    let fx = Fixture::new();
    fx.book("doc-a", "同名论文", "job-a");
    fx.book("doc-b", "同名论文", "job-b");
    fx.collect(COL, &["doc-a", "doc-b"]);

    let manifest = ensure_collection_workspace(&fx.db, &fx.root, COL, &|d| fx.reading_book(d)).unwrap();
    assert_eq!(manifest.books.len(), 2);
    let dirs: Vec<_> = manifest.books.iter().filter_map(|b| b.dir.clone()).collect();
    assert_eq!(dirs.len(), 2);
    assert_ne!(dirs[0], dirs[1], "两本书指向同一个链接名");
    for dir in &dirs {
        assert!(fx.workspace(COL).join("books").join(dir).exists());
    }
}

// ------------------------------------------------------- 链接名（纯逻辑，不碰磁盘）
//
// 这里**故意没有**走真实目录的那一版。写过，删了：把清洗改坏之后它是**挂住**
// 而不是变红（symlink 到一个解析出去的绝对路径上），在 CI 上表现成超时，不是
// 失败 —— 等于没有门禁。纯函数没有这个问题。

/// 标题来自 PDF 元数据，不是我们控制的。
///
/// **这几条刻意不走文件系统。** 同样的断言用真实目录做过一版，而把清洗改坏之后
/// 那一版是**挂住**而不是变红 —— 在 CI 上表现成超时，不是失败，等于没有门禁。
/// 纯函数没有这个问题：改坏了当场断言失败。
#[test]
fn a_title_that_looks_like_a_path_never_becomes_one() {
    let mut taken = std::collections::HashSet::new();
    for title in [
        "../../../etc/passwd",
        "/etc/passwd",
        "a/b/c",
        "..",
        ".",
        ".hidden",
        "....//....//x",
    ] {
        let name = link_name(title, "doc-0123456789", &mut taken);
        assert!(!name.contains('/'), "{title} -> {name}：名字里有路径分隔符");
        assert!(!name.contains('\\'), "{title} -> {name}：名字里有反斜杠");
        assert!(!name.starts_with('.'), "{title} -> {name}：名字以点开头");
        assert!(!name.is_empty(), "{title} -> 空名字");
        // 拼进目录之后不能跳出去 —— Path::join 遇到绝对路径会把基路径整个换掉。
        let joined = std::path::Path::new("/tmp/books").join(&name);
        assert_eq!(
            joined.parent(),
            Some(std::path::Path::new("/tmp/books")),
            "{title} -> {name}：拼出来的路径跑到 books/ 外面了"
        );
    }
}

#[test]
fn chinese_titles_survive_intact() {
    // 上一条那些检查很容易用「ASCII 白名单」一刀切满足,代价是中文标题全被削光,
    // `ls books/` 变成一排 book-1 book-2 —— 那就白做了。
    let mut taken = std::collections::HashSet::new();
    let name = link_name("注意力机制综述（第二版）", "doc-0123456789", &mut taken);
    assert_eq!(name, "注意力机制综述（第二版）");
}

#[test]
fn a_title_that_sanitizes_to_nothing_still_gets_a_name() {
    // 空名字会让链接建在 books/ 目录本身上。
    let mut taken = std::collections::HashSet::new();
    for title in ["", "   ", "...", "///", "..."] {
        let name = link_name(title, "doc-0123456789", &mut taken);
        assert!(!name.is_empty(), "「{title}」削成了空名字");
    }
}

/// 回归（状态审计 明细2 #7）：整本已经由一个成功任务覆盖，书卡又指着新提交、还在跑的任务时，
/// 工作区要链到成功的那个（阅读入口说的那个），而不是半成品目录。
#[test]
fn a_book_covered_by_one_job_links_the_finished_job_not_the_running_card() {
    use retainpdf_api_base::test_fixtures::minimal_pdf_bytes;
    use retainpdf_core::models::domain::{JobStatusKind, WorkflowKind};
    use retainpdf_files::DerivedArtifactDeps;

    let fx = Fixture::new();
    fx.book("doc-a", "重新翻译中的书", "job-new");
    fs::write(fx.root.join("uploads").join("doc-a.pdf"), minimal_pdf_bytes(595, 842)).unwrap();
    // job-new（running）是书卡指的；job-done 是已经成功、覆盖整本的那个。
    let rendered = fx.root.join("jobs/job-done/rendered");
    fs::create_dir_all(&rendered).unwrap();
    fs::write(rendered.join("out.pdf"), minimal_pdf_bytes(595, 842)).unwrap();
    let mut done = JobSnapshot::new("job-done".into(), CreateJobInput::default(), vec![]);
    done.status = JobStatusKind::Succeeded;
    done.workflow = WorkflowKind::Book;
    done.created_at = "2026-10-01T00:00:00".into();
    done.finished_at = Some("2026-10-01T01:00:00".into());
    done.artifacts = Some(JobArtifacts {
        job_root: Some("jobs/job-done".into()),
        output_pdf: Some("jobs/job-done/rendered/out.pdf".into()),
        ocr_page_numbers: vec![1],
        ..Default::default()
    });
    fx.db.save_job(&done).unwrap();
    fx.db.link_job_to_document("job-done", "upload-doc-a-job-new").unwrap();
    let mut running = fx.db.get_job("job-new").unwrap();
    running.status = JobStatusKind::Running;
    running.created_at = "2026-10-02T00:00:00".into();
    fx.db.save_job(&running).unwrap();
    fx.db.set_document_active_job("doc-a", "job-new", None).unwrap();
    fx.collect(COL, &["doc-a"]);

    let document = fx.db.get_document("doc-a").unwrap();
    let deps = DerivedArtifactDeps::with_pipeline_command("python3", "retainpdf-pipeline");
    let book = crate::reading::workspace_reading_book(&fx.db, &fx.root, deps, &document)
        .expect("one finished job covers the whole book");
    assert_eq!(book.job_id, "job-done", "工作区链到了书卡指的、还在跑的任务");
    assert!(book.root.ends_with("jobs/job-done"));

    let manifest = ensure_collection_workspace(&fx.db, &fx.root, COL, &|document| {
        let deps = DerivedArtifactDeps::with_pipeline_command("python3", "retainpdf-pipeline");
        crate::reading::workspace_reading_book(&fx.db, &fx.root, deps, document)
    })
    .unwrap();
    assert_eq!(manifest.books[0].job_id, "job-done");
    assert_eq!(manifest.books[0].job_root, "../../../jobs/job-done");
}

// ------------------------------------------------------- 书库接口审计 A6

fn status_of(error: retainpdf_api_base::error::AppError) -> StatusCode {
    error.into_response().status()
}

#[test]
fn collection_ids_must_have_the_generated_shape() {
    for _ in 0..50 {
        let id = crate::collections::new_collection_id();
        assert!(is_valid_collection_id(&id), "生成器生成的 {id} 没通过校验");
    }
    assert!(is_valid_collection_id(COL));
    for id in [
        "",
        "..",
        ".",
        "../ai",
        "../../etc",
        "/etc",
        "col-1",
        "col-",
        "col-merged",
        "col-20260923000000-ABCDEF",
        "col-20260923000000-abcde",
        "col-20260923000000-abcdef0",
        "col-2026092300000x-abcdef",
        "col-20260923000000_abcdef",
        "col-20260923000000-abcdef/..",
        "col-20260923000000-abcdef\0",
        " col-20260923000000-abcdef",
        "COL-20260923000000-abcdef",
        "col-２０２６0923000000-abcdef",
    ] {
        assert!(!is_valid_collection_id(id), "{id:?} 不该通过校验");
    }
}

/// 回归（A6）：`..` 这种编号以前拼成 `<data_root>/collections/../ai`，对数据目录下的 `ai/books`
/// 做 `remove_dir_all` 再重建。现在 400，磁盘一点不碰。
#[test]
fn an_invalid_id_is_rejected_before_touching_the_disk() {
    let fx = Fixture::new();
    fs::create_dir_all(fx.root.join("collections")).unwrap();
    let decoy = fx.root.join("ai/books");
    fs::create_dir_all(&decoy).unwrap();
    fs::write(decoy.join("keep.txt"), b"agent output").unwrap();

    for id in ["..", "../collections/..", "col-20260923000000-0c0c01/..", "col-1"] {
        let error = ensure_collection_workspace(&fx.db, &fx.root, id, &|d| fx.reading_book(d))
            .err()
            .unwrap_or_else(|| panic!("{id:?} 物化成功了"));
        assert_eq!(status_of(error), StatusCode::BAD_REQUEST, "{id:?}");
        assert!(collection_workspace_dir(&fx.root, id).is_err(), "{id:?} 拼出了路径");
        remove_collection_dir(&fx.root, id);
        assert!(decoy.join("keep.txt").is_file(), "{id:?} 碰到了 collections/ 外面的 ai/books");
        assert!(!fx.root.join("ai/collection.v1.json").exists(), "{id:?} 在 collections/ 外面写了清单");
    }
}

#[test]
fn an_unknown_collection_is_not_found_and_creates_nothing() {
    let fx = Fixture::new();
    let error = ensure_collection_workspace(&fx.db, &fx.root, COL, &|d| fx.reading_book(d))
        .err()
        .expect("不存在的合集物化成功了");
    assert_eq!(status_of(error), StatusCode::NOT_FOUND);
    assert!(!fx.root.join("collections").join(COL).exists(), "不存在的合集建出了目录");
}

/// 回归（正确性审计 明细2 #7 的后一半）：书还没有可读的译文时，以前退回书卡的 `active_job_id`，
/// 那个任务可能还在跑，链过去是半成品目录。现在直接进 `skipped`。
#[test]
fn a_book_without_a_readable_translation_is_skipped_even_when_the_card_points_at_a_job() {
    let fx = Fixture::new();
    fx.book("doc-a", "刚提交翻译的书", "job-running");
    fx.collect(COL, &["doc-a"]);

    let manifest = ensure_collection_workspace(&fx.db, &fx.root, COL, &|_| None).unwrap();
    assert!(manifest.books.is_empty(), "链到了书卡指的任务（可能是在跑的半成品）");
    assert_eq!(manifest.skipped.len(), 1);
    assert_eq!(manifest.skipped[0].document_id, "doc-a");
    assert_eq!(manifest.skipped[0].reason, "还没有可读的译文");
    let links: Vec<_> = fs::read_dir(fx.workspace(COL).join("books")).unwrap().collect();
    assert!(links.is_empty(), "books/ 下建了链接");
}

/// 超过上限的书以前悄悄少掉（只读前 200 本），现在写进 `skipped` 并说明原因。
#[test]
fn books_over_the_cap_are_reported_in_skipped() {
    let fx = Fixture::new();
    fx.book("doc-a", "第一本", "job-a");
    fx.book("doc-b", "第二本", "job-b");
    fx.book("doc-c", "第三本", "job-c");
    fx.collect(COL, &["doc-a", "doc-b", "doc-c"]);

    let manifest =
        materialize_collection_workspace(&fx.db, &fx.root, COL, &|d| fx.reading_book(d), 2).unwrap();
    assert_eq!(manifest.books.len(), 2);
    assert_eq!(manifest.skipped.len(), 1, "超过上限的书没进 skipped");
    assert!(manifest.skipped[0].reason.contains("上限"), "{}", manifest.skipped[0].reason);
    let mut seen: Vec<_> = manifest
        .books
        .iter()
        .map(|book| book.document_id.clone())
        .chain(manifest.skipped.iter().map(|skip| skip.document_id.clone()))
        .collect();
    seen.sort();
    assert_eq!(seen, ["doc-a", "doc-b", "doc-c"]);
}

/// 删合集清目录：`books/` 下是指向任务目录的符号链接，清掉的只能是链接，任务的产物要还在。
#[test]
fn removing_the_collection_directory_keeps_the_linked_job_outputs() {
    let fx = Fixture::new();
    let job_root = fx.book("doc-a", "要留下的书", "job-a");
    fx.collect(COL, &["doc-a"]);
    ensure_collection_workspace(&fx.db, &fx.root, COL, &|d| fx.reading_book(d)).unwrap();
    assert!(fx.workspace(COL).join("books/要留下的书/ocr/document.v1.json").is_file());

    remove_collection_dir(&fx.root, COL);
    assert!(!fx.root.join("collections").join(COL).exists(), "合集目录没清掉");
    assert!(job_root.join("ocr/document.v1.json").is_file(), "清合集目录时顺着链接删了任务产物");
    // 再清一次（目录已经没了）不报错。
    remove_collection_dir(&fx.root, COL);
}
