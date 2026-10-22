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

test("phase-1a endpoints carry the client names from the frontend list over the json transport", () => {
  const named = [];
  for (const fileName of readdirSync(resolve(".")).filter((name) => name.endsWith(".schema.json") && name !== "library-documents.v1.schema.json")) {
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
