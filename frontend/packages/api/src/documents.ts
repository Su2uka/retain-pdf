// documents — pure
import { apiFetch } from "./internal/runtime.js";
import { buildApiHeaders, unwrapEnvelope } from "./internal/runtime.js";
import { buildApiEndpoint, pageQuotaErrorFromPayload } from "./http.js";

import type {
  ApplyDocumentMetadataSuggestionInput,
  CoverageJobView,
  CoverageSegment,
  CreateDocumentMetadataSuggestionInput,
  DocumentDeleteResultView,
  DocumentListView,
  DocumentMetadataEvidenceView,
  DocumentMetadataSuggestionApplyView,
  DocumentMetadataSuggestionListView,
  DocumentMetadataSuggestionView,
  DocumentReadingView,
  DocumentRecord,
  DocumentTitleCandidateView,
  TranslationCoverageView,
} from "@retainpdf/contracts/library-documents";

// 书库的线上形状以契约 library-documents.v1 为准；下面是沿用的旧名字。
export type {
  ApplyDocumentMetadataSuggestionInput,
  CreateDocumentMetadataSuggestionInput,
  DocumentDeleteResultView,
  DocumentListView,
  DocumentReadingView,
  DocumentRecord,
  TranslationCoverageView,
};
export type DocumentMetadataEvidence = DocumentMetadataEvidenceView;
export type DocumentTitleCandidate = DocumentTitleCandidateView;
export type DocumentMetadataSuggestion = DocumentMetadataSuggestionView;
export type TranslationCoverageSegment = CoverageSegment;
export type TranslationCoverageJob = CoverageJobView;

export type DocumentListQuery = {
  limit?: number;
  offset?: number;
  readingStatus?: string;
  collectionId?: string;
  /** 服务端标题/原始文件名包含匹配（后端 LIKE，已转义 %/_）。空串不发送。 */
  q?: string;
};

export type DocumentJobSummary = Record<string, any> & {
  job_id?: string;
  workflow?: string;
  status?: string;
  ocr_reused?: boolean;
  source_artifact_job_id?: string | null;
  stages?: DocumentJobStages;
};

export type DocumentJobStageState =
  | "reused"
  | "queued"
  | "pending"
  | "in_progress"
  | "completed"
  | "failed"
  | "skipped";

export type DocumentJobStages = {
  ocr?: { state?: DocumentJobStageState; [key: string]: unknown };
  translation?: { state?: DocumentJobStageState; [key: string]: unknown };
  render?: { state?: DocumentJobStageState; [key: string]: unknown };
  [key: string]: unknown;
};

export type DocumentJobSubmissionView = {
  job_id: string;
  workflow: string;
  status: string;
  ocr_reused: boolean;
  source_artifact_job_id?: string | null;
  stages?: DocumentJobStages;
  [key: string]: unknown;
};

export type DocumentJobsView = {
  items: DocumentJobSummary[];
  invocation_summary?: Record<string, unknown>;
  total?: number;
  limit?: number;
  offset?: number;
  has_more?: boolean;
};

export type DocumentJobsQuery = {
  limit?: number;
  offset?: number;
};

export interface DocumentRequestError extends Error {
  status?: number;
  errorCode?: string;
  reason?: string;
  canFallbackToOcr?: boolean;
}

function documentRequestError(
  fallback: string,
  status: number,
  payload: any,
): DocumentRequestError {
  // 通用错误把结构化数据放在 payload.error.details；老接口可能直接在 payload.details/data。
  const structured = payload?.error && typeof payload.error === "object" ? payload.error : null;
  const details = structured?.details && typeof structured.details === "object"
    ? structured.details
    : payload?.details && typeof payload.details === "object"
      ? payload.details
      : payload?.data && typeof payload.data === "object"
        ? payload.data
        : {};
  const message = `${payload?.message || details?.message || fallback}`;
  const error = new Error(`${message}(${status})`) as DocumentRequestError;
  error.status = status;
  const errorCode = `${payload?.error_code || structured?.code || details?.error_code || details?.code || (typeof payload?.code === "string" ? payload.code : "")}`.trim();
  if (errorCode) error.errorCode = errorCode;
  const reason = `${payload?.reason || details?.reason || ""}`.trim();
  if (reason) error.reason = reason;
  const canFallback = payload?.can_fallback_to_ocr ?? details?.can_fallback_to_ocr;
  if (typeof canFallback === "boolean") error.canFallbackToOcr = canFallback;
  return error;
}

