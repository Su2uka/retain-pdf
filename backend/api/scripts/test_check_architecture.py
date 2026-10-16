"""Synthetic-source regression tests; no Rust build or real configuration reads."""
from __future__ import annotations

import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location(
    "api_architecture_checks", Path(__file__).with_name("check_architecture.py")
)
assert spec and spec.loader
checks = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checks)


class UploadBoundaryTests(unittest.TestCase):
    def test_persistence_only_runner_leaves_accept_only_narrow_dependencies(self):
        for name in (
            "ocr_flow/bundle_events.rs", "render_flow_artifacts.rs",
            "translation_flow_artifacts.rs", "render_flow_artifacts/prepare.rs",
        ):
            path = f"job_runner/{name}"
            self.assertEqual([], self.check_sources(
                {path: "fn prepare(deps: &JobPersistDeps) { let db = &deps.db; }"},
                checks.check_job_persist_deps_usage,
            ))
            for capability in ("ProcessRuntimeDeps", "AppState", "AppConfig", "RuntimeControl", "Semaphore"):
                with self.subTest(path=path, capability=capability):
                    self.assertTrue(self.check_sources(
                        {path: f"use crate::job_runner::{{{capability} as Runtime}};"},
                        checks.check_job_persist_deps_usage,
                    ))
        self.assertTrue(self.check_sources(
            {"services/jobs/unrelated.rs": "struct Wide { persist: JobPersistDeps }"},
            checks.check_job_persist_deps_usage,
        ))
        self.assertEqual([], self.check_sources({
            "job_runner/process_runner/io_support/events.rs": "fn emit(deps: &JobPersistDeps) {}",
            "job_runner/process_runner/startup/mod.rs": "fn start(deps: &JobPersistDeps) {}",
            "job_runner/process_runner/persistence_tests.rs": "let deps: JobPersistDeps;",
        }, checks.check_job_persist_deps_usage))

    def test_persist_leaf_guard_ignores_documentation_literals_and_local_tests(self):
        self.assertEqual([], self.check_sources({"job_runner/ocr_flow/bundle_events.rs": '''
            // Formerly ProcessRuntimeDeps; AppConfig is deliberately absent.
            const NOTE: &str = r#"RuntimeControl"#;
            fn emit(deps: &JobPersistDeps) {}
#[cfg(test)]
mod tests { fn fixture() { let runtime: ProcessRuntimeDeps; } }
        '''}, checks.check_job_persist_deps_usage))

    def test_shared_artifact_display_cannot_reintroduce_business_cycles(self):
        self.assertEqual([], self.check_sources({
            "services/jobs/presentation/detail_projection.rs": "use crate::services::files::artifacts::build_artifacts_display;",
            "services/library/book_projection.rs": "use crate::services::files::artifacts::build_artifacts_display; use crate::services::jobs::job_readiness;",
            "services/files/artifacts/presentation.rs": "use crate::models::api::{ArtifactLinksView, ArtifactDisplayItemView};",
            "services/jobs/event_feed/batch_tests.rs": "use crate::services::library::book_projection;",
        }, checks.check_shared_artifact_presentation))
        for path in ("services/jobs/presentation/detail_projection.rs", "services/jobs/query/nested.rs"):
            self.assertTrue(self.check_sources(
                {path: "use crate::services::{book_projection::build_artifacts_display};"},
                checks.check_shared_artifact_presentation,
            ))
        for source in (
            "use crate::services::jobs;", "use crate::services::{library};", "fn build(db: &Db) {}",
            "std::fs::read(path);", "use crate::ocr_provider;", "use crate::storage_paths;",
            "use std::{fs as files};", "tokio::process::Command::new(program);",
            'let raw = payload["layoutParsingResults"];',
        ):
            self.assertTrue(self.check_sources(
                {"services/files/artifacts/presentation/nested.rs": source},
                checks.check_shared_artifact_presentation,
            ))
        self.assertTrue(self.check_sources(
            {"services/library/book_projection/artifacts.rs": ""},
            checks.check_shared_artifact_presentation,
        ))

    def test_shared_artifact_guard_ignores_comments_and_literals(self):
        self.assertEqual([], self.check_sources({
            "services/files/artifacts/presentation.rs": '// no jobs or Db access\nconst LABEL: &str = "library";',
            "services/jobs/query.rs": 'const NOTE: &str = r#"book_projection"#;',
        }, checks.check_shared_artifact_presentation))

    def test_jobs_dependencies_allow_explicit_query_resources(self):
        self.assertEqual([], self.check_sources({
            "services/jobs/deps/query.rs": "use crate::db::Db; use super::ReplayDeps; struct QueryJobsDeps;",
            "services/jobs/deps/command.rs": "use crate::services::ingest::uploads::UploadService;",
        }, checks.check_jobs_dependency_boundaries))

    def test_jobs_dependencies_reject_wide_state_and_command_in_query(self):
        for source in ("AppState", "AppConfig", "env::var(\"ROOT\")", "Config::from_env()",
                       "CommandJobsDeps", "JobSubmitDeps", "JobRuntime", "RuntimeControl",
                       "JobLaunchDeps", "UploadService"):
            with self.subTest(source=source):
                self.assertTrue(self.check_sources({"services/jobs/deps/query.rs": source},
                                                   checks.check_jobs_dependency_boundaries))
        self.assertTrue(self.check_sources({"services/jobs/lifecycle/creation/context.rs": ""},
                                           checks.check_jobs_dependency_boundaries))

    def test_runtime_ownership_allows_assembly_and_health(self):
        self.assertEqual([], self.check_sources({
            "runtime/ai_supervisor.rs": "use crate::config::AppConfig;",
            "app/server.rs": "crate::runtime::ai_supervisor::spawn_ai_supervisor();",
            "routes/common/health.rs": "crate::runtime::jobsd_supervisor::jobsd_status();",
        }, checks.check_runtime_ownership))

    def test_basic_job_queries_cannot_acquire_execution_dependencies(self):
        path = "services/jobs/query.rs"
        for source in ("AppState", "AppConfig", "Config::from_env()", 'env::var("ROOT")',
                       "JobsFacade", "CommandJobsDeps", "JobSubmitDeps", "JobRuntime",
                       "RuntimeControl", "JobLaunchDeps", "UploadService"):
            with self.subTest(source=source):
                self.assertTrue(self.check_sources({path: source},
                                                   checks.check_jobs_dependency_boundaries))
        self.assertEqual([], self.check_sources(
            {path: "struct JobQueries<'a> { db: &'a Db, data_root: &'a Path }"},
            checks.check_jobs_dependency_boundaries,
        ))

    def test_basic_job_read_routes_use_query_only_builder(self):
        path = "routes/jobs/reports.rs"
        for builder in ("build_jobs_route_deps", "build_jobs_facade_from_state", "jobs_facade"):
            with self.subTest(builder=builder):
                self.assertTrue(self.check_sources(
                    {path: f"let deps = {builder}(&state);"}, checks.check_jobs_route_deps_dedup,
                ))
        self.assertEqual([], self.check_sources(
            {path: "let deps = build_jobs_query_route_deps(&state);"},
            checks.check_jobs_route_deps_dedup,
        ))

    def test_narrow_job_service_boundaries_follow_module_splits(self):
        for path in (
            "services/jobs/query/nested.rs", "services/jobs/deps/query/nested.rs",
            "services/jobs/files/downloads.rs", "services/jobs/files/downloads/deps.rs",
            "services/jobs/files/downloads/files/cache.rs", "services/jobs/files/queries.rs",
            "services/jobs/lifecycle/queries.rs", "services/jobs/progress/queries.rs",
            "services/jobs/reader/queries.rs", "services/jobs/reports/queries.rs",
            "services/jobs/translations/queries.rs", "services/jobs/reports/failure.rs",
            "services/jobs/actions/resume_plan.rs", "services/jobs/reader/markdown.rs",
            "services/jobs/ai/ai_board.rs",
        ):
            for capability in ("AppState", "AppConfig", "JobsFacade", "JobRuntimeLauncher", "UploadService"):
                with self.subTest(path=path, capability=capability):
                    self.assertTrue(self.check_sources({path: capability}, checks.check_jobs_dependency_boundaries))
        for path in ("services/jobs/reports/failure.rs", "services/jobs/files/downloads/deps.rs"):
            for capability in ("ReplayDeps", "QueryJobsDeps"):
                self.assertTrue(self.check_sources({path: capability}, checks.check_jobs_dependency_boundaries))
        self.assertEqual([], self.check_sources({
            "services/jobs/query/reader.rs": "struct ReadDeps<'a> { db: &'a Db, events: &'a JobEventFeed }",
            "services/jobs/files/downloads/deps.rs": "struct DownloadJobsDeps { scheduler: DownloadGenerationScheduler }",
            "services/jobs/deps/query.rs": "struct QueryJobsDeps<'a> { replay: ReplayDeps<'a> }",
        }, checks.check_jobs_dependency_boundaries))

    def test_narrow_job_route_boundaries_follow_module_splits(self):
        for path in (
            "routes/jobs/files.rs", "routes/jobs/files/legacy.rs", "routes/jobs/reports.rs",
            "routes/jobs/reports/quality.rs", "routes/jobs/reader.rs", "routes/jobs/data.rs",
            "routes/jobs/ai.rs", "routes/download_response/files.rs",
            "routes/download_response/markdown/cache.rs",
        ):
            with self.subTest(path=path):
                self.assertTrue(self.check_sources(
                    {path: "let deps = build_jobs_route_deps(&state);"}, checks.check_jobs_route_deps_dedup,
                ))

    def test_mixed_job_routes_keep_only_explicit_commands_wide(self):
        for path, reader, command in (
            ("routes/jobs/lifecycle.rs", "get_job", "create_job"),
            ("routes/jobs/lifecycle/read.rs", "list_jobs", "create_ocr_job"),
            ("routes/jobs/actions.rs", "get_resume_plan", "retry_stage"),
            ("routes/jobs/progress.rs", "get_job_events", "get_live_translation_layout"),
            ("routes/jobs/translations.rs", "get_translation_item", "replay_translation_item_route"),
        ):
            source = f"""
                use crate::routes::common::{{build_jobs_route_deps, build_jobs_query_route_deps}};
                pub async fn {reader}() {{
                    if ready {{ return build_jobs_query_route_deps(&state); }}
                    build_jobs_query_route_deps(&state)
                }}
                pub async fn {command}() {{ build_jobs_route_deps(&state); }}
            """
            with self.subTest(path=path):
                self.assertEqual([], self.check_sources({path: source}, checks.check_jobs_route_deps_dedup))
                self.assertTrue(self.check_sources(
                    {path: source.replace("return build_jobs_query_route_deps", "return build_jobs_route_deps")},
                    checks.check_jobs_route_deps_dedup,
                ))
                self.assertTrue(self.check_sources(
                    {path: source + "fn local_helper() { build_jobs_route_deps(&state); }"},
                    checks.check_jobs_route_deps_dedup,
                ))

    def test_mixed_job_routes_detect_full_builder_import_aliases(self):
        self.assertTrue(self.check_sources({"routes/jobs/lifecycle.rs": """
            use crate::routes::common::build_jobs_route_deps as full;
            pub async fn get_job() { full(&state); }
        """}, checks.check_jobs_route_deps_dedup))

    def test_job_boundaries_ignore_comments_and_rust_literals(self):
        notes = '''
            // migrated from build_jobs_route_deps; AppState must stay out
            /* JobsFacade /* nested UploadService */ JobRuntimeLauncher */
            const URL: &str = "https://example.com/JobsFacade";
            const NOTE: &str = r##"raw } build_jobs_route_deps /* AppState */"##;
            const BRACE: char = '}';
        '''
        self.assertEqual([], self.check_sources({
            "routes/jobs/lifecycle.rs": notes + "fn list_jobs() { build_jobs_query_route_deps(&state); }",
        }, checks.check_jobs_route_deps_dedup))
        self.assertEqual([], self.check_sources({
            "services/jobs/query/nested.rs": notes + "struct JobQueries<'a> { db: &'a Db }",
        }, checks.check_jobs_dependency_boundaries))
        self.assertEqual([], self.check_sources({
            "services/jobs/query/nested.rs": notes + "struct JobQueries<'a> { db: &'a Db }",
        }, checks.check_appstate_boundaries))
        self.assertEqual([], self.check_sources({
            "routes/jobs/lifecycle.rs": "/* use crate::services::jobs::JobsFacade; */",
        }, checks.check_route_service_imports))
        self.assertTrue(self.check_sources({
            "services/jobs/query/nested.rs": notes + "struct Bad { runtime: JobRuntime }",
        }, checks.check_jobs_dependency_boundaries))
        self.assertTrue(self.check_sources({"routes/jobs/lifecycle.rs": notes + '''
            async fn get_job() {
                let ignored = r#"} fn create_job() {"#;
                build_jobs_route_deps(&state)
            }
        '''}, checks.check_jobs_route_deps_dedup))

    def test_runtime_ownership_rejects_reverse_dependencies_and_old_locations(self):
        for path, source in (
            ("runtime/jobsd_supervisor.rs", "use crate::services::jobs;"),
            ("runtime/ai_supervisor.rs", "use crate::AppState;"),
            ("routes/health.rs", "crate::runtime::ai_supervisor::ai_service_status();"),
            ("services/jobs/facade.rs", "crate::runtime::jobsd_supervisor::jobsd_status();"),
            ("services/ai_supervisor.rs", ""),
        ):
            with self.subTest(path=path):
                self.assertTrue(self.check_sources({path: source}, checks.check_runtime_ownership))

    def test_ai_gateway_has_no_hidden_configuration_or_health_reads(self):
        for source in ("AiProxyConfig::from_env()", "std::env::var(\"BASE\")", "ai_service_status()", "Lazy<Client>"):
            with self.subTest(source=source):
                self.assertTrue(self.check_sources(
                    {"services/ai/gateway.rs": source}, checks.check_ai_gateway_dependencies,
                ))
        self.assertEqual([], self.check_sources(
            {"services/ai/gateway.rs": "fn new(config: &AiProxyConfig, status: fn() -> u8) {}"},
            checks.check_ai_gateway_dependencies,
        ))

    def test_agent_calculations_use_narrow_dependencies(self):
        for name in ("agent_calculations/service.rs", "agent_calculations/api.rs", "agent_calculations/tests.rs"):
            for dependency in ("AppConfig", "AppState", "api_tests"):
                with self.subTest(name=name, dependency=dependency):
                    errors = self.check_sources(
                        {f"services/ai/{name}": f"use crate::{dependency};"},
                        checks.check_agent_calculation_dependencies,
                    )
                    self.assertTrue(any("narrow Db/data_root" in error for error in errors))
        self.assertEqual([], self.check_sources(
            {"services/ai/agent_calculations/service.rs": "use crate::db::Db; fn f(data_root: &Path) {}"},
            checks.check_agent_calculation_dependencies,
        ))

    def check_sources(self, sources, check=None):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative, source in sources.items():
                path = root / "src" / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source, encoding="utf-8")
            with patch.object(checks, "REPO_ROOT", root), patch.object(
                checks, "SRC_ROOT", root / "src"
            ), patch.object(checks, "ALL_SRC_ROOTS", (root / "src",)):
                errors = []
                (check or checks.check_upload_boundaries)(errors)
                return errors

    def check_upload_sources(self, sources):
        """sources: {入口包里 uploads/ 下的相对路径: 源码}。"""
        with tempfile.TemporaryDirectory() as directory:
            crate_root = Path(directory) / "retain-pdf-ingest"
            for relative, source in sources.items():
                path = crate_root / "src" / "uploads" / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source, encoding="utf-8")
            with patch.object(checks, "extracted_domain_root", lambda domain: crate_root):
                errors = []
                checks.check_upload_boundaries(errors)
                return errors

    def test_narrow_dependencies_are_allowed(self):
        self.assertEqual([], self.check_upload_sources({
            "service.rs": "use retain_pdf_data::db::Db; use super::capacity::UploadCapacity;",
            "tests/mod.rs": "let root = std::env::temp_dir();",
        }))

    def test_environment_reads_are_rejected(self):
        for source in (
            "let config = UploadProcessingConfig::from_env();",
            'let value = std::env::var("CONFIG");',
            "let values = env::vars_os();",
        ):
            with self.subTest(source=source):
                errors = self.check_upload_sources({"service.rs": source})
                self.assertTrue(any("startup configuration" in error for error in errors))

    def test_http_error_mapping_stays_outside_uploads(self):
        errors = self.check_upload_sources({
            "error.rs": "use retain_pdf_api_base::error::AppError;",
        })
        self.assertTrue(any("HTTP error mapping" in error for error in errors))
        self.assertEqual([], self.check_upload_sources({
            "api.rs": "use retain_pdf_api_base::error::AppError;",
        }))

    def test_comments_do_not_trigger_the_upload_rules(self):
        self.assertEqual([], self.check_upload_sources({
            "service.rs": "// AppError is mapped in lib.rs\n/* env::var() */",
        }))

    def test_upload_route_deps_only_import_the_application_facade(self):
        self.assertEqual([], self.check_sources(
            {"routes/common/uploads.rs": "use crate::services::ingest::UploadApiDeps;"},
            checks.check_route_service_imports,
        ))
        for source in (
            "use crate::services::ingest::UploadService;",
            "use crate::services::ingest::uploads::api::UploadApiDeps;",
            "use crate::services::upload_api::UploadApiDeps;",
        ):
            with self.subTest(source=source):
                errors = self.check_sources(
                    {"routes/common/uploads.rs": source},
                    checks.check_route_service_imports,
                )
                self.assertTrue(any("must not import internal services" in error for error in errors))

    def test_library_route_dependencies_do_not_import_job_capabilities(self):
        self.assertEqual([], self.check_sources(
            {"routes/common/library.rs": "use crate::services::library::LibraryDeps;"},
            checks.check_route_service_imports,
        ))
        for source in (
            "use crate::services::jobs::JobsFacade;",
            "use crate::services::{jobs::JobsFacade, library::LibraryDeps};",
        ):
            with self.subTest(source=source):
                self.assertTrue(self.check_sources(
                    {"routes/common/library.rs": source}, checks.check_route_service_imports,
                ))

    def test_ai_private_gateway_rejects_external_and_grouped_access(self):
        for source in (
            "use crate::services::ai::gateway::AiGateway;",
            "use crate::services::ai::{gateway::AiGateway};",
            "crate::services::ai::gateway::run();",
        ):
            with self.subTest(source=source):
                self.assertTrue(self.check_sources(
                    {"api_tests/ai_proxy.rs": source}, checks.check_business_private_modules,
                ))
        self.assertEqual([], self.check_sources({
            "app/state.rs": "use crate::services::ai::AiGateway;",
            "services/ai/api.rs": "use super::gateway::AiGateway;",
        }, checks.check_business_private_modules))

    def test_ai_route_only_imports_its_domain_facade(self):
        for source in ("use crate::services::ai;", "use crate::services::ai::{ask, get_runtime_config};"):
            with self.subTest(source=source):
                self.assertEqual([], self.check_sources(
                    {"routes/ai_proxy.rs": source}, checks.check_route_service_imports,
                ))
        for source in (
            "use crate::services::jobs::JobsFacade;",
            "use crate::services::ai_proxy_api;",
        ):
            self.assertTrue(self.check_sources(
                {"routes/ai_proxy.rs": source}, checks.check_route_service_imports,
            ))

    def test_ai_models_remain_in_facade_guard(self):
        self.assertTrue(self.check_sources(
            {"services/ai/gateway.rs": "use crate::models::UploadRecord;"},
            checks.check_service_model_facade_boundaries,
        ))
        self.assertEqual([], self.check_sources(
            {"services/ai/gateway.rs": "use crate::models::domain::UploadRecord;"},
            checks.check_service_model_facade_boundaries,
        ))

    def test_small_module_routes_import_only_their_domain_facade(self):
        for domain, route in (
            ("platform", "fonts"), ("providers", "credentials"),
            ("ai", "agent_calculations"), ("glossaries", "glossaries"),
        ):
            with self.subTest(domain=domain):
                self.assertEqual([], self.check_sources(
                    {f"routes/{route}.rs": f"use crate::services::{domain}::Entry;"},
                    checks.check_route_service_imports,
                ))
                self.assertTrue(self.check_sources(
                    {f"routes/{route}.rs": "use crate::services::jobs::Entry;"},
                    checks.check_route_service_imports,
                ))

    def test_small_module_private_paths_are_guarded(self):
        # 字体、凭据库拆进了平台包、服务商包，子模块由编译器保证私有，不再归这条规则管。
        for domain, member in (("agent_calculations", "service"),):
            with self.subTest(domain=domain, member=member):
                for suffix in (f"{member}::Entry", f"{{{member}::Entry}}"):
                    self.assertTrue(self.check_sources(
                        {"api_tests/consumer.rs": f"use crate::services::{domain}::{suffix};"},
                        checks.check_business_private_modules,
                    ))
                self.assertEqual([], self.check_sources(
                    {"services/jobs/consumer.rs": f"use crate::services::{domain}::Entry;"},
                    checks.check_business_private_modules,
                ))

    def test_small_module_model_facades_are_scanned(self):
        for domain in ("ai/agent_calculations",):
            with self.subTest(domain=domain):
                self.assertTrue(self.check_sources(
                    {f"services/{domain}/api.rs": "use crate::models::UploadRecord;"},
                    checks.check_service_model_facade_boundaries,
                ))
                self.assertEqual([], self.check_sources(
                    {f"services/{domain}/api.rs": "use crate::models::domain::UploadRecord;"},
                    checks.check_service_model_facade_boundaries,
                ))


