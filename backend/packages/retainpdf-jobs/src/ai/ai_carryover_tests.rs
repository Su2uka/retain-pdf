//! 重译之后 agent 工作区的接力。
//!
//! 起因：`<job>/ai/` 下的产物挂在 job 上。重新翻译一本书就开一个新 job，新
//! job 的 `ai/` 是空的 —— 用户在这本书上攒的画板文件、笔记
//! **一声不响地留在旧 job 里**，界面上和「从没让 agent 干过活」长得一样。
//!
//! 这里守四件事，每一件反过来都是一个安静的数据事故：
//! - 接得过来（不然这个修复等于没做）
//! - **不覆盖新 job 上已有的产出**（覆盖 = 直接销毁用户的工作）
//! - 只接同一本书的（接错书 = 把别人的概念图贴到这本书上）
//! - 不跟随符号链接（`ai/` 是 agent 可写目录）

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use retainpdf_data::db::Db;
use retainpdf_core::models::domain::{
    CreateJobInput, JobArtifacts, JobSnapshot, UploadRecord,
};
use retainpdf_files::DownloadGeneration;

use crate::files::downloads::JobDownloads;

const DOCUMENT_ID: &str = "sha256-of-the-very-same-pdf";

struct Fixture {
    root: PathBuf,
    db: Db,
    generation: Arc<DownloadGeneration>,
}

