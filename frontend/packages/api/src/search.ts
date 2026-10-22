// search — pure
import { apiFetch } from "./internal/runtime.js";
import { buildApiHeaders, unwrapEnvelope } from "./internal/runtime.js";
import { buildApiEndpoint } from "./http.js";
import type { SearchResultView } from "@retainpdf/contracts/library-documents";

export type { BlockSearchHit, SearchResultView } from "@retainpdf/contracts/library-documents";

export async function searchLibrary(apiPrefix: string, q: string, { limit = 20 }: { limit?: number } = {}): Promise<SearchResultView> {
  const query = `${q || ""}`.trim();
  if (!query) return { query, hits: [], limit, has_more: false };
  const params = new URLSearchParams();
  params.set("q", query);
  params.set("limit", `${limit}`);
  const resp = await apiFetch(`${buildApiEndpoint(apiPrefix, "search")}?${params.toString()}`, { headers: buildApiHeaders() });
  if (!resp.ok) throw new Error(`检索失败，请稍后重试。(${resp.status})`);
  return unwrapEnvelope<SearchResultView>(await resp.json());
}
