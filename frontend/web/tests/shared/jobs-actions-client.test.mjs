import test from "node:test";
import assert from "node:assert/strict";

globalThis.window = {
  location: { protocol: "http:", hostname: "127.0.0.1" },
  __FRONT_RUNTIME_CONFIG__: {
    apiBase: "http://127.0.0.1:41000",
    xApiKey: "jobs-actions-key",
  },
};

const {
  cancelJob,
  cancelOcrJob,
  resolveOcrAmbiguity,
  retryJobStage,
} = await import("@retainpdf/api/jobs-actions");

function okResponse(data) {
  return new Response(JSON.stringify({ data }), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

test("jobs actions:普通取消、OCR 取消同一个端点（后端按类型分支），ambiguity 恢复走自己的", async () => {
  const calls = [];
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (url, options) => {
    calls.push({ url: `${url}`, options });
    return okResponse({
      resolution: "accept_duplicate_risk",
      submission: { job_id: "ocr-recovery-1" },
    });
  };

  try {
    await cancelJob("job id", "/api/v1");
    await cancelOcrJob("job id", "/api/v1");
    await resolveOcrAmbiguity("job id", "/api/v1", {
      resolution: "accept_duplicate_risk",
      resolution_revision: 7,
    });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(calls.map(({ url }) => url), [
    "http://127.0.0.1:41000/api/v1/jobs/job%20id/actions/cancel",
    "http://127.0.0.1:41000/api/v1/jobs/job%20id/actions/cancel",
    "http://127.0.0.1:41000/api/v1/jobs/job%20id/actions/resolve-ocr-ambiguity",
  ]);
  for (const { options } of calls) {
    assert.equal(options.method, "POST");
    assert.equal(options.headers["X-API-Key"], "jobs-actions-key");
  }
  assert.deepEqual(JSON.parse(calls[0].options.body), {});
  assert.deepEqual(JSON.parse(calls[1].options.body), {});
  assert.deepEqual(JSON.parse(calls[2].options.body), {
    resolution: "accept_duplicate_risk",
    resolution_revision: 7,
  });
});

test("jobs actions:409 错误保留 status 供前端刷新诊断", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async () => new Response(
    JSON.stringify({ message: "revision stale" }),
    { status: 409, headers: { "Content-Type": "application/json" } },
  );
  try {
    await assert.rejects(
      resolveOcrAmbiguity("job-stale", "/api/v1", {
        resolution: "accept_duplicate_risk",
        resolution_revision: 3,
      }),
      (error) => error.status === 409 && /revision stale/.test(error.message),
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("jobs actions:409 的 error.code 和 details.reason 挂到错误对象上", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async () => new Response(
    JSON.stringify({
      code: 40900,
      message: "job is running; cancel it first",
      error: { code: "CONFLICT", http_status: 409, details: { reason: "job_active" } },
    }),
    { status: 409, headers: { "Content-Type": "application/json" } },
  );
  try {
    await assert.rejects(
      retryJobStage("job-1", "/api/v1", "render", {}),
      (error) => error.status === 409 && error.errorCode === "CONFLICT" && error.reason === "job_active",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
  // 普通错误的 details 是 {}：不挂 reason。
  globalThis.fetch = async () => new Response(
    JSON.stringify({ code: 40000, message: "bad", error: { code: "BAD_REQUEST", details: {} } }),
    { status: 400, headers: { "Content-Type": "application/json" } },
  );
  try {
    await assert.rejects(
      retryJobStage("job-1", "/api/v1", "render", {}),
      (error) => error.status === 400 && error.reason === undefined && error.errorCode === "BAD_REQUEST",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("retry-stage 请求体只发后端认的字段，书的信息只用来补返回值", async () => {
  const calls = [];
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (url, options) => {
    calls.push({ url: `${url}`, options });
    return new Response(JSON.stringify({ code: 0, message: "ok", data: { job_id: "job-retry-1", workflow: "book" } }), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    });
  };
  try {
    const result = await retryJobStage("job-1", "/api/v1", "refine", {
      // 书籍详情、任务卡会把书的信息和请求字段混在一个对象里传进来。
      document_id: "doc-1",
      title: "书名",
      display_name: "书名",
      cover_url: "/cover.jpg",
      thumbnail_url: "/thumb.jpg",
      page_count: 12,
      mode: "fresh",
      create_new_job: false,
      overrides: { render: { engine: "typst" } },
      ambiguous_request_policy: "accept_duplicate_risk",
      refine: { mode: "editorial", start_page: 3 },
    });
    assert.equal(calls.length, 1);
    assert.match(calls[0].url, /\/api\/v1\/jobs\/job-1\/actions\/retry-stage$/);
    assert.deepEqual(JSON.parse(calls[0].options.body), {
      stage: "refine",
      mode: "fresh",
      create_new_job: false,
      overrides: { render: { engine: "typst" } },
      ambiguous_request_policy: "accept_duplicate_risk",
      refine: { mode: "editorial", start_page: 3 },
    });
    assert.equal(result.job_id, "job-retry-1");
    assert.equal(result.document_id, "doc-1");
    assert.equal(result.title, "书名");
    assert.equal(result.page_count, 12);

    await retryJobStage("job-1", "/api/v1", "render", { document_id: "doc-1", title: "书名" });
    assert.deepEqual(JSON.parse(calls[1].options.body), { stage: "render" }, "没给请求字段时只发 stage");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("删书被拒的 409 带上 error.details.reason，书库能分清“任务在跑”和“别的任务在用它的产物”", async () => {
  const { deleteLibraryBook } = await import("@retainpdf/api/library-books");
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async () => new Response(JSON.stringify({
    code: 409,
    message: "other jobs still use this job's artifacts: job-r (uses job-a); delete them first or pass force=true",
    error: { code: "CONFLICT", details: { reason: "job_has_dependents" } },
  }), { status: 409, headers: { "Content-Type": "application/json" } });
  try {
    await assert.rejects(deleteLibraryBook("/api/v1", "job-a"), (error) => {
      assert.equal(error.status, 409);
      assert.equal(error.errorCode, "CONFLICT");
      assert.equal(error.reason, "job_has_dependents");
      assert.match(error.message, /job-r/);
      return true;
    });
  } finally {
    globalThis.fetch = originalFetch;
  }
});