export async function fetchDocumentList(
  apiPrefix: string,
  { limit = 50, offset = 0, readingStatus = "", collectionId = "", q = "" }: DocumentListQuery = {},
): Promise<DocumentListView> {
  const params = new URLSearchParams();
  params.set("limit", `${limit}`);
  params.set("offset", `${offset}`);
  if (`${readingStatus || ""}`.trim()) params.set("reading_status", `${readingStatus}`.trim());
  if (`${collectionId || ""}`.trim()) params.set("collection_id", `${collectionId}`.trim());
  if (`${q || ""}`.trim()) params.set("q", `${q}`.trim());
  const resp = await apiFetch(`${buildApiEndpoint(apiPrefix, "documents")}?${params.toString()}`, { headers: buildApiHeaders() });
  if (!resp.ok) throw new Error(`读取文档库失败，请稍后重试。(${resp.status})`);
  return unwrapEnvelope<DocumentListView>(await resp.json());
}

export async function fetchDocumentByJobId(apiPrefix: string, jobId: string): Promise<DocumentRecord | null> {
  const normalized = `${jobId || ""}`.trim();
  if (!normalized) return null;
  const params = new URLSearchParams();
  params.set("job_id", normalized);
  const resp = await apiFetch(`${buildApiEndpoint(apiPrefix, "documents")}?${params.toString()}`, { headers: buildApiHeaders() });
  if (!resp.ok) throw new Error(`按 job 查文档失败，请稍后重试。(${resp.status})`);
  const payload: any = unwrapEnvelope(await resp.json()) || { documents: [], total: 0, limit: 0, offset: 0 };
  const { documents = [] } = payload;
  return Array.isArray(documents) && documents.length ? documents[0] : null;
}

export async function fetchDocument(apiPrefix: string, documentId: string): Promise<DocumentRecord> {
  const normalized = `${documentId || ""}`.trim();
  if (!normalized) throw new Error("缺少 document_id。");
  const resp = await apiFetch(buildApiEndpoint(apiPrefix, `documents/${encodeURIComponent(normalized)}`), { headers: buildApiHeaders() });
  if (!resp.ok) throw new Error(`读取文档详情失败，请稍后重试。(${resp.status})`);
  return unwrapEnvelope<DocumentRecord>(await resp.json());
}

