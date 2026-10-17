import { r as C, l as J, h as $ } from "../markdown-payload-kK3ewW_I.js";
import { d as T } from "../pdf-document-config-DOSsufI-.js";
import { e as Re, f as we, a as me, b as he, i as _e, n as ke, c as Ue, p as ve, r as ge, d as Me, g as Ee, h as Ae, j as De, k as Le } from "../reader-regions-CP3ZMHPc.js";
const G = "/api/v1", y = 250;
function q(e, t) {
  return typeof globalThis.fetch == "function" ? globalThis.fetch(e, t) : Promise.reject(new Error(`fetchProtected not injected for ${e}`));
}
function S(e, t) {
  return e().then(
    (u) => ({ value: u, error: null }),
    (u) => ({ value: t, error: u })
  );
}
function X() {
  return Promise.resolve(null);
}
function Z() {
  return Promise.resolve({ items: [] });
}
function Q() {
  return Promise.resolve(null);
}
function Y() {
  return Promise.resolve(null);
}
function V() {
  return Promise.resolve({ items: [] });
}
function x() {
  return Promise.resolve(null);
}
function I({
  apiPrefix: e = G,
  loadJob: t = X,
  loadManifest: u = Z,
  loadMarkdown: r = Q,
  loadMarkdownDocument: s = Y,
  loadMarkdownSource: o = null,
  fetchMarkdownRange: l = null,
  loadRegions: f = V,
  loadMetadata: p = x,
  fetchProtectedResource: U = q,
  liveTranslation: h = null
} = {}) {
  const R = /* @__PURE__ */ new Map(), _ = /* @__PURE__ */ new Map();
  function w(a) {
    const n = _.get(a);
    if (n && Date.now() - n.at < y)
      return Promise.resolve(n.value);
    const d = R.get(a);
    if (d) return d;
    let c;
    try {
      c = Promise.resolve(t(a, e)).then((i) => {
        const v = Date.now();
        _.set(a, { at: v, value: i });
        for (const [E, A] of _)
          v - A.at >= y && _.delete(E);
        return i;
      }).finally(() => {
        R.get(a) === c && R.delete(a);
      });
    } catch (i) {
      c = Promise.reject(i);
    }
    return R.set(a, c), c;
  }
  async function F(a) {
    const [n, d] = await Promise.all([
      S(() => f(a, e), { items: [] }),
      S(() => p(a, e), null)
    ]);
    return {
      readerMetadata: d.value,
      regionsPayload: n.value,
      readerErrors: {
        regions: n.error,
        metadata: d.error
      }
    };
  }
  async function B(a, n = {}) {
    const d = n.includeOptionalArtifacts !== !1, c = w(a), i = u(a, e).catch((m) => {
      if (Number(m == null ? void 0 : m.status) === 404) return { items: [] };
      throw m;
    });
    if (!d) {
      const [m, W] = await Promise.all([c, i]);
      return {
        jobPayload: m,
        manifestPayload: W,
        readerMetadata: null,
        regionsPayload: { items: [] },
        readerErrors: { regions: null, metadata: null }
      };
    }
    const [v, E, A] = await Promise.all([
      c,
      i,
      F(a)
    ]);
    return {
      jobPayload: v,
      manifestPayload: E,
      ...A
    };
  }
  function z(a) {
    return w(a);
  }
  async function H(a) {
    const n = await J(
      () => s(a, e),
      () => r(a, e)
    );
    if ($(n)) return n;
    try {
      const d = await w(a), c = C(d, a);
      if (!c) return n;
      const i = await J(
        () => s(c, e),
        () => r(c, e)
      );
      return $(i) ? i : n;
    } catch {
      return n;
    }
  }
  async function K(a) {
    if (typeof o != "function") return null;
    let n = await o(a, e).catch(() => null);
    if (n != null && n.rawUrl) return n;
    try {
      const d = await w(a), c = C(d, a);
      return c ? (n = await o(c, e).catch(() => null), n != null && n.rawUrl ? n : null) : n;
    } catch {
      return n;
    }
  }
  function N(a, n, d, c, i) {
    return typeof l != "function" ? Promise.reject(new Error("fetchMarkdownRange not injected")) : l(a, n, d, c, i);
  }
  return Object.freeze({
    apiPrefix: e,
    fetchProtected: U,
    loadMarkdownPayload: H,
    loadMarkdownSource: K,
    loadMarkdownRange: N,
    loadJobPayload: z,
    loadReaderPayload: B,
    loadReaderOptionalArtifacts: F,
    liveTranslation: h
  });
}
const ue = I();
function O(e) {
  return `${e ?? ""}`.trim();
}
function g(e = "") {
  return `${e ?? ""}`.trim() ? `${e}`.trim() : "";
}
const P = 512 * 1024;
let k = null;
function D(e = g) {
  return {
    moduleUrl: e("build/pdf.mjs"),
    workerUrl: e("build/pdf.worker.mjs"),
    cmapUrl: e("cmaps/"),
    standardFontDataUrl: e("standard_fonts/")
  };
}
async function b({ resolvePdfjsVendorUrl: e = g } = {}) {
  const { moduleUrl: t, workerUrl: u } = D(e);
  if (!t)
    throw new Error("resolvePdfjsVendorUrl not injected");
  return k || (k = import(t).then((r) => (r.GlobalWorkerOptions.workerSrc = u, r)).catch((r) => {
    throw k = null, r;
  })), k;
}
function j(e, { resolveResourceUrl: t = O } = {}) {
  return t((e == null ? void 0 : e.resource_url) || (e == null ? void 0 : e.resource_path) || "");
}
function ee({
  url: e,
  configPort: t = T,
  resolvePdfjsVendorUrl: u = g
} = {}) {
  var o;
  if (!e)
    return null;
  const { cmapUrl: r, standardFontDataUrl: s } = D(u);
  return {
    url: e,
    httpHeaders: ((o = t == null ? void 0 : t.apiHeaders) == null ? void 0 : o.call(t)) ?? {},
    withCredentials: !1,
    disableRange: !1,
    disableStream: !1,
    rangeChunkSize: P,
    cMapUrl: r,
    cMapPacked: !0,
    standardFontDataUrl: s
  };
}
async function oe({
  itemOrUrl: e,
  configPort: t = T,
  fetchProtected: u = null,
  resolveResourceUrl: r = O,
  resolvePdfjsVendorUrl: s = g
} = {}) {
  const o = typeof e == "string" ? e : j(e, { resolveResourceUrl: r });
  if (!o)
    return null;
  const l = await b({ resolvePdfjsVendorUrl: s }), { cmapUrl: f, standardFontDataUrl: p } = D(s);
  if (o.startsWith("mock://") && typeof u == "function") {
    const U = await u(o), h = new Uint8Array(await U.arrayBuffer());
    return l.getDocument({
      data: h,
      cMapUrl: f,
      cMapPacked: !0,
      standardFontDataUrl: p
    }).promise;
  }
  return l.getDocument(ee({ url: o, configPort: t, resolvePdfjsVendorUrl: s })).promise;
}
function le() {
  k = null;
}
function M(e) {
  return `${e ?? ""}`.trim();
}
function L(e, t) {
  return (Array.isArray(e == null ? void 0 : e.items) ? e.items : []).find((r) => (r == null ? void 0 : r.artifact_key) === t && (r == null ? void 0 : r.ready)) || null;
}
function te(e, t, { resolveResourceUrl: u = M, findReadyManifestArtifact: r = L } = {}) {
  const s = r(e, t), o = `${(s == null ? void 0 : s.resource_url) || (s == null ? void 0 : s.resource_path) || ""}`.trim();
  return o ? u(o) : "";
}
function re(e, { resolveResourceUrl: t = M } = {}) {
  return t((e == null ? void 0 : e.resource_url) || (e == null ? void 0 : e.resource_path) || "");
}
function ne(e) {
  var o, l, f, p;
  if (!e) return null;
  const t = (e == null ? void 0 : e.actions) || {}, u = (e == null ? void 0 : e.artifacts) || {}, r = !!(((o = t.download_pdf) == null ? void 0 : o.enabled) ?? ((l = u.pdf) == null ? void 0 : l.ready) ?? (e == null ? void 0 : e.pdf_ready) ?? (e == null ? void 0 : e.output_pdf_ready)), s = `${((f = t.download_pdf) == null ? void 0 : f.url) || ((p = u.pdf) == null ? void 0 : p.url) || (e == null ? void 0 : e.pdf_url) || ""}`.trim();
  return { pdfEnabled: r, pdf: s ? M(s) : "" };
}
function ce(e) {
  var t;
  return ((t = e == null ? void 0 : e.readerJobId) == null ? void 0 : t.call(e)) || "";
}
function de(e, {
  findReadyManifestArtifact: t = L,
  resolveManifestArtifactUrl: u = (r, s) => te(r, s, { findReadyManifestArtifact: t })
} = {}) {
  const r = u(e, "source_pdf");
  return r || t(e, "source_pdf");
}
function ie(e, t, {
  resolveJobActions: u = ne,
  findReadyManifestArtifact: r = L,
  resolveReaderArtifactUrl: s = re,
  resolveResourceUrl: o = M
} = {}) {
  const l = e ? u(e) : null;
  if (l != null && l.pdfEnabled && (l != null && l.pdf))
    return l.pdf;
  const f = ["pdf", "translated_pdf", "result_pdf"];
  for (const h of f) {
    const R = r(t, h), w = s(R, { resolveResourceUrl: o }) || s(R);
    if (w)
      return w;
  }
  const p = `${(e == null ? void 0 : e.workflow) || (e == null ? void 0 : e.job_type) || ""}`.trim().toLowerCase();
  return ((l == null ? void 0 : l.pdfEnabled) || `${(e == null ? void 0 : e.status) || ""}`.trim().toLowerCase() === "succeeded" && p !== "ocr") && (e != null && e.job_id) ? o(`/api/v1/jobs/${encodeURIComponent(e.job_id)}/files/output/translated.pdf`) : "";
}
export {
  le as __resetPdfjsForTests,
  ee as buildPdfDocumentOptions,
  I as createReaderDataPort,
  ue as defaultReaderDataPort,
  Re as extractReaderFormulaLatex,
  we as findReaderRegion,
  me as findReaderRegionByAssetUrl,
  he as findReaderRegionByCitation,
  _e as isStructuredReaderRegion,
  oe as loadPdfDocument,
  ke as normalizeReaderMetadata,
  Ue as normalizeReaderRegions,
  ve as projectReaderRegion,
  ge as readerRegionContent,
  Me as readerRegionCopyText,
  Ee as readerRegionKind,
  Ae as readerRegionKindForRegion,
  De as regionBoxForPane,
  j as resolveReaderArtifactUrl,
  ce as resolveReaderJobId,
  Le as resolveReaderRegionHighlight,
  de as resolveReaderSourcePdf,
  ie as resolveReaderTranslatedPdfUrl
};
//# sourceMappingURL=data.js.map
