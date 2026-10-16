import { n as y } from "./block-key-BTxcG28S.js";
function g(t) {
  return t && typeof t == "object" && !Array.isArray(t) ? t : null;
}
function M(t) {
  const n = g(t);
  return n && "data" in n ? n.data : t;
}
function h(t) {
  const n = Number(t);
  return Number.isFinite(n) && n > 0 ? n : null;
}
function w(t) {
  const n = g(t);
  if (!n || !Array.isArray(n.bbox) || n.bbox.length !== 4) return null;
  const e = n.bbox.map(Number);
  if (!e.every(Number.isFinite)) return null;
  const s = h(n.page);
  if (s == null) return null;
  const [o, a, r, u] = e, l = Math.min(o, r), i = Math.min(a, u), f = Math.max(o, r), c = Math.max(a, u);
  if (f <= l || c <= i) return null;
  const p = `${n.unit || "pdf_point"}`.trim().toLowerCase();
  if (p !== "pdf_point" && p !== "pt") return null;
  const d = `${n.origin || "top_left"}`.trim().toLowerCase();
  return d !== "top_left" && d !== "bottom_left" ? null : {
    page: Math.floor(s),
    bbox: [l, i, f, c],
    unit: "pdf_point",
    origin: d,
    text: `${n.text || ""}`
  };
}
function A(t) {
  const n = {}, e = Number((t == null ? void 0 : t.reading_order) ?? (t == null ? void 0 : t.readingOrder));
  ((t == null ? void 0 : t.reading_order) != null || (t == null ? void 0 : t.readingOrder) != null) && Number.isFinite(e) && (n.readingOrder = e);
  const s = Number((t == null ? void 0 : t.heading_level) ?? (t == null ? void 0 : t.headingLevel));
  Number.isFinite(s) && s > 0 && (n.headingLevel = s);
  const o = `${(t == null ? void 0 : t.sub_type) ?? (t == null ? void 0 : t.subType) ?? ""}`.trim();
  o && (n.subType = o);
  const a = `${(t == null ? void 0 : t.continuation_group_id) ?? (t == null ? void 0 : t.continuationGroupId) ?? ""}`.trim();
  a && (n.continuationGroupId = a);
  const r = (t == null ? void 0 : t.translated_block_text) ?? (t == null ? void 0 : t.translatedBlockText);
  return typeof r == "string" && r && (n.translatedBlockText = r), n;
}
function F(t) {
  const n = M(t), e = g(n), s = Array.isArray(n) ? n : Array.isArray(e == null ? void 0 : e.items) ? e.items : [], o = [];
  for (const a of s) {
    const r = g(a), u = `${(r == null ? void 0 : r.item_id) || (r == null ? void 0 : r.itemId) || ""}`.trim(), l = w(r == null ? void 0 : r.source), i = w(r == null ? void 0 : r.translated);
    !u || !l || !i || o.push({
      itemId: u,
      source: l,
      translated: i,
      markdown: `${(r == null ? void 0 : r.markdown) || ""}`,
      regionType: `${(r == null ? void 0 : r.region_type) || (r == null ? void 0 : r.regionType) || ""}`,
      status: `${(r == null ? void 0 : r.status) || ""}`,
      assetIds: (Array.isArray(r == null ? void 0 : r.asset_ids) ? r.asset_ids : Array.isArray(r == null ? void 0 : r.assetIds) ? r.assetIds : []).map((f) => `${f || ""}`.trim()).filter(Boolean),
      assetUrls: (Array.isArray(r == null ? void 0 : r.asset_urls) ? r.asset_urls : Array.isArray(r == null ? void 0 : r.assetUrls) ? r.assetUrls : []).map((f) => `${f || ""}`.trim()).filter(Boolean),
      ...A(r)
    });
  }
  return o;
}
function N(t) {
  const n = `${t || ""}`.trim().toLowerCase().replace(/[\s-]+/g, "_");
  return n.includes("formula") || n.includes("equation") ? "formula" : n.includes("table") ? "table" : n.includes("figure") || n.includes("image") || n.includes("chart") || n.includes("seal") ? "figure" : n.includes("text") || n.includes("title") || n.includes("paragraph") || n.includes("reference") || n.includes("caption") ? "text" : "region";
}
function R(t) {
  const n = N(t.regionType);
  if (n !== "region") return n;
  if (t.assetIds.length || t.assetUrls.length) return "figure";
  const e = `${t.markdown || t.source.text || t.translated.text || ""}`.trim();
  return /^<table(?:\s|>)/i.test(e) || /\n\s*\|?\s*:?-{3,}/.test(e) ? "table" : /^\$\$[\s\S]+\$\$$/.test(e) || /^\\\[[\s\S]+\\\]$/.test(e) || /^\\begin\{(?:equation|align|gather|multline)\*?\}/.test(e) ? "formula" : e ? "text" : n;
}
function L(t) {
  const n = R(t);
  return n === "formula" || n === "table" || n === "figure";
}
function k(t, n) {
  return `${m(t, n).text || t.markdown || ""}`.trim();
}
function z(t, n) {
  const e = k(t, n) || (n === "translated" ? `${t.source.text || ""}`.trim() : "");
  return R(t) === "formula" ? C(e) : e;
}
function C(t) {
  let n = `${t || ""}`.trim();
  if (!n) return "";
  const e = n.match(/^```(?:latex|tex|math)?\s*([\s\S]*?)\s*```$/i);
  e && (n = e[1].trim());
  const s = [
    ["$$", "$$"],
    ["\\[", "\\]"],
    ["\\(", "\\)"],
    ["$", "$"]
  ];
  for (const [o, a] of s)
    if (n.startsWith(o) && n.endsWith(a) && n.length > o.length + a.length)
      return n.slice(o.length, -a.length).trim();
  return n;
}
function $(t) {
  const n = g(t);
  if (!n) return null;
  const e = [];
  for (const o of Array.isArray(n.pages) ? n.pages : []) {
    const a = g(o), r = h(a == null ? void 0 : a.page), u = h(a == null ? void 0 : a.width), l = h(a == null ? void 0 : a.height);
    r == null || u == null || l == null || e.push({ page: Math.floor(r), width: u, height: l });
  }
  if (!e.length) return null;
  const s = h(n.page_count ?? n.pageCount);
  return {
    pageCount: s == null ? e.length : Math.floor(s),
    pages: e
  };
}
function U(t) {
  const n = g(M(t));
  return {
    source: $(n == null ? void 0 : n.source),
    translated: $(n == null ? void 0 : n.translated)
  };
}
function I(t, n) {
  const e = y(n);
  return e && t.find((s) => y(s.itemId) === e) || null;
}
function b(t) {
  return `${t || ""}`.normalize("NFKC").toLocaleLowerCase().replace(/[\p{P}\p{S}\s]+/gu, "").trim();
}
function B(t) {
  const n = `${t || ""}`.trim();
  if (!n) return [];
  const e = n.split(/\n\s*\n/g).map(b).filter(Boolean), s = n.split(">").map(b).filter(Boolean), o = [...e.reverse(), ...s.reverse(), b(n)];
  return [...new Set(o)].filter((a) => a.length >= 16);
}
function S(t, n) {
  if (!t || !n) return 0;
  if (t.includes(n)) return 1e4 + n.length;
  const e = Math.min(72, n.length);
  if (e < 24) return 0;
  const s = Math.min(32, Math.max(0, n.length - e));
  for (let o = 0; o <= s; o += 4) {
    const a = n.slice(o, o + e);
    if (a.length >= 24 && t.includes(a))
      return a.length * 100 - o;
  }
  return 0;
}
function v(t, n) {
  if (!n) return null;
  const e = I(t, n.block_id);
  if (e) return e;
  const s = B(n.snippet);
  if (!s.length) return null;
  const o = n.page_idx != null ? Number(n.page_idx) + 1 : n.page != null ? Number(n.page) : null, a = Number.isFinite(o) && Number(o) >= 1 ? t.filter((i) => i.source.page === Math.floor(Number(o)) || i.translated.page === Math.floor(Number(o))) : t;
  let r = null, u = 0, l = !1;
  for (const i of a) {
    const f = [i.source.text, i.translated.text, i.markdown].map(b).filter(Boolean);
    let c = 0;
    for (const p of s)
      for (const d of f)
        c = Math.max(c, S(d, p));
    c > u ? (r = i, u = c, l = !1) : c > 0 && c === u && (l = !0);
  }
  return u > 0 && !l ? r : null;
}
function _(t) {
  let n = `${t || ""}`.trim().replace(/\\/g, "/");
  if (!n) return "";
  try {
    n = decodeURIComponent(new URL(n, "http://retainpdf.local/").pathname);
  } catch {
    try {
      n = decodeURIComponent(n);
    } catch {
    }
  }
  for (const e of ["/files/output/ocr/images/", "/markdown/images/"]) {
    const s = n.toLowerCase().indexOf(e);
    if (s >= 0) {
      n = n.slice(s + e.length);
      break;
    }
  }
  return n.replace(/^\.?\/?(?:images\/)?/i, "").replace(/\/{2,}/g, "/");
}
function O(t, n, e) {
  const s = _(n);
  if (!s) return null;
  const o = Number(e);
  return (Number.isFinite(o) && o >= 1 ? t.filter((r) => r.source.page === Math.floor(o)) : t).find((r) => [...r.assetUrls, ...r.assetIds].some((u) => {
    const l = _(u);
    return !!l && (l === s || s.endsWith(`/${l}`) || l.endsWith(`/${s}`));
  })) || null;
}
function m(t, n) {
  return n === "translated" ? t.translated : t.source;
}
function P(t, n, e) {
  if (!t || !n) return null;
  const s = m(t, e), o = e === "translated" ? n.translated : n.source || n.translated, a = o == null ? void 0 : o.pages.find((r) => r.page === s.page);
  return a ? { itemId: t.itemId, region: t, box: s, pageSize: a } : null;
}
function K(t, n, e) {
  if (!t || n <= 0 || e <= 0) return null;
  const { box: s, pageSize: o } = t;
  if (o.width <= 0 || o.height <= 0) return null;
  const [a, r, u, l] = s.bbox, i = s.origin === "bottom_left" ? o.height - l : r, f = s.origin === "bottom_left" ? o.height - r : l, c = Math.max(0, Math.min(n, a / o.width * n)), p = Math.max(c, Math.min(n, u / o.width * n)), d = Math.max(0, Math.min(e, i / o.height * e)), x = Math.max(d, Math.min(e, f / o.height * e));
  return p <= c || x <= d ? null : { left: c, top: d, width: p - c, height: x - d };
}
export {
  O as a,
  v as b,
  F as c,
  z as d,
  C as e,
  I as f,
  N as g,
  R as h,
  L as i,
  m as j,
  P as k,
  U as n,
  K as p,
  k as r
};
//# sourceMappingURL=reader-regions-CP3ZMHPc.js.map
