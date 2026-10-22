import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";

// 端点表的 client_name（前端生成的函数名）和 transport（返回方式）。
// 格式、取值、不重名由 scripts/lint-schemas.mjs 查；这里钉住第 0 期（library-documents.v1）和第 1a 期的名单。
const read = (name) => JSON.parse(readFileSync(resolve(name), "utf8"));

test("library-documents.v1 endpoints all carry the phase-0 client names over the json transport", () => {
  const contract = read("library-documents.v1.schema.json");
  const table = contract.endpoints.map(({ method, path, client_name, transport }) => [method, path, client_name, transport]);
  assert.deepEqual(table, [
    ["GET", "/api/v1/documents", "listDocuments", "json"],
    ["GET", "/api/v1/documents/:document_id", "getDocument", "json"],
    ["PATCH", "/api/v1/documents/:document_id", "patchDocument", "json"],
    ["DELETE", "/api/v1/documents/:document_id", "deleteDocument", "json"],
    ["GET", "/api/v1/documents/:document_id/reading", "getDocumentReading", "json"],
    ["GET", "/api/v1/documents/:document_id/translation-coverage", "getTranslationCoverage", "json"],
    ["GET", "/api/v1/documents/:document_id/files", "listDocumentFiles", "json"],
    ["GET", "/api/v1/documents/:document_id/metadata-suggestions", "listMetadataSuggestions", "json"],
    ["POST", "/api/v1/documents/:document_id/metadata-suggestions", "createMetadataSuggestion", "json"],
    ["POST", "/api/v1/documents/:document_id/metadata-suggestions/:suggestion_id/apply", "applyMetadataSuggestion", "json"],
    ["GET", "/api/v1/search", "searchLibrary", "json"],
    ["GET", "/api/v1/collections", "listCollections", "json"],
    ["POST", "/api/v1/collections", "createCollection", "json"],
    ["PATCH", "/api/v1/collections/:collection_id", "patchCollection", "json"],
    ["DELETE", "/api/v1/collections/:collection_id", "deleteCollection", "json"],
    ["POST", "/api/v1/collections/:collection_id/documents", "addCollectionDocuments", "json"],
    ["DELETE", "/api/v1/collections/:collection_id/documents/:document_id", "removeCollectionDocument", "json"],
  ]);
});

// 第 1a 期补名字的 7 份契约。之后几期在别的契约里起的名字各有自己的 test。
const PHASE_1A_CONTRACTS = new Set([
  "create-job.v1.schema.json",
  "job-data.v1.schema.json",
  "job-events.v2.schema.json",
  "job-status.v1.schema.json",
  "library-books.v1.schema.json",
  "reader-data.v1.schema.json",
  "token-usage.v1.schema.json",
]);

test("phase-1a endpoints carry the client names from the frontend list over the json transport", () => {
  const named = [];
  for (const fileName of readdirSync(resolve(".")).filter((name) => PHASE_1A_CONTRACTS.has(name))) {
    for (const { method, path, client_name, transport } of read(fileName).endpoints || []) {
      if (client_name !== undefined) named.push([fileName, method, path, client_name, transport]);
    }
  }
  named.sort((left, right) => left[3].localeCompare(right[3]));
  assert.deepEqual(named, [
    ["create-job.v1.schema.json", "POST", "/api/v1/jobs", "createJob", "json"],
    ["library-books.v1.schema.json", "DELETE", "/api/v1/jobs/:job_id", "deleteJob", "json"],
    ["token-usage.v1.schema.json", "GET", "/api/v1/documents/:document_id/usage", "getDocumentUsage", "json"],
    ["job-status.v1.schema.json", "GET", "/api/v1/jobs/:job_id", "getJob", "json"],
    ["job-data.v1.schema.json", "GET", "/api/v1/jobs/:job_id/data/:dataset", "getJobDataset", "json"],
    ["token-usage.v1.schema.json", "GET", "/api/v1/jobs/:job_id/reports/usage", "getJobUsage", "json"],
    ["reader-data.v1.schema.json", "GET", "/api/v1/jobs/:job_id/progress/layout", "getLiveLayout", "json"],
    ["reader-data.v1.schema.json", "GET", "/api/v1/jobs/:job_id/progress/pages/:page", "getLivePage", "json"],
    ["reader-data.v1.schema.json", "GET", "/api/v1/jobs/:job_id/reader/markdown", "getReaderMarkdown", "json"],
    ["reader-data.v1.schema.json", "GET", "/api/v1/jobs/:job_id/reader/metadata", "getReaderMetadata", "json"],
    ["reader-data.v1.schema.json", "GET", "/api/v1/jobs/:job_id/reader/regions", "getReaderRegions", "json"],
    ["token-usage.v1.schema.json", "GET", "/api/v1/usage", "getUsageSummary", "json"],
    ["job-data.v1.schema.json", "GET", "/api/v1/jobs/:job_id/data", "listJobDatasets", "json"],
    ["job-events.v2.schema.json", "GET", "/api/v1/jobs/:job_id/progress/events", "listJobEvents", "json"],
    ["reader-data.v1.schema.json", "GET", "/api/v1/jobs/:job_id/files", "listJobFiles", "json"],
    ["job-status.v1.schema.json", "GET", "/api/v1/jobs", "listJobs", "json"],
  ]);
  // GET /api/v1/jobs 在 library-books.v1 也登记了（为了导出 JobListView），名字只写在 job-status.v1。
  const duplicate = read("library-books.v1.schema.json").endpoints.find((endpoint) => endpoint.method === "GET" && endpoint.path === "/api/v1/jobs");
  assert.equal(duplicate.client_name, undefined);
});