export async function patchDocument(apiPrefix: string, documentId: string, payload: Record<string, unknown> = {}): Promise<DocumentRecord> {
  const normalized = `${documentId || ""}`.trim();
  if (!normalized) throw new Error("缺少 document_id。");
  const resp = await apiFetch(buildApiEndpoint(apiPrefix, `documents/${encodeURIComponent(normalized)}`), {
    method: "PATCH",
    headers: { ...buildApiHeaders(), "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!resp.ok) {
    const envelope: any = await resp.json().catch(() => null);
    throw new Error(`${envelope?.message || "更新文档失败，请稍后重试。"}(${resp.status})`);
  }
  return unwrapEnvelope<DocumentRecord>(await resp.json());
}

/** GET /documents/:id/reading —— 阅读器该打开哪个任务。 */

export async function fetchDocumentReading(apiPrefix: string, documentId: string): Promise<DocumentReadingView> {
  const normalized = `${documentId || ""}`.trim();
  if (!normalized) throw new Error("缺少 document_id。");
  const resp = await apiFetch(buildApiEndpoint(apiPrefix, `documents/${encodeURIComponent(normalized)}/reading`), {
    headers: buildApiHeaders(),
  });
  if (!resp.ok) throw new Error(`读取阅读入口失败。(${resp.status})`);
  return unwrapEnvelope<DocumentReadingView>(await resp.json());
}

/** GET /documents/:id/translation-coverage —— 一本书翻了哪些页、每次任务提供了几页。 */

export async function fetchDocumentTranslationCoverage(
  apiPrefix: string,
  documentId: string,
): Promise<TranslationCoverageView> {
  const normalized = `${documentId || ""}`.trim();
  if (!normalized) throw new Error("缺少 document_id。");
  const resp = await apiFetch(buildApiEndpoint(apiPrefix, `documents/${encodeURIComponent(normalized)}/translation-coverage`), {
    headers: buildApiHeaders(),
  });
  if (!resp.ok) throw new Error(`读取翻译覆盖失败。(${resp.status})`);
  return unwrapEnvelope<TranslationCoverageView>(await resp.json());
}

export async function createDocumentMetadataSuggestion(
  apiPrefix: string,
  documentId: string,
  payload: CreateDocumentMetadataSuggestionInput = {},
): Promise<DocumentMetadataSuggestion> {
  const normalized = `${documentId || ""}`.trim();
  if (!normalized) throw new Error("缺少 document_id。");
  const resp = await apiFetch(
    buildApiEndpoint(apiPrefix, `documents/${encodeURIComponent(normalized)}/metadata-suggestions`),
    {
      method: "POST",
      headers: { ...buildApiHeaders(), "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    },
  );
  if (!resp.ok) {
    const envelope: any = await resp.json().catch(() => null);
    throw documentRequestError("生成文档元数据建议失败。", resp.status, envelope);
  }
  return unwrapEnvelope<DocumentMetadataSuggestion>(await resp.json());
}

export async function fetchDocumentMetadataSuggestions(
  apiPrefix: string,
  documentId: string,
  { limit = 20 }: { limit?: number } = {},
): Promise<DocumentMetadataSuggestion[]> {
  const normalized = `${documentId || ""}`.trim();
  if (!normalized) return [];
  const params = new URLSearchParams({ limit: `${limit}` });
  const resp = await apiFetch(
    `${buildApiEndpoint(apiPrefix, `documents/${encodeURIComponent(normalized)}/metadata-suggestions`)}?${params.toString()}`,
    { headers: buildApiHeaders() },
  );
  if (!resp.ok) {
    const envelope: any = await resp.json().catch(() => null);
    throw documentRequestError("读取文档元数据建议失败。", resp.status, envelope);
  }
  const payload = unwrapEnvelope<DocumentMetadataSuggestionListView>(await resp.json());
  return Array.isArray(payload?.suggestions) ? payload.suggestions : [];
}

export async function applyDocumentMetadataSuggestion(
  apiPrefix: string,
  documentId: string,
  suggestionId: string,
  payload: ApplyDocumentMetadataSuggestionInput = {},
): Promise<DocumentMetadataSuggestionApplyView> {
  const normalizedDocumentId = `${documentId || ""}`.trim();
  const normalizedSuggestionId = `${suggestionId || ""}`.trim();
  if (!normalizedDocumentId) throw new Error("缺少 document_id。");
  if (!normalizedSuggestionId) throw new Error("缺少 suggestion_id。");
  const resp = await apiFetch(
    buildApiEndpoint(
      apiPrefix,
      `documents/${encodeURIComponent(normalizedDocumentId)}/metadata-suggestions/${encodeURIComponent(normalizedSuggestionId)}/apply`,
    ),
    {
      method: "POST",
      headers: { ...buildApiHeaders(), "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    },
  );
  if (!resp.ok) {
    const envelope: any = await resp.json().catch(() => null);
    throw documentRequestError("应用文档元数据建议失败。", resp.status, envelope);
  }
  return unwrapEnvelope<DocumentMetadataSuggestionApplyView>(await resp.json());
}

export async function deleteDocument(apiPrefix: string, documentId: string, { force = false }: { force?: boolean } = {}): Promise<DocumentDeleteResultView> {
  const normalized = `${documentId || ""}`.trim();
  if (!normalized) throw new Error("缺少 document_id。");
  const params = force ? "?force=true" : "";
  const resp = await apiFetch(buildApiEndpoint(apiPrefix, `documents/${encodeURIComponent(normalized)}`) + params, { method: "DELETE", headers: buildApiHeaders() });
  if (!resp.ok) {
    const envelope: any = await resp.json().catch(() => null);
    // 保留 HTTP 状态：409 是文档下还有排队或运行中的任务（force=true 才删）
    throw documentRequestError("删除文档失败，请稍后重试。", resp.status, envelope);
  }
  return unwrapEnvelope<DocumentDeleteResultView>(await resp.json());
}

export async function translateDocument(
  apiPrefix: string,
  documentId: string,
  payload: Record<string, unknown> = {},
): Promise<DocumentJobSubmissionView> {
  const normalized = `${documentId || ""}`.trim();
  if (!normalized) throw new Error("缺少 document_id。");
  const resp = await apiFetch(buildApiEndpoint(apiPrefix, `documents/${encodeURIComponent(normalized)}/translate`), {
    method: "POST",
    headers: { ...buildApiHeaders(), "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!resp.ok) {
    const envelope: any = await resp.json().catch(() => null);
    throw pageQuotaErrorFromPayload(resp.status, envelope) ?? documentRequestError("发起翻译失败，请稍后重试。", resp.status, envelope);
  }
  return unwrapEnvelope<DocumentJobSubmissionView>(await resp.json());
}

export async function ocrDocument(
  apiPrefix: string,
  documentId: string,
  payload: Record<string, unknown> = {},
): Promise<any> {
  const normalized = `${documentId || ""}`.trim();
  if (!normalized) throw new Error("缺少 document_id。");
  const resp = await apiFetch(buildApiEndpoint(apiPrefix, `documents/${encodeURIComponent(normalized)}/ocr`), {
    method: "POST",
    headers: { ...buildApiHeaders(), "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!resp.ok) {
    const envelope: any = await resp.json().catch(() => null);
    throw pageQuotaErrorFromPayload(resp.status, envelope) ?? new Error(`${envelope?.message || "发起 OCR 失败，请稍后重试。"}(${resp.status})`);
  }
  return unwrapEnvelope(await resp.json());
}

export async function submitDocument(
  apiPrefix: string,
  documentId: string,
  payload: Record<string, unknown> = {},
): Promise<DocumentJobSubmissionView> {
  const workflow = `${(payload as Record<string, unknown>)?.workflow || ""}`.trim().toLowerCase();
  if (workflow === "ocr") {
    return ocrDocument(apiPrefix, documentId, payload);
  }
  return translateDocument(apiPrefix, documentId, payload);
}

export async function fetchDocumentJobs(
  apiPrefix: string,
  documentId: string,
  { limit = 50, offset = 0 }: DocumentJobsQuery = {},
): Promise<DocumentJobsView> {
  const normalized = `${documentId || ""}`.trim();
  if (!normalized) return { items: [] };
  const params = new URLSearchParams();
  params.set("limit", `${limit}`);
  params.set("offset", `${offset}`);
  const resp = await apiFetch(
    `${buildApiEndpoint(apiPrefix, `documents/${encodeURIComponent(normalized)}/jobs`)}?${params.toString()}`,
    { headers: buildApiHeaders() },
  );
  if (!resp.ok) throw new Error(`读取文档任务失败，请稍后重试。(${resp.status})`);
  const payload = unwrapEnvelope<DocumentJobsView>(await resp.json());
  return {
    ...payload,
    items: Array.isArray(payload?.items) ? payload.items : [],
  };
}
