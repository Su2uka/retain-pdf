//! 模型的门面：核心包的模型，加上各领域包自己的返回类型。
//!
//! 返回类型按领域放在各自的包里（方案见 docs/ops/planning/api-reorg/20261018-返回类型按领域拆开.md），
//! 这里把它们并回 `models::api`，路由、测试照旧写 `crate::models::api::…`，不用知道类型在哪个包。

pub use retainpdf_core::models::*;

pub mod api {
    pub use retainpdf_core::models::api::*;
    pub use retainpdf_glossaries::{
        GlossaryCsvExportView, GlossaryCsvParseInput, GlossaryCsvParseView, GlossaryDetailView,
        GlossaryListView, GlossarySummaryView, GlossaryUpsertInput, ListGlossariesQuery,
    };
    pub use retainpdf_library::{
        AddCollectionDocumentsInput, AppendMessageInput, ApplyDocumentMetadataSuggestionInput,
        CollectionListView, CollectionMutationResult, ConversationDetailView, ConversationListView,
        ConversationMutationResult, CreateCollectionInput, CreateConversationInput,
        CreateDocumentMetadataSuggestionInput, DocumentDeleteResultView, DocumentListView,
        DocumentMetadataEvidenceView, DocumentMetadataSuggestionApplyView,
        DocumentMetadataSuggestionListView, DocumentMetadataSuggestionView, DocumentTitleCandidateView,
        ForkConversationInput, ForkMessageInput, LibraryBookDetailView, LibraryBookListItemView,
        LibraryBookListView, LibraryDeleteQuery, LibraryDeleteResultView, ListConversationsQuery,
        ListDocumentMetadataSuggestionsQuery, ListDocumentsQuery, PatchCollectionInput,
        PatchConversationInput, PatchDocumentInput, SearchQuery, SearchResultView,
    };
    pub use retainpdf_platform::{BackupItemView, BackupRestoreView, BackupStatusView};
}
