//! 管理后台保存的平台设置在建任务时现读：保存后下一个任务就用新的，不用重启。

use crate::config::PlatformModels;
use crate::models::{now_iso, UploadRecord, WorkflowKind};
use crate::services::platform_settings::{
    self, PlatformSource, SaveOcrSettings, SavePlatformSettingsInput, SaveTranslationSettings,
};
use crate::test_support::pdf::build_test_pdf_bytes;

use super::submit::create_translation_job;
use super::tests::{base_translation_input, submit_context, test_state};

fn settings(model: &str, api_key: Option<&str>) -> SavePlatformSettingsInput {
    SavePlatformSettingsInput {
        translation: SaveTranslationSettings {
            model: model.into(),
            base_url: "https://api.example.com/v1".into(),
            api_protocol: "openai".into(),
            thinking: "low".into(),
            max_workers: 3,
            api_key: api_key.map(Into::into),
        },
        ocr: SaveOcrSettings {
            provider: "mineru".into(),
            token: api_key.map(|_| "mineru-platform-token".into()),
        },
    }
}

#[test]
fn saved_platform_settings_apply_to_the_next_job_without_restart() {
    let state = test_state("platform-live");
    let env = PlatformModels::default();
    let upload_dir = state.config.uploads_dir.join("up-1");
    std::fs::create_dir_all(&upload_dir).unwrap();
    let path = upload_dir.join("input.pdf");
    std::fs::write(&path, build_test_pdf_bytes()).unwrap();
    state
        .db
        .save_upload(&UploadRecord {
            upload_id: "up-1".into(),
            filename: "input.pdf".into(),
            stored_path: path.to_string_lossy().into_owned(),
            bytes: 1,
            page_count: 1,
            uploaded_at: now_iso(),
            developer_mode: false,
            content_hash: String::new(),
        })
        .unwrap();
    let runtime = state.config.provider_runtime.deepseek.clone();
    let save = |input| {
        platform_settings::save(
            &state.db,
            &state.config.data_root,
            &env,
            &runtime,
            "u_admin",
            input,
        )
        .unwrap()
    };
    let mut deps = submit_context(&state);
    deps.launcher = deps.launcher.with_platform(Some(PlatformSource {
        db: state.db.as_ref(),
        env: &env,
    }));
    let mut input = base_translation_input(WorkflowKind::Book);
    input.source.upload_id = "up-1".into();
    input.translation.workers = 20;

    save(settings("platform-model-a", Some("sk-platform-key-1234")));
    let first = create_translation_job(&deps, &input).expect("first job");
    let t = &first.request_payload.translation;
    assert_eq!(
        (t.model.as_str(), t.thinking.as_str(), t.workers),
        ("platform-model-a", "low", 3)
    );
    assert!(
        t.api_key.is_empty() && !t.credential_ref.is_empty(),
        "只带凭据编号，不带明文"
    );

    save(settings("platform-model-b", None));
    let second = create_translation_job(&deps, &input).expect("second job");
    assert_eq!(second.request_payload.translation.model, "platform-model-b");
    assert_eq!(
        first.request_payload.translation.model, "platform-model-a",
        "已提交的任务不变"
    );
    assert_eq!(
        second.request_payload.translation.credential_ref, t.credential_ref,
        "没换密钥就沿用同一条凭据"
    );
}
