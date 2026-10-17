//! 接口清单快照：`docs/routes.txt` 列出全部路由（方法、路径、多用户下网站账号的访问策略）。
//!
//! 接口多了、杂了，很大一部分是因为加接口没有门槛——加一条就是加一条，没人注意到总数在涨。
//! 这份快照让每次增删、改策略都出现在提交的 diff 里，要有意识地确认。改了路由之后这里会红，
//! 确认是有意的，就用 `UPDATE_ROUTES=1 cargo test -p retainpdf-api --lib route_manifest` 重新生成。
//!
//! 清单和接口目录（`docs/api/00-接口目录.md`）都从接口登记表（`app/endpoints.rs`）汇总，不启动服务。

use std::collections::BTreeMap;
use std::path::Path;

use crate::app::endpoints::Access;
use crate::services::accounts::{route_policy, RoutePolicy};

fn policy_label(method: &str, path: &str) -> &'static str {
    match route_policy(&method.to_ascii_uppercase(), path) {
        RoutePolicy::Signed => "signed",
        RoutePolicy::AdminOnly => "admin",
        RoutePolicy::Closed => "closed",
    }
}

/// 接口清单：从登记表汇总，一行一个接口。
fn manifest() -> String {
    let mut table: BTreeMap<String, Vec<(String, &'static str)>> = BTreeMap::new();
    for endpoint in crate::app::registered_endpoints() {
        // 登记表里写的谁能用，要和账号包现在的判定一致（第 8 步第二个分支把判定改成直接查登记表）。
        if endpoint.access != Access::Public {
            assert_eq!(
                endpoint.access.label(),
                policy_label(endpoint.method, endpoint.path),
                "{} {}: 登记表写的谁能用和 accounts::route_policy 不一致",
                endpoint.method,
                endpoint.path
            );
        }
        table
            .entry(endpoint.path.to_string())
            .or_default()
            .push((endpoint.method.to_string(), endpoint.access.label()));
    }
    let mut lines = vec![
        "# 全部 HTTP 路由：方法 路径 多用户下网站账号的访问策略（signed 登录可用 / admin 只给管理员 /".to_string(),
        "# closed 关闭 / public 不需要登录）。由 src/api_tests/route_manifest.rs 从接口登记表生成，不要手改：".to_string(),
        "#   UPDATE_ROUTES=1 cargo test -p retainpdf-api --lib route_manifest".to_string(),
    ];
    let count: usize = table.values().map(Vec::len).sum();
    lines.push(format!("# {} 条路径，{count} 个接口", table.len()));
    for (path, methods) in &mut table {
        methods.sort();
        for (method, access) in methods.iter() {
            lines.push(format!("{method:<7}{path}  {access}"));
        }
    }
    lines.join("\n") + "\n"
}

/// 路由只在登记表里注册：路由文件里不许再直接写 `.route(`，否则这条接口不进接口目录、也没有说明。
#[test]
fn routes_are_registered_only_through_endpoint_tables() {
    for file in crate::test_support::routes::router_source_files() {
        let source = std::fs::read_to_string(&file).unwrap();
        let code = source.split("\n#[cfg(test)]").next().unwrap_or_default();
        assert!(
            !code.contains(".route("),
            "{}: 直接写了 .route(…)；改成在这个文件的 crate::endpoints! 登记表里加一条",
            file.display()
        );
    }
}

#[test]
fn route_manifest_matches_snapshot() {
    let snapshot = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/routes.txt");
    let current = manifest();
    if std::env::var_os("UPDATE_ROUTES").is_some() {
        std::fs::write(&snapshot, &current).unwrap();
        return;
    }
    let saved = std::fs::read_to_string(&snapshot).unwrap_or_default();
    if saved != current {
        let saved_lines: std::collections::BTreeSet<&str> = saved.lines().collect();
        let current_lines: std::collections::BTreeSet<&str> = current.lines().collect();
        let added: Vec<_> = current_lines.difference(&saved_lines).collect();
        let removed: Vec<_> = saved_lines.difference(&current_lines).collect();
        panic!(
            "routes changed (docs/routes.txt is out of date).\n+ {added:#?}\n- {removed:#?}\n\
             If intended: UPDATE_ROUTES=1 cargo test -p retainpdf-api --lib route_manifest"
        );
    }
}

/// 路由文件（模块路径的最后几段）→ 接口目录里的分组标题。没列的照模块名显示。
const SECTIONS: &[(&str, &str)] = &[
    ("router::public", "登录和健康检查（不需要登录）"),
    ("router::accounts", "账号、管理后台"),
    ("router::credentials", "凭据"),
    ("router::providers", "模型和 OCR 服务商"),
    ("router::ingestion", "上传和建任务的旧入口"),
    ("router::jobs::lifecycle", "任务"),
    ("router::jobs::actions", "任务 · 操作"),
    ("router::jobs::progress", "任务 · 进度"),
    ("router::jobs::files", "任务 · 文件"),
    ("router::jobs::reports", "任务 · 报告"),
    ("router::jobs::reader", "任务 · 阅读"),
    ("router::jobs::translations", "任务 · 译文"),
    ("router::jobs::data", "任务 · 数据"),
    ("router::jobs::ai", "任务 · AI"),
    ("router::documents", "书（文档）"),
    ("router::library", "书库"),
    ("router::collections", "合集"),
    ("router::glossaries", "术语表"),
    ("router::fonts", "字体"),
    ("router::sync", "多设备同步"),
    ("router::backups", "备份"),
    ("router::ai", "AI 助手"),
    ("router::internal_agent", "内部：Agent"),
    ("router::model_requests", "内部：模型调用"),
    ("router::terminal", "助手终端（只在单机版）"),
    ("router::simple", "简化接口（另一个端口）"),
];

fn section_of(module: &str) -> (usize, &'static str) {
    SECTIONS
        .iter()
        .enumerate()
        .filter(|(_, (suffix, _))| module.ends_with(suffix) || module.contains(&format!("{suffix}::")))
        .max_by_key(|(_, (suffix, _))| suffix.len())
        .map(|(index, (_, title))| (index, *title))
        .unwrap_or((SECTIONS.len(), "其他"))
}

/// 接口目录 `docs/api/00-接口目录.md`：登记表里的全部接口，按路由文件分组，旧地址列在新地址后面。
fn directory() -> String {
    let endpoints = crate::app::registered_endpoints();
    let mut sections: BTreeMap<(usize, &str), Vec<crate::app::endpoints::Endpoint>> = BTreeMap::new();
    for endpoint in &endpoints {
        sections.entry(section_of(endpoint.module)).or_default().push(*endpoint);
    }
    let current = endpoints.iter().filter(|e| e.replaced_by.is_none()).count();
    let mut out = vec![
        "# 接口目录".to_string(),
        String::new(),
        "<!-- 由 backend/api/src/api_tests/route_manifest.rs 从接口登记表（backend/api/src/app/endpoints.rs）生成，不要手改：".to_string(),
        "     UPDATE_ROUTES=1 cargo test -p retainpdf-api --lib route_manifest -->".to_string(),
        String::new(),
        format!(
            "共 {} 个接口，其中 {current} 个现用地址、{} 个旧地址（还能用，新代码不要再调）。地址前缀省略了 `/api/v1`。",
            endpoints.len(),
            endpoints.len() - current
        ),
        String::new(),
        "「谁能用」只在多用户模式下起作用：`signed` 登录就能用（路径里的编号要是自己的）、`admin` 只给管理员、".to_string(),
        "`closed` 对网站账号关闭、`public` 不需要登录。单机版里所有接口都能用。".to_string(),
    ];
    let short = |path: &str| path.strip_prefix("/api/v1").filter(|rest| !rest.is_empty()).unwrap_or(path).to_string();
    for ((_, title), items) in &sections {
        out.push(String::new());
        out.push(format!("## {title}"));
        out.push(String::new());
        out.push("| 方法 | 地址 | 谁能用 | 说明 | 旧地址 | 处理函数 |".to_string());
        out.push("|---|---|---|---|---|---|".to_string());
        for endpoint in items.iter().filter(|e| e.replaced_by.is_none()) {
            // 旧地址可能登记在别的路由文件里（比如任务下的封面现在在书下面），所以在全部接口里找。
            let olds: Vec<String> = endpoints
                .iter()
                .filter(|old| old.replaced_by == Some(endpoint.path) && old.method == endpoint.method)
                .map(|old| match old.summary.trim() {
                    "" => format!("`{}`", short(old.path)),
                    note => format!("`{}`（{}）", short(old.path), note.replace('|', "\\|")),
                })
                .collect();
            out.push(format!(
                "| {} | `{}` | {} | {} | {} | `{}` |",
                endpoint.method,
                short(endpoint.path),
                endpoint.access.label(),
                endpoint.summary.trim().replace('|', "\\|"),
                olds.join("<br>"),
                endpoint.handler.replace(' ', ""),
            ));
        }
    }
    out.join("\n") + "\n"
}

/// 登记表自己的规矩：现用地址都要有说明；旧地址要指向一个存在的、方法相同的新地址。
#[test]
fn endpoint_registry_is_complete() {
    let endpoints = crate::app::registered_endpoints();
    for endpoint in &endpoints {
        match endpoint.replaced_by {
            None => assert!(
                !endpoint.summary.trim().is_empty(),
                "{} {}（{}）：登记表里缺一句话说明",
                endpoint.method,
                endpoint.path,
                endpoint.module
            ),
            Some(replacement) => assert!(
                endpoints
                    .iter()
                    .any(|e| e.path == replacement && e.method == endpoint.method && e.replaced_by.is_none()),
                "{} {}：旧地址指向的新地址 {replacement} 不在登记表里（或者方法不同、它自己也是旧地址）",
                endpoint.method,
                endpoint.path
            ),
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    for endpoint in &endpoints {
        assert!(seen.insert((endpoint.method, endpoint.path)), "{} {} 登记了两次", endpoint.method, endpoint.path);
    }
}

#[test]
fn endpoint_directory_matches_registry() {
    let doc = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/api/00-接口目录.md");
    let current = directory();
    if std::env::var_os("UPDATE_ROUTES").is_some() {
        std::fs::write(&doc, &current).unwrap();
        return;
    }
    let saved = std::fs::read_to_string(&doc).unwrap_or_default();
    assert!(
        saved == current,
        "docs/api/00-接口目录.md is out of date. Regenerate: UPDATE_ROUTES=1 cargo test -p retainpdf-api --lib route_manifest"
    );
}