impl Fixture {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("retain-ai-carryover-{}", fastrand::u64(..)));
        fs::create_dir_all(root.join("uploads")).unwrap();
        let db = Db::new(root.join("jobs.db"), root.clone());
        db.init().unwrap();
        Self {
            root,
            db,
            generation: Arc::new(DownloadGeneration::default()),
        }
    }

    /// 建一个 job 并把它归属到 `document_id`。
    ///
    /// 走的是产品里真正的那条路 —— `save_upload_with_document` +
    /// `link_job_to_document` —— 而不是直接 UPDATE jobs。直接写表的话，
    /// 归属关系哪天换个存法，这些测试会继续绿着骗人。
    fn job(&self, job_id: &str, document_id: &str) -> PathBuf {
        let job_root = self.root.join("jobs").join(job_id);
        fs::create_dir_all(&job_root).unwrap();
        let mut job = JobSnapshot::new(job_id.into(), CreateJobInput::default(), vec![]);
        job.artifacts = Some(JobArtifacts {
            job_root: Some(format!("jobs/{job_id}")),
            ..Default::default()
        });
        self.db.save_job(&job).unwrap();

        let upload_id = format!("upload-for-{document_id}");
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
                page_count: 1,
                uploaded_at: "2026-09-23T00:00:00Z".into(),
                developer_mode: false,
                content_hash: document_id.into(),
            })
            .unwrap();
        self.db
            .link_job_to_document(job_id, &upload_id)
            .unwrap()
            .expect("job 应该被归属到 document");
        job_root
    }

    /// 往一个 job 的 `ai/` 里放一套「agent 干过活」的现场。
    fn seed_workspace(&self, job_root: &Path) -> PathBuf {
        let ai = job_root.join("ai");
        fs::create_dir_all(ai.join("board")).unwrap();
        fs::write(ai.join("notes.md"), "旧笔记").unwrap();
        fs::write(ai.join("board").join("chart.png"), b"fake png").unwrap();
        // agent 自己写的脚本也是它的工作，要一起搬。
        fs::write(ai.join("probe.sh"), b"#!/bin/sh\necho hi\n").unwrap();
        // 这份是 Python 每次起终端按当前模板重新生成的，属于生成物。
        fs::write(ai.join("AGENTS.md"), "# 旧模板").unwrap();
        ai
    }

    fn downloads(&self) -> JobDownloads<'_> {
        JobDownloads::new(
            &self.db,
            &self.root,
            &self.root,
            &self.generation,
            "unused-python",
            "unused-pipeline",
        )
    }

    /// 打一次终端条会轮询的那个端点（画板列表），返回列表里的文件名。
    ///
    /// 列表没有东西也是 200，所以「没接力过来」必须断文件系统，断 `Err` 会永远为假。
    async fn fetch_board(&self, job_id: &str) -> Vec<String> {
        self.downloads()
            .ai_board_listing(job_id)
            .await
            .expect("读画板列表不该失败")
            .items
            .into_iter()
            .map(|item| item.name)
            .collect()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[tokio::test]
async fn carries_previous_workspace_to_the_retranslated_job() {
    let fx = Fixture::new();
    let old = fx.job("old-job", DOCUMENT_ID);
    fx.seed_workspace(&old);
    let new = fx.job("new-job", DOCUMENT_ID);

    // 这一次请求本身就该看到接过来的画板文件，不用等下一轮轮询。
    assert_eq!(fx.fetch_board("new-job").await, vec!["chart.png".to_string()]);

    let ai = new.join("ai");
    assert_eq!(fs::read_to_string(ai.join("notes.md")).unwrap(), "旧笔记");
    assert!(ai.join("board").join("chart.png").exists(), "画板文件没接过来");
    assert!(ai.join("probe.sh").exists(), "agent 自己写的脚本没接过来");
    // 来源留痕：现场排查时这是唯一能回答「这些概念图哪来的」的东西。
    assert_eq!(
        fs::read_to_string(ai.join(".carried-from")).unwrap().trim(),
        "old-job"
    );
}

#[tokio::test]
async fn leaves_the_generated_agents_md_behind() {
    // 模板在演进（实测同一台机器上两个 job 的 AGENTS.md 是 4685 和 7328 字节）。
    // 搬一份旧的过去，新 job 里就躺着一份过期说明，而 Python 未必会覆盖它。
    let fx = Fixture::new();
    fx.seed_workspace(&fx.job("old-job", DOCUMENT_ID));
    let new = fx.job("new-job", DOCUMENT_ID);
    fx.fetch_board("new-job").await;
    // 先确认接力真的发生了 —— 否则「AGENTS.md 不在」在「什么都没搬」时
    // 也成立，这条就成了永远绿的假门禁。
    assert!(new.join("ai").join("notes.md").exists(), "根本没接力");
    assert!(!new.join("ai").join("AGENTS.md").exists(), "旧模板被搬过来了");
}

#[tokio::test]
async fn never_overwrites_work_already_done_on_this_job() {
    // 这条要是破了，用户在新 job 上刚写的概念图会被旧 job 的版本静默盖掉 ——
    // 比原本那个 bug 严重得多：原来只是看不到，这里是真的销毁。
    let fx = Fixture::new();
    fx.seed_workspace(&fx.job("old-job", DOCUMENT_ID));
    let new = fx.job("new-job", DOCUMENT_ID);
    fs::create_dir_all(new.join("ai")).unwrap();
    fs::write(new.join("ai").join("notes.md"), "新笔记").unwrap();

    fx.fetch_board("new-job").await;
    assert_eq!(fs::read_to_string(new.join("ai").join("notes.md")).unwrap(), "新笔记");
    assert!(
        !new.join("ai").join("probe.sh").exists(),
        "这个 job 已经在用了，不该再往里灌旧产物"
    );
}

#[tokio::test]
async fn carries_over_once_so_deletions_stick() {
    // 没有标记的话：用户删掉接过来的概念图 → 下一次轮询发现 ai/ 里只剩 AGENTS.md
    // → 又接一遍。删除操作永远不生效，而且没有任何报错。
    let fx = Fixture::new();
    fx.seed_workspace(&fx.job("old-job", DOCUMENT_ID));
    let new = fx.job("new-job", DOCUMENT_ID);
    fx.fetch_board("new-job").await;

    fs::remove_dir_all(new.join("ai").join("board")).unwrap();
    fs::remove_file(new.join("ai").join("notes.md")).unwrap();
    fs::remove_file(new.join("ai").join("probe.sh")).unwrap();

    fx.fetch_board("new-job").await;
    assert!(!new.join("ai").join("notes.md").exists(), "删掉的笔记又长回来了");
    assert!(!new.join("ai").join("board").exists(), "删掉的画板又长回来了");
}

#[tokio::test]
async fn never_carries_over_from_a_different_document() {
    // 接错书 = 把另一本书的概念图贴到这本书的页面坐标上。界面不会报错，
    // 只是每一条都指向错的地方。
    let fx = Fixture::new();
    fx.seed_workspace(&fx.job("other-book-job", "sha256-of-a-different-pdf"));
    let new = fx.job("new-job", DOCUMENT_ID);

    fx.fetch_board("new-job").await;
    assert!(!new.join("ai").join("notes.md").exists(), "接了别的书的笔记");
    assert!(!new.join("ai").join(".carried-from").exists(), "标记都写下了，说明真接了");
}

#[tokio::test]
async fn skips_jobs_with_no_document() {
    // 归属是建完 job 之后补写的，也可能压根没补上。这时候「同一本书」无从谈起，
    // 只能什么都不做 —— 绝不能退化成「随便找个最近的 job 接过来」。
    let fx = Fixture::new();
    fx.seed_workspace(&fx.job("old-job", DOCUMENT_ID));

    let orphan_root = fx.root.join("jobs").join("orphan-job");
    fs::create_dir_all(&orphan_root).unwrap();
    let mut job = JobSnapshot::new("orphan-job".into(), CreateJobInput::default(), vec![]);
    job.artifacts = Some(JobArtifacts {
        job_root: Some("jobs/orphan-job".into()),
        ..Default::default()
    });
    fx.db.save_job(&job).unwrap();

    fx.fetch_board("orphan-job").await;
    assert!(!orphan_root.join("ai").join("notes.md").exists());
    assert!(!orphan_root.join("ai").join(".carried-from").exists());
}

#[tokio::test]
async fn does_not_follow_symlinks() {
    // `ai/` 是 agent 可写目录。跟随链接就等于 `ln -s /etc/passwd board/x.png`
    // 能把任意文件复制进新 job 的可读目录 —— 而那个目录前端是能列出来的。
    let fx = Fixture::new();
    let old = fx.job("old-job", DOCUMENT_ID);
    let ai = fx.seed_workspace(&old);
    let secret = fx.root.join("secret.txt");
    fs::write(&secret, "不该被搬走的东西").unwrap();
    symlink(&secret, ai.join("board").join("leak.png")).unwrap();

    let new = fx.job("new-job", DOCUMENT_ID);
    fx.fetch_board("new-job").await;

    let leaked = new.join("ai").join("board").join("leak.png");
    assert!(!leaked.exists(), "符号链接被跟着复制了");
    // 正常文件照搬 —— 免得这条测试被「干脆不搬 board」蒙混过去。
    assert!(new.join("ai").join("board").join("chart.png").exists());
}

// ── 合并书的文档级工作区 ───────────────────────────────────────────────────

fn seed_document_workspace(fx: &Fixture, notes: &str) -> PathBuf {
    let ai = fx.root.join("documents").join(DOCUMENT_ID).join("ai");
    fs::create_dir_all(&ai).unwrap();
    fs::write(ai.join("notes.md"), notes).unwrap();
    ai
}

fn age(path: &Path, seconds_ago: u64) {
    let when = std::time::SystemTime::now() - std::time::Duration::from_secs(seconds_ago);
    let file = fs::File::options().write(true).open(path).unwrap();
    file.set_modified(when).unwrap();
}

#[tokio::test]
async fn a_single_translation_picks_up_newer_work_done_on_the_merged_book() {
    // 用户在合并书上整理了笔记，之后单独打开某一次翻译：接合并书上那份更新的。
    let fx = Fixture::new();
    let old = fx.seed_workspace(&fx.job("old-job", DOCUMENT_ID));
    age(&old.join("notes.md"), 3600);
    age(&old.join("probe.sh"), 3600);
    age(&old.join("board").join("chart.png"), 3600);
    seed_document_workspace(&fx, "合并书上的笔记");
    let new = fx.job("new-job", DOCUMENT_ID);

    fx.fetch_board("new-job").await;
    assert_eq!(fs::read_to_string(new.join("ai").join("notes.md")).unwrap(), "合并书上的笔记");
}

#[tokio::test]
async fn an_older_merged_workspace_does_not_beat_newer_work_on_a_job() {
    let fx = Fixture::new();
    let document_ai = seed_document_workspace(&fx, "合并书上的旧笔记");
    age(&document_ai.join("notes.md"), 3600);
    fx.seed_workspace(&fx.job("old-job", DOCUMENT_ID));
    let new = fx.job("new-job", DOCUMENT_ID);

    fx.fetch_board("new-job").await;
    assert_eq!(fs::read_to_string(new.join("ai").join("notes.md")).unwrap(), "旧笔记");
}

// ── 轮询的代价 ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn repeated_reads_do_no_further_copying() {
    // 画板列表会被前端每几秒轮询。接力只在第一次读时发生，之后只看一眼标记：
    // 来源上后来多出来的文件不会再被搬过来，标记也不会被重写。
    let fx = Fixture::new();
    let old = fx.job("old-job", DOCUMENT_ID);
    fx.seed_workspace(&old);
    let new = fx.job("new-job", DOCUMENT_ID);
    fx.fetch_board("new-job").await;
    let marker = new.join("ai").join(".carried-from");
    age(&marker, 3600);
    let marker_modified = fs::metadata(&marker).unwrap().modified().unwrap();

    fs::write(old.join("ai").join("later.md"), "接力之后才写的").unwrap();
    for _ in 0..3 {
        fx.fetch_board("new-job").await;
    }
    assert!(!new.join("ai").join("later.md").exists(), "重复读又复制了一遍");
    assert_eq!(
        fs::metadata(&marker).unwrap().modified().unwrap(),
        marker_modified,
        "重复读又重写了标记"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn listing_does_not_block_the_async_thread() {
    // 列表和接力都是同步磁盘活。单线程运行时里，另起的任务要能在列表等待期间跑起来，
    // 说明磁盘活没有占着异步线程（直接同步调用的话，这个 future 一次 poll 就做完，
    // 别的任务根本轮不到）。
    use std::sync::atomic::{AtomicBool, Ordering};
    let fx = Fixture::new();
    fx.seed_workspace(&fx.job("old-job", DOCUMENT_ID));
    fx.job("new-job", DOCUMENT_ID);

    let ran = Arc::new(AtomicBool::new(false));
    let flag = ran.clone();
    tokio::spawn(async move { flag.store(true, Ordering::SeqCst) });
    fx.fetch_board("new-job").await;
    assert!(ran.load(Ordering::SeqCst), "列表在异步线程里同步做完了，别的任务没机会跑");
}