class FailureCatalogueCoverageTests(unittest.TestCase):
    """恢复目录必须登记 Python 侧发出的每一个 failure code。

    漏登记不会报错、不会崩，只会让用户看到「未识别的失败类型，重试会从头开始」——
    而这句话通常是假的：恢复按钮并不看分类，照样只重跑渲染。这条门禁存在的意义
    就是把一个静默的谎言变成一次构建失败。
    """

    CATALOGUE = (
        "const CATALOGUE: &[(&str, FailureRecovery)] = &[\n"
        '    ("render_failed", FailureRecovery {\n'
        "        resume_from: Some(ResumeFrom::Render),\n"
        '        hint: "...",\n'
        "    }),\n"
        '    ("auth_failed", FailureRecovery {\n'
        "        resume_from: None,\n"
        '        hint: "...",\n'
        "    }),\n"
        "];\n"
    )

    PYTHON = (
        "def classify(exc):\n"
        "    if a:\n"
        '        error_type = "render_failed"\n'
        "    elif b:\n"
        '        error_type = "auth_failed"\n'
    )

    def run_check(self, catalogue, python):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            catalogue_path = root / "job_failure_catalogue.rs"
            python_path = root / "structured_errors.py"
            catalogue_path.write_text(catalogue, encoding="utf-8")
            python_path.write_text(python, encoding="utf-8")
            with patch.object(checks, "FAILURE_CATALOGUE_PATH", catalogue_path), patch.object(
                checks, "PYTHON_STRUCTURED_ERRORS", python_path
            ):
                errors = []
                checks.check_failure_catalogue_covers_python_codes(errors)
                return errors

    def test_full_coverage_passes(self):
        self.assertEqual([], self.run_check(self.CATALOGUE, self.PYTHON))

    def test_unregistered_python_code_is_reported(self):
        python = self.PYTHON + "    elif c:\n" + '        error_type = "typst_runtime_failed"\n'
        errors = self.run_check(self.CATALOGUE, python)
        self.assertEqual(1, len(errors))
        self.assertIn("typst_runtime_failed", errors[0])

    def test_every_missing_code_is_reported_not_just_the_first(self):
        python = (
            self.PYTHON
            + "    elif c:\n"
            + '        error_type = "json_decode_failed"\n'
            + "    elif d:\n"
            + '        error_type = "upstream_bad_request"\n'
        )
        self.assertEqual(2, len(self.run_check(self.CATALOGUE, python)))

    def test_catalogue_only_codes_are_not_reported(self):
        """Rust 侧也会产生分类（结构化 JSON 缺失时兜底），目录里多出来是正常的。"""
        extra = (
            '    ("process_timeout", FailureRecovery {\n'
            "        resume_from: None,\n"
            '        hint: "...",\n'
            "    }),\n];"
        )
        self.assertEqual([], self.run_check(self.CATALOGUE.replace("];", extra), self.PYTHON))

    def test_gate_reports_when_it_can_no_longer_parse_the_catalogue(self):
        """写法变了而门禁没跟上时必须叫出来，而不是静默变成空操作。"""
        errors = self.run_check("const CATALOGUE: &[Foo] = &[];\n", self.PYTHON)
        self.assertEqual(1, len(errors))
        self.assertIn("一条目录项都没解析出来", errors[0])

    def test_gate_reports_when_it_can_no_longer_parse_the_python_source(self):
        errors = self.run_check(self.CATALOGUE, "FAILURE_CODES = {'a': 1}\n")
        self.assertEqual(1, len(errors))
        self.assertIn("一个 error_type", errors[0])

    def test_missing_catalogue_is_reported(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            python_path = root / "structured_errors.py"
            python_path.write_text(self.PYTHON, encoding="utf-8")
            with patch.object(checks, "FAILURE_CATALOGUE_PATH", root / "gone.rs"), patch.object(
                checks, "PYTHON_STRUCTURED_ERRORS", python_path
            ):
                errors = []
                checks.check_failure_catalogue_covers_python_codes(errors)
        self.assertEqual(1, len(errors))
        self.assertIn("恢复目录不见了", errors[0])


class ServiceDependencyCycleTests(unittest.TestCase):
    check_sources = UploadBoundaryTests.check_sources

    def run_check(self, sources, known=frozenset()):
        with patch.object(checks, "KNOWN_SERVICE_CYCLE_EDGES", set(known)):
            return self.check_sources(sources, checks.check_service_dependency_cycles)

    def test_one_way_dependencies_pass(self):
        self.assertEqual([], self.run_check({
            "services/jobs/query.rs": "use crate::services::files::artifacts::build;",
            "services/library/mod.rs": "use crate::services::{jobs::JobQueries, artifacts};",
        }))

    def test_cycle_is_found_through_grouped_relative_and_inline_paths(self):
        for back_edge in (
            "use crate::services::{library::{api::Books}};",
            "use super::super::super::library::api::Books;",
            "fn f() { crate::services::library::api::load(); }",
        ):
            with self.subTest(back_edge=back_edge):
                errors = self.run_check({
                    "services/library/mod.rs": "use crate::services::jobs::JobQueries;",
                    "services/jobs/query/read.rs": back_edge,
                })
                self.assertEqual(2, len(errors), errors)
                self.assertTrue(all("closes a dependency cycle" in error for error in errors))

    def test_tests_comments_and_strings_do_not_count(self):
        self.assertEqual([], self.run_check({
            "services/library/mod.rs": "use crate::services::jobs::JobQueries;",
            "services/jobs/tests.rs": "use crate::services::library::api::Books;",
            "services/jobs/query.rs": '''
                // use crate::services::library::api::Books;
                const NOTE: &str = "crate::services::library::x";
                fn real() {}
#[cfg(test)]
mod tests { use crate::services::library::api::Books; }
            ''',
        }))

    def test_known_cycles_are_frozen_and_must_shrink(self):
        sources = {
            "services/library/mod.rs": "use crate::services::jobs::JobQueries;",
            "services/jobs/query.rs": "use crate::services::library::api::Books;",
        }
        known = {("library", "jobs"), ("jobs", "library")}
        self.assertEqual([], self.run_check(sources, known))
        errors = self.run_check({"services/library/mod.rs": sources["services/library/mod.rs"]}, known)
        self.assertEqual(2, len(errors), errors)
        self.assertTrue(all("no longer part of a cycle" in error for error in errors))



class ArtifactPathOwnershipTests(unittest.TestCase):
    check_sources = UploadBoundaryTests.check_sources

    def run_check(self, sources, known=frozenset()):
        with patch.object(checks, "KNOWN_ARTIFACT_PATH_READERS", {Path(p) for p in known}):
            return self.check_sources(sources, checks.check_artifact_path_ownership)

    def test_exit_modules_may_build_paths(self):
        self.assertEqual([], self.run_check({
            "services/jobs/files/downloads/files.rs": "fn f() { resolve_output_pdf(job, root); }",
            "services/jobs/reader/markdown.rs": "fn f() { resolve_markdown_path(job, root); }",
            "services/files/derived_artifacts/word.rs": "use crate::storage_paths::resolve_source_pdf;",
            "services/files/artifacts/bundle.rs": "fn f() { resolve_markdown_path(job, root); }",
        }))

    def test_new_readers_are_rejected(self):
        for path in ("services/library/new.rs", "routes/jobs/files.rs"):
            with self.subTest(path=path):
                errors = self.run_check({path: "fn f() { resolve_output_pdf(job, root); }"})
                self.assertEqual(1, len(errors), errors)
                self.assertIn("builds artifact paths itself", errors[0])

    def test_tests_comments_and_strings_do_not_count(self):
        self.assertEqual([], self.run_check({
            "services/library/tests.rs": "fn f() { resolve_output_pdf(job, root); }",
            "services/library/x.rs": '''
                // resolve_output_pdf(job, root)
                const NOTE: &str = "resolve_source_pdf";
#[cfg(test)]
mod tests { fn f() { resolve_output_pdf(job, root); } }
            ''',
        }))

    def test_known_readers_are_frozen_and_must_shrink(self):
        source = {"services/library/metadata_suggestions.rs": "fn f() { resolve_source_pdf(job, root); }"}
        known = {"src/services/library/metadata_suggestions.rs"}
        self.assertEqual([], self.run_check(source, known))
        errors = self.run_check({}, known)
        self.assertEqual(1, len(errors), errors)
        self.assertIn("remove it from KNOWN_ARTIFACT_PATH_READERS", errors[0])


class ServiceCycleTests(unittest.TestCase):
    check_sources = UploadBoundaryTests.check_sources

    def test_no_cycle_is_grandfathered_any_more(self):
        self.assertEqual(set(), checks.KNOWN_SERVICE_CYCLE_EDGES)
        errors = self.check_sources({
            "services/files/download_generation.rs": "use crate::services::jobs::FileDownload;",
            "services/jobs/files/downloads.rs": "use crate::services::files::download_generation::DownloadGeneration;",
        }, checks.check_service_dependency_cycles)
        self.assertEqual(2, len(errors), errors)
        self.assertTrue(all("closes a dependency cycle" in error for error in errors), errors)


class DomainFacadeTests(unittest.TestCase):
    _check_sources = UploadBoundaryTests.check_sources

    def check_sources(self, sources, check=None):
        # 这一组只管 services/ 下的领域目录；拆成独立包的领域另有测试。
        with patch.object(checks, "EXTRACTED_DOMAINS", {}):
            return self._check_sources(sources, check)

    def domain_tree(self, **extra):
        sources = {f"services/{domain}/mod.rs": "mod inner;\npub(crate) use inner::Thing;\n"
                   for domain in checks.SERVICE_DOMAINS}
        sources["services/mod.rs"] = ""
        sources.update(extra)
        return sources

    def test_current_layout_passes(self):
        self.assertEqual([], self.check_sources(self.domain_tree(), checks.check_service_domains))
        self.assertEqual([], self.check_sources(self.domain_tree(), checks.check_domain_facades))

    def test_new_top_level_service_module_is_rejected(self):
        for path in ("services/billing.rs", "services/billing/mod.rs"):
            with self.subTest(path=path):
                errors = self.check_sources(self.domain_tree(**{path: ""}), checks.check_service_domains)
                self.assertEqual(1, len(errors), errors)
                self.assertIn("services only holds the domains", errors[0])

    def test_public_submodule_breaks_the_facade(self):
        for declaration in ("pub mod inner;", "pub(crate) mod inner;", "pub(super) mod inner;"):
            with self.subTest(declaration=declaration):
                errors = self.check_sources(
                    self.domain_tree(**{"services/files/mod.rs": declaration}), checks.check_domain_facades,
                )
                self.assertEqual(1, len(errors), errors)
                self.assertIn("must stay private", errors[0])
        # 测试模块里的声明不算
        self.assertEqual([], self.check_sources(
            self.domain_tree(**{"services/files/mod.rs": "mod inner;\n#[cfg(test)]\nmod tests {\npub mod x;\n}\n"}),
            checks.check_domain_facades,
        ))

    def test_domain_dependencies_only_point_down(self):
        self.assertEqual([], [
            error for error in self.check_sources({
                "services/library/media.rs": "use crate::services::files::DownloadGeneration;",
            }, checks.check_domain_dependency_direction)
            if "not an allowed domain dependency" in error
        ])
        errors = self.check_sources({
            "services/files/download_generation.rs": "use crate::services::library::Disposition;",
        }, checks.check_domain_dependency_direction)
        self.assertTrue(any(
            "services::files -> services::library is not an allowed domain dependency" in error
            for error in errors
        ), errors)

    def test_unused_allowed_dependency_must_be_removed(self):
        errors = self.check_sources({"services/jobs/x.rs": ""}, checks.check_domain_dependency_direction)
        self.assertEqual(len(checks.ALLOWED_DOMAIN_DEPENDENCIES), len(errors), errors)
        self.assertTrue(all("nothing uses any more" in error for error in errors), errors)


class ExtractedDomainTests(unittest.TestCase):
    """拆成独立包的领域（例如术语表）：门面是那个包的 lib.rs，services 里只剩一行接上它。"""

    def run_check(self, services_mod, lib, extra=None):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sources = {f"src/services/{domain}/mod.rs": "mod inner;\n"
                       for domain in checks.SERVICE_DOMAINS if domain != "glossaries"}
            sources["src/services/mod.rs"] = services_mod
            sources["packages/retain-pdf-glossaries/src/lib.rs"] = lib
            sources.update(extra or {})
            for relative, source in sources.items():
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source, encoding="utf-8")
            with patch.object(checks, "REPO_ROOT", root), patch.object(checks, "SRC_ROOT", root / "src"), \
                    patch.object(checks, "ALL_SRC_ROOTS", (root / "src",)), \
                    patch.object(checks, "EXTRACTED_DOMAINS", {"glossaries": "retain-pdf-glossaries"}), \
                    patch.object(checks, "extracted_domain_root",
                                 lambda domain: root / "packages" / "retain-pdf-glossaries"):
                errors = []
                checks.check_domain_facades(errors)
                return errors

    def test_crate_facade_and_reexport_pass(self):
        self.assertEqual([], self.run_check(
            "pub(crate) use retain_pdf_glossaries as glossaries;\n", "mod api;\npub use api::Thing;\n",
        ))

    def test_missing_reexport_public_submodule_and_leftover_directory_are_rejected(self):
        errors = self.run_check("", "pub mod api;\n", {"src/services/glossaries/extra.rs": ""})
        self.assertEqual(3, len(errors), errors)
        self.assertTrue(any("re-export the retain-pdf-glossaries crate" in error for error in errors), errors)
        self.assertTrue(any("must stay private" in error for error in errors), errors)
        self.assertTrue(any("do not add code back under services/" in error for error in errors), errors)

    def crate_edges(self, crates):
        """crates: {领域: {包内相对路径: 源码}}，每个领域都当作已拆成独立包。"""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for domain, files in crates.items():
                for relative, source in files.items():
                    path = root / f"retain-pdf-{domain}" / "src" / relative
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text(source, encoding="utf-8")
            extracted = {domain: f"retain-pdf-{domain}" for domain in crates}
            with patch.object(checks, "EXTRACTED_DOMAINS", extracted), \
                    patch.object(checks, "extracted_domain_root", lambda domain: root / f"retain-pdf-{domain}"):
                return checks.extracted_domain_dependency_edges()

    def test_crate_name_uses_count_as_domain_edges(self):
        edges = self.crate_edges({
            "accounts": {"platform_settings/mod.rs": "use retain_pdf_providers::{create_credential};\n"},
            "providers": {"lib.rs": "use retain_pdf_core::models;\nuse retain_pdf_api_base::error::AppError;\n"},
        })
        self.assertEqual({("accounts", "providers")}, set(edges))

    def test_crate_name_uses_in_tests_and_comments_are_ignored(self):
        edges = self.crate_edges({
            "accounts": {"lib.rs": "// 以前用过 retain_pdf_providers\n",
                         "platform_settings/tests.rs": "use retain_pdf_providers::create_credential;\n"},
            "providers": {"lib.rs": "mod probe;\n#[cfg(test)]\nmod tests {\n    use retain_pdf_accounts::Principal;\n}\n"},
        })
        self.assertEqual({}, edges)


