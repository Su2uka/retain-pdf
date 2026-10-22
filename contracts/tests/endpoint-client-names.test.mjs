import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";

// 端点表的 client_name（前端生成的函数名）和 transport（返回方式）。
// 格式、取值、不重名由 scripts/lint-schemas.mjs 查；这里钉住第 0 期（library-documents.v1）的名单。
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

test("the old response_body key is merged into transport", () => {
  for (const fileName of readdirSync(resolve(".")).filter((name) => name.endsWith(".schema.json"))) {
    for (const endpoint of read(fileName).endpoints || []) {
      assert.equal(Object.hasOwn(endpoint, "response_body"), false, `${fileName} ${endpoint.path} still has response_body`);
    }
  }
  const artifact = read("agent-calculation.v1.schema.json").endpoints.find((endpoint) => endpoint.path.endsWith("/artifacts/:artifact_id"));
  assert.equal(artifact.transport, "binary");
});