test("the old response_body key is merged into transport", () => {
  for (const fileName of readdirSync(resolve(".")).filter((name) => name.endsWith(".schema.json"))) {
    for (const endpoint of read(fileName).endpoints || []) {
      assert.equal(Object.hasOwn(endpoint, "response_body"), false, `${fileName} ${endpoint.path} still has response_body`);
    }
  }
  const artifact = read("agent-calculation.v1.schema.json").endpoints.find((endpoint) => endpoint.path.endsWith("/artifacts/:artifact_id"));
  assert.equal(artifact.transport, "binary");
});

// 第 1b 期：任务操作（job-actions.v1）和任务报告（job-reports.v1）两份新契约，每条端点都有名字。
test("phase-1b job actions and reports endpoints carry the client names from the frontend list", () => {
  const table = (fileName) => read(fileName).endpoints.map(({ method, path, client_name, transport, input, query, output }) => [
    method, path, client_name, transport, input, query, output,
  ]);
  assert.deepEqual(table("job-actions.v1.schema.json"), [
    ["GET", "/api/v1/jobs/:job_id/actions", "listJobActions", "json", undefined, undefined, "StageActionsView"],
    // 取消不读请求体。
    ["POST", "/api/v1/jobs/:job_id/actions/cancel", "cancelJob", "json", undefined, undefined, "JobSubmissionView"],
    ["POST", "/api/v1/jobs/:job_id/actions/resolve-ocr-ambiguity", "resolveOcrAmbiguity", "json", "OcrAmbiguityResolutionRequest", undefined, "OcrAmbiguityResolutionView"],
    ["GET", "/api/v1/jobs/:job_id/actions/resume", "getResumePlan", "json", undefined, undefined, "JobResumePlanView"],
    ["POST", "/api/v1/jobs/:job_id/actions/resume", "resumeJob", "json", "ResumeJobInput", undefined, "JobSubmissionView"],
    ["POST", "/api/v1/jobs/:job_id/actions/retry-stage", "retryJobStage", "json", "RetryStageRequest", undefined, "RetryStageSubmissionView"],
  ]);
  assert.deepEqual(table("job-reports.v1.schema.json"), [
    ["GET", "/api/v1/jobs/:job_id/reports/failure", "getFailureReport", "json", undefined, undefined, "JobDiagnosticsView"],
    ["GET", "/api/v1/jobs/:job_id/reports/quality", "getQualityReport", "json", undefined, undefined, "QualitySummaryView"],
    ["GET", "/api/v1/jobs/:job_id/reports/quality/items", "listQualityItems", "json", undefined, "QualityItemsQuery", "QualityItemsView"],
    ["GET", "/api/v1/jobs/:job_id/reports/translation", "getTranslationReport", "json", undefined, undefined, "TranslationDiagnosticsView"],
  ]);
});
