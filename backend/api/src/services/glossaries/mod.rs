//! Glossary HTTP entry points and task glossary resolution.

mod api;
mod csv;
mod entries;
mod records;

#[cfg(test)]
use csv::parse_glossary_csv_text;
#[cfg(test)]
use entries::{merge_glossary_entries, normalize_glossary_entries, MAX_GLOSSARY_ENTRIES};
#[cfg(test)]
use records::{
    create_glossary, delete_glossary, filter_glossaries, list_glossaries, load_glossary_or_404,
    update_glossary,
};

#[cfg(test)]
mod tests;

// 门面：领域外只能用这里导出的东西，子模块都是私有的（见 BOUNDARIES.md「Domain Facades」）。
pub(crate) use api::{
    create_glossary_view, delete_glossary_view, export_glossary_csv_view, get_glossary_view,
    import_glossary_view, list_glossaries_view, parse_glossary_csv_view, update_glossary_view,
    GlossaryApiDeps,
};
pub(crate) use entries::resolve_task_glossary_request;