class RequestPayloadAccessTests(unittest.TestCase):
    check_sources = UploadBoundaryTests.check_sources

    def writers(self):
        return {str(path.relative_to("src")): "job.request_payload.translation.api_key.clear();"
                for path in checks.KNOWN_REQUEST_PAYLOAD_WRITERS}

    def test_readers_ask_the_job(self):
        sources = self.writers()
        sources["services/library/reading.rs"] = (
            "let id = job.source_job_id();\nlet secrets = sensitive_values(&job.request_payload);"
        )
        self.assertEqual([], self.check_sources(sources, checks.check_request_payload_access))

    def test_field_access_outside_writers_is_rejected(self):
        for source in (
            "let id = job.request_payload.source.artifact_job_id.trim();",
            "if job\n    .request_payload\n    .translation\n    .execution_connection\n    .is_some() {}",
        ):
            with self.subTest(source=source):
                sources = self.writers()
                sources["services/library/reading.rs"] = source
                errors = self.check_sources(sources, checks.check_request_payload_access)
                self.assertEqual(1, len(errors), errors)
                self.assertIn("ask the job instead", errors[0])

    def test_tests_may_build_requests(self):
        sources = self.writers()
        sources["services/library/reading.rs"] = (
            "fn f() {}\n#[cfg(test)]\nmod tests {\n    fn g() { job.request_payload.workflow = x; }\n}\n"
        )
        sources["api_tests/jobs.rs"] = "job.request_payload.workflow = x;"
        self.assertEqual([], self.check_sources(sources, checks.check_request_payload_access))

    def test_writer_list_only_shrinks(self):
        errors = self.check_sources({}, checks.check_request_payload_access)
        self.assertEqual(len(checks.KNOWN_REQUEST_PAYLOAD_WRITERS), len(errors), errors)
        self.assertTrue(all("remove it from KNOWN_REQUEST_PAYLOAD_WRITERS" in error for error in errors))


