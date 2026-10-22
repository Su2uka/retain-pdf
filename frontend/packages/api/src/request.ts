// 生成的请求函数（src/generated/）共用的传输层：拼地址和查询串、带 key、发 JSON、拆 {code, data} 信封。
// 给用户看的报错文案不在这里定——各领域的手写函数通过 onError 传进来，生成层只负责把失败的
// 状态码和后端回的错误体交出去。方案见 docs/ops/planning/api-client-codegen/。

import { apiFetch, buildApiHeaders, buildApiUrl, unwrapEnvelope } from "./internal/runtime.js";
import { attachErrorDetails, type HttpError } from "./http.js";

export type QueryValue = string | number | boolean | null | undefined | ReadonlyArray<string | number>;

export interface RequestOptions {
  signal?: AbortSignal;
  /** 把失败的响应变成错误对象；不传时用后端的 message 加状态码，并挂上 errorCode、reason。 */
  onError?: (status: number, payload: any) => Error;
}

export interface RequestSpec {
  apiPrefix: string | undefined;
  method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  /** 相对 /api/v1 的路径，路径参数已经编码。 */
  path: string;
  query?: object;
  body?: unknown;
  options?: RequestOptions;
}

/** 查询参数按传入顺序编码：undefined、null、空串不发；数组按后端约定用逗号连接（job_ids=a,b）。 */
export function encodeQuery(query: object | undefined): string {
  if (!query) return "";
  const params = new URLSearchParams();
  for (const [key, raw] of Object.entries(query as Record<string, QueryValue>)) {
    if (raw === undefined || raw === null) continue;
    const value = Array.isArray(raw) ? raw.join(",") : `${raw}`;
    if (value === "") continue;
    params.set(key, value);
  }
  return params.toString();
}

export function encodePathParam(value: string): string {
  return encodeURIComponent(`${value ?? ""}`);
}

export async function request<T>({ apiPrefix, method, path, query, body, options = {} }: RequestSpec): Promise<T> {
  const search = encodeQuery(query);
  const url = `${buildApiUrl(apiPrefix, path)}${search ? `?${search}` : ""}`;
  // GET 不带 Content-Type：它不在 CORS 安全列表里，会把每个 GET 都变成预检请求。
  const init: RequestInit = body === undefined
    ? { headers: buildApiHeaders() }
    : { headers: { ...buildApiHeaders(), "Content-Type": "application/json" }, body: JSON.stringify(body) };
  if (method !== "GET") init.method = method;
  if (options.signal) init.signal = options.signal;
  const resp = await apiFetch(url, init);
  if (!resp.ok) {
    const payload: any = await resp.json().catch(() => null);
    if (options.onError) throw options.onError(resp.status, payload);
    const error = new Error(`${payload?.message || "请求失败，请稍后重试。"}(${resp.status})`) as HttpError;
    error.status = resp.status;
    error.url = url;
    throw attachErrorDetails(error, payload);
  }
  return unwrapEnvelope<T>(await resp.json());
}

/** 只用固定文案加状态码，例如「读取文档库失败，请稍后重试。(500)」。 */
export function failWith(message: string): RequestOptions["onError"] {
  return (status) => new Error(`${message}(${status})`);
}

/** 优先用后端回的 message，没有时用固定文案，都加状态码。 */
export function failWithServerMessage(fallback: string): RequestOptions["onError"] {
  return (status, payload) => new Error(`${payload?.message || fallback}(${status})`);
}
