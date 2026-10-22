//! Block full-text search.

use retainpdf_core::config::limits::MAX_SEARCH_LIMIT;
use retainpdf_api_base::error::AppError;
use retainpdf_api_base::paging::checked_limit;
use crate::views::{SearchQuery, SearchResultView};

use super::LibraryDeps;

pub fn search_blocks(
    deps: &LibraryDeps<'_>,
    query: &SearchQuery,
    owner: Option<&str>,
) -> Result<SearchResultView, AppError> {
    let limit = checked_limit(query.limit, MAX_SEARCH_LIMIT)?;
    let document_id = query.document_id.trim();
    // 多取一条：有第 limit+1 条就说明命中不止这些。
    let mut hits = deps.db.search_blocks_for_owner(
        &query.q,
        limit + 1,
        if document_id.is_empty() {
            None
        } else {
            Some(document_id)
        },
        owner,
    )?;
    let has_more = hits.len() > limit as usize;
    hits.truncate(limit as usize);
    Ok(SearchResultView {
        query: query.q.clone(),
        hits,
        limit,
        has_more,
    })
}
