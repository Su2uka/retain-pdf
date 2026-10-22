import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const jobStatus = JSON.parse(readFileSync(resolve("job-status.v1.schema.json"), "utf8"));
const libraryBooks = JSON.parse(readFileSync(resolve("library-books.v1.schema.json"), "utf8"));
// ArtifactDisplayItemView 只给书架单本用，2026-10-22 从 library-books 删了。
const EXPECTED_SHARED_DEFINITIONS = [
  "InvocationSummaryView",
  "JobFailureBriefView",
  "JobListInvocationSummaryView",
  "JobListItemView",
  "JobListView",
  "JobProgressView",
  "JobStageRuntimeView",
  "JobStageSnapshotView",
  "JobStageStateView",
  "JobStagesView",
  "JobStatusKind",
  "JobTimestampsView",
  "ListJobsQuery",
  "WorkflowKind",
];

function withoutDescriptions(value) {
  if (Array.isArray(value)) return value.map(withoutDescriptions);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value)
      .filter(([key]) => key !== "description")
      .map(([key, nested]) => [key, withoutDescriptions(nested)]),
  );
}

test("duplicated Job definitions stay structurally identical", () => {
  const sharedNames = Object.keys(jobStatus.definitions)
    .filter((name) => Object.hasOwn(libraryBooks.definitions, name))
    .sort();
  assert.deepEqual(sharedNames, EXPECTED_SHARED_DEFINITIONS);

  for (const name of sharedNames) {
    assert.deepEqual(
      withoutDescriptions(libraryBooks.definitions[name]),
      withoutDescriptions(jobStatus.definitions[name]),
      `${name} drifted between job-status and library-books`,
    );
  }
});
