// jobs-actions — pure (no mock)
import { apiFetch } from "./internal/runtime.js";
import { buildApiHeaders, unwrapEnvelope } from "./internal/runtime.js";
import { buildJobDetailEndpoint, submitJson } from "./http.js";
export async function fetchJobDiagnostics(jobId, apiPrefix) {
    const resp = await apiFetch(`${buildJobDetailEndpoint(jobId, apiPrefix)}/reports/failure`, { headers: buildApiHeaders() });
    if (!resp.ok) {
        if (resp.status === 404)
            return null;
        throw new Error(`读取失败诊断失败，请稍后重试。(${resp.status})`);
    }
    return unwrapEnvelope(await resp.json());
}
export async function fetchResumePlan(jobId, apiPrefix) {
    const resp = await apiFetch(`${buildJobDetailEndpoint(jobId, apiPrefix)}/actions/resume`, { headers: buildApiHeaders() });
    if (!resp.ok) {
        if (resp.status === 404)
            return null;
        throw new Error(`读取恢复计划失败，请稍后重试。(${resp.status})`);
    }
    return unwrapEnvelope(await resp.json());
}
/** body 可带 `{ overrides: { translation: { api_key | credential_ref } } }`：续跑出来的新任务用这把 key。 */
export async function resumeJob(jobId, apiPrefix, body = {}) {
    return submitJson(`${buildJobDetailEndpoint(jobId, apiPrefix)}/actions/resume`, body);
}
export async function cancelJob(jobId, apiPrefix) {
    return submitJson(`${buildJobDetailEndpoint(jobId, apiPrefix)}/actions/cancel`, {});
}
/** OCR 任务的取消和其它任务同一个地址（后端按任务类型分支）；保留这个名字给老调用方。 */
export async function cancelOcrJob(jobId, apiPrefix) {
    return cancelJob(jobId, apiPrefix);
}
export async function resolveOcrAmbiguity(jobId, apiPrefix, request) {
    return submitJson(`${buildJobDetailEndpoint(jobId, apiPrefix)}/actions/resolve-ocr-ambiguity`, request);
}
export async function fetchJobStageActions(jobId, apiPrefix) {
    const resp = await apiFetch(`${buildJobDetailEndpoint(jobId, apiPrefix)}/actions`, { headers: buildApiHeaders() });
    if (!resp.ok) {
        if (resp.status === 404)
            return null;
        throw new Error(`读取阶段操作失败，请稍后重试。(${resp.status})`);
    }
    return unwrapEnvelope(await resp.json());
}
/** retry-stage 请求体认的字段（后端 RetryStageRequest）。后端会对不认识的字段报 400。 */
const RETRY_STAGE_BODY_FIELDS = ["mode", "create_new_job", "overrides", "ambiguous_request_policy", "refine"];
/**
 * `payload` 里既有请求体字段，也可能带书的信息（document_id、title、cover_url…），后者只用来补返回值。
 * 请求体只发 RETRY_STAGE_BODY_FIELDS 里的字段。
 */
export async function retryJobStage(jobId, apiPrefix, stage, payload = {}) {
    const normalizedStage = `${stage || ""}`.trim();
    if (!normalizedStage)
        throw new Error("阶段重试失败: 缺少 stage");
    const body = { stage: normalizedStage };
    for (const field of RETRY_STAGE_BODY_FIELDS) {
        if (payload?.[field] !== undefined)
            body[field] = payload[field];
    }
    const result = await submitJson(`${buildJobDetailEndpoint(jobId, apiPrefix)}/actions/retry-stage`, body);
    const bookMeta = payload && typeof payload === "object" ? payload : {};
    const nextJobId = `${result?.job_id || result?.id || jobId}`.trim();
    return {
        ...result,
        job_id: nextJobId,
        source_job_id: jobId,
        document_id: result?.document_id || bookMeta.document_id,
        title: bookMeta.title || bookMeta.display_name || result?.title,
        display_name: bookMeta.display_name || bookMeta.title || result?.display_name,
        cover_url: bookMeta.cover_url || result?.cover_url,
        thumbnail_url: bookMeta.thumbnail_url || result?.thumbnail_url,
        page_count: bookMeta.page_count ?? result?.page_count,
        library_only: false,
        active_job_id: nextJobId,
    };
}