class JobEntryOwnershipTests(unittest.TestCase):
    check_sources = UploadBoundaryTests.check_sources

    def run_check(self, sources):
        return self.check_sources(sources, checks.check_job_entry_ownership)

    def test_the_entry_module_may_persist_and_launch(self):
        self.assertEqual([], self.run_check({
            "services/jobs/lifecycle/creation/submit.rs": """
                fn f() {
                    persist_job_with_resources(db, a, b, &job)?;
                    deps.db.create_ocr_recovery_job_state(d, &job, r, None)?;
                    deps.runtime.launch(job.job_id.clone());
                }
            """,
        }))

    def test_other_modules_are_rejected(self):
        for path, body in (
            ("services/jobs/actions/rerun.rs", "fn f() { persist_job_with_resources(db, a, b, &job); }"),
            ("services/jobs/actions/x.rs", "fn f() { self.db.create_ocr_recovery_job_state(d, &j, r, None); }"),
            ("services/library/translate.rs", "fn f() { deps.launcher.runtime.launch(id); }"),
            ("routes/jobs/lifecycle.rs", "fn f() { deps.runtime . launch(id); }"),
        ):
            with self.subTest(path=path):
                errors = self.run_check({path: body})
                self.assertEqual(1, len(errors), errors)
                self.assertIn("submit_job", errors[0])

    def test_cas_writes_tests_and_comments_do_not_count(self):
        self.assertEqual([], self.run_check({
            "services/jobs/actions/cancel.rs": "fn f() { cas_persist_job_with_resources(db, a, b, &job, &[]); }",
            "services/jobs/lifecycle/creation/tests.rs": "fn f() { persist_job_with_resources(db, a, b, &job); }",
            "services/jobs/x.rs": """
                // persist_job_with_resources(db, a, b, &job)
                const NOTE: &str = "runtime.launch(id)";
            """,
        }))

if __name__ == "__main__":
    unittest.main()
