import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const read = name => JSON.parse(readFileSync(new URL(`../${name}`, import.meta.url)));
const schema = read("job-events.v2.schema.json");

test("events v2 has one route and no legacy offset operation", () => {
  // 旧地址 /jobs/:id/events、/ocr/jobs/:id/events 2026-10-18 删了。
  assert.deepEqual(schema.endpoints.map(endpoint => endpoint.path), ["/api/v1/jobs/:job_id/progress/events"]);
  assert.equal(Object.hasOwn(schema.endpoints[0], "deprecated_aliases"), false);
  assert.ok(read("job-status.v1.schema.json").endpoints.every(endpoint => !endpoint.path.endsWith("/events")));
  const query = schema.definitions.ListJobEventsQuery;
  assert.equal(query.additionalProperties, false);
  assert.equal(Object.hasOwn(query.properties, "offset"), false);
  assert.deepEqual(query.not, { required: ["cursor", "start"] });
  assert.deepEqual(query.properties.start.enum, ["head", "tail"]);
});

test("events v2 requires stable identity and an explicit bounded cursor response", () => {
  const { JobEventRecord, JobEventListView } = schema.definitions;
  assert.ok(JobEventRecord.required.includes("event_id"));
  assert.equal(JobEventRecord.properties.seq.minimum, 1);
  assert.equal(JobEventListView.properties.protocol_version.const, 2);
  assert.deepEqual(JobEventListView.required, ["protocol_version", "items", "next_cursor", "has_more", "limit"]);
  assert.equal(JobEventListView.properties.items.maxItems, 500);
});
