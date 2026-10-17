import { g as ut, n as lt, o as dt, b as mt, c as ft, p as gt, l as pt, j as $t, h as It, q as yt, i as ht, k as At, t as bt, u as vt, v as St, w as kt, x as wt, y as Et, m as _t, r as Ct, f as Tt, a as Rt, e as jt, d as Mt, s as Nt, z as Lt } from "../answer-enhance-d3OW6M6M.js";
import { r as ue } from "../config-CgaWliJ_.js";
import { C as Ot, M as Ft, h as Ht, n as Dt, a as Bt, b as Pt, s as Kt } from "../config-CgaWliJ_.js";
import { n as le } from "../markdown-payload-kK3ewW_I.js";
import { Marked as de } from "marked";
import { p as me } from "../markdown-math-XkF5urpn.js";
function fe(t = null) {
  return le(t).content.trim();
}
function D(t = "") {
  return `${t}`.replace(/```[\s\S]*?```/g, " ").replace(/!\[[^\]]*]\([^)]+\)/g, " ").replace(/\[[^\]]+]\([^)]+\)/g, " ").replace(/[#>*_`~|[\]()]/g, " ").replace(/\s+/g, " ").trim();
}
function ge(t = "") {
  const r = D(t).toLowerCase(), e = r.match(/[a-z0-9][a-z0-9-]{1,}/g) || [], n = r.match(/[\u4e00-\u9fff]{2,}/g) || [];
  return [.../* @__PURE__ */ new Set([...e, ...n])].slice(0, 40);
}
function pe(t = "") {
  const r = [];
  let e = "文档开头", n = [];
  for (const i of `${t}`.split(/\r?\n/)) {
    const s = i.match(/^(#{1,4})\s+(.+?)\s*$/);
    s && n.join(`
`).trim() && (r.push({
      title: e,
      text: n.join(`
`).trim()
    }), n = []), s && (e = s[2].trim()), n.push(i);
  }
  return n.join(`
`).trim() && r.push({
    title: e,
    text: n.join(`
`).trim()
  }), r;
}
function $e(t, r) {
  const e = D(`${t.title}
${t.text}`).toLowerCase();
  return r.reduce((n, i) => n + (e.includes(i) ? 1 : 0), 0);
}
function Ie(t = "", r = 420) {
  const e = D(t);
  return e.length <= r ? e : `${e.slice(0, r).trim()}...`;
}
function ye(t, r) {
  return r.length ? [
    "我先基于当前 Markdown 找到这些相关片段：",
    ...r.map((n, i) => `${i + 1}. ${n.title}：${Ie(n.text)}`),
    "",
    `问题：${t}`
  ].join(`
`) : "我没有在当前 Markdown 里找到足够相关的片段。可以换一个更具体的问题，或确认这个任务已经生成 Markdown。";
}
function Qe({
  loadMarkdownPayload: t,
  maxSections: r = 3
} = {}) {
  let e = null, n = "";
  async function i(o) {
    return n || (e = await (t == null ? void 0 : t(o)), n = fe(e), n);
  }
  async function s({ jobId: o = "", question: c = "", scope: m = "document", context: g = null } = {}) {
    const I = await i(o);
    if (!I)
      throw new Error("当前任务还没有可用于问答的 Markdown。");
    const v = ge(`${c} ${g != null && g.page ? `第 ${g.page} 页` : ""}`), S = pe(I).map((l) => ({
      ...l,
      score: $e(l, v)
    })).sort((l, p) => p.score - l.score).filter((l, p) => l.score > 0 || p < r).slice(0, r);
    return {
      answer: ye(c, S),
      citations: S.map((l) => l.title),
      scope: m
    };
  }
  return {
    answer: s,
    ensureLoaded: i
  };
}
const x = "CITE_", ee = "";
function B(t) {
  return `${t}`.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;").replaceAll("'", "&#39;");
}
function he(t) {
  const r = [];
  return { text: `${t ?? ""}`.replace(/\[(\d+)\]/g, (n, i) => {
    const s = `${x}${r.length}${ee}`;
    return r.push(i), s;
  }), refs: r };
}
function Ae(t, r) {
  return r.length ? `${t ?? ""}`.replace(
    new RegExp(`${x}(\\d+)${ee}`, "g"),
    (e, n) => {
      const i = r[Number(n)];
      return i != null ? `[${i}]` : "";
    }
  ) : t;
}
function We(t) {
  return B(t || "").replace(/`([^`\n]+)`/g, "<code>$1</code>").replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>").replace(/\n/g, "<br />");
}
function be(t) {
  if (typeof t == "string") return t;
  const r = t;
  return `${(r == null ? void 0 : r.raw) ?? (r == null ? void 0 : r.text) ?? ""}`;
}
const P = new de();
P.setOptions({ gfm: !0, breaks: !0 });
P.use({
  renderer: {
    html: (t) => B(be(t))
  }
});
const ve = /^\s*(?:javascript|vbscript|data:text\/html)/i;
function Se(t) {
  const r = globalThis.document;
  if (!r)
    return B(t);
  const e = r.createElement("template");
  e.innerHTML = t;
  const n = e.content;
  return n.querySelectorAll("script, iframe, object, embed, base, link, meta, form").forEach((i) => i.remove()), n.querySelectorAll("*").forEach((i) => {
    for (const s of [...i.attributes]) {
      const o = s.name.toLowerCase();
      if (o.startsWith("on") || o === "srcdoc") {
        i.removeAttribute(s.name);
        continue;
      }
      if ((o === "href" || o === "src" || o === "xlink:href") && ve.test(s.value)) {
        i.removeAttribute(s.name);
        continue;
      }
      o === "target" && i.removeAttribute(s.name);
    }
  }), e.innerHTML;
}
const y = /* @__PURE__ */ new Map(), ke = 48;
function Ve(t) {
  const r = `${t || ""}`.trim();
  return r ? y.get(r) ?? null : null;
}
function we(t, r) {
  const e = `${t || ""}`.trim();
  if (e)
    for (y.has(e) && y.delete(e), y.set(e, r); y.size > ke; ) {
      const n = y.keys().next().value;
      if (n == null) break;
      y.delete(n);
    }
}
async function Ye(t) {
  const r = `${t || ""}`;
  if (!r.trim()) return "";
  const e = y.get(r);
  if (e != null) return e;
  const { text: n, refs: i } = he(r), s = await me(n, (c) => {
    const m = String(P.parse(c, { async: !1 }));
    return Se(m);
  }), o = Ae(s, i);
  return we(r, o), o;
}
const Ee = /\[\s*(p\d+[-_]b\d+)\s*\]/gi, _e = new RegExp("(?<![\\w/])(p\\d+[-_]b\\d+)(?![\\w/])", "gi");
function O(t) {
  return `${t || ""}`.trim().toLowerCase().replace(/_/g, "-");
}
const Ce = /```[\s\S]*?(?:```|$)|`[^`\n]+`/g, q = "CODE_", J = "";
function Ze(t, r = []) {
  let e = `${t || ""}`;
  if (!e) return "";
  const n = [];
  e = e.replace(Ce, (s) => {
    const o = `${q}${n.length}${J}`;
    return n.push(s), o;
  });
  const i = /* @__PURE__ */ new Map();
  for (const s of r) {
    const o = O(`${s.block_id || ""}`);
    if (!o) continue;
    const c = `${s.ref ?? ""}`.trim();
    c && i.set(o, c);
  }
  return e = e.replace(Ee, (s, o) => {
    const c = i.get(O(o));
    return c ? `[${c}]` : "";
  }), e = e.replace(_e, (s, o) => {
    const c = i.get(O(o));
    return c ? `[${c}]` : "";
  }), e = e.replace(/\bblock_id\s*[=:：]\s*\S+/gi, ""), e = e.replace(/\bpage_idx\s*[=:：]\s*\d+/gi, ""), e = e.replace(/[ \t]{2,}/g, " "), e = e.replace(/ *\n/g, `
`), e = e.trim(), n.length && (e = e.replace(
    new RegExp(`${q}(\\d+)${J}`, "g"),
    (s, o) => n[Number(o)] ?? ""
  )), e;
}
const R = "retainpdf.reader.ai.conversation.v1:";
function K(t = {}) {
  const r = `${t.jobId || ""}`.trim(), e = `${t.documentId || ""}`.trim();
  return e ? `${R}doc:${e}` : r ? `${R}job:${r}` : `${R}anonymous`;
}
function te(t) {
  const r = `${t.jobId || ""}`.trim();
  return r ? `${R}job:${r}` : "";
}
function X() {
  try {
    return typeof globalThis.localStorage > "u" ? null : globalThis.localStorage;
  } catch {
    return null;
  }
}
function E(t = {}) {
  const r = X();
  if (!r)
    return "";
  try {
    const e = K(t), n = `${r.getItem(e) || ""}`.trim();
    if (n) return n;
    const i = `${t.documentId || ""}`.trim() ? te(t) : "", s = i ? `${r.getItem(i) || ""}`.trim() : "";
    return s && r.setItem(e, s), s;
  } catch {
    return "";
  }
}
function Te(t, r) {
  const e = `${r || ""}`.trim(), n = X();
  if (!(!n || !e))
    try {
      n.setItem(K(t), e);
    } catch {
    }
}
function F(t = {}) {
  const r = X();
  if (r)
    try {
      r.removeItem(K(t));
      const e = `${t.documentId || ""}`.trim() ? te(t) : "";
      e && r.removeItem(e);
    } catch {
    }
}
const Q = "retainpdf.reader.ai.thread-branch.v1:";
function M(t) {
  return typeof t == "string" ? { jobId: `${t || ""}`.trim(), documentId: "" } : {
    jobId: `${(t == null ? void 0 : t.jobId) || ""}`.trim(),
    documentId: `${(t == null ? void 0 : t.documentId) || ""}`.trim()
  };
}
function $(t, r = "") {
  const { jobId: e, documentId: n } = M(t), i = n ? "doc" : "job", s = n || e || "anonymous", o = `${r || ""}`.trim();
  return o ? `${Q}${i}:${s}:conv:${o}` : `${Q}${i}:${s}`;
}
function U() {
  try {
    return typeof globalThis.localStorage > "u" ? null : globalThis.localStorage;
  } catch {
    return null;
  }
}
function j(t) {
  return !!t && typeof t == "object" && !Array.isArray(t);
}
function Re(t) {
  if (!j(t) || typeof t.type != "string") return;
  const r = typeof t.reason == "string" ? t.reason : void 0;
  return r ? { type: t.type, reason: r } : { type: t.type };
}
function je(t) {
  if (!j(t)) return null;
  const r = `${t.id || ""}`.trim(), e = t.role === "user" || t.role === "assistant" ? t.role : null;
  if (!r || !e) return null;
  const n = Array.isArray(t.citations) ? t.citations : void 0, i = typeof t.progress == "string" ? t.progress : void 0;
  let s = Re(t.status);
  return (s == null ? void 0 : s.type) === "running" && (s = { type: "incomplete", reason: "cancelled" }), {
    id: r,
    role: e,
    content: typeof t.content == "string" ? t.content : "",
    ...i ? { progress: i } : {},
    ...n != null && n.length ? { citations: n } : {},
    ...s ? { status: s } : {}
  };
}
function Me(t) {
  var s;
  if (!j(t) || t.version !== 1 || !Array.isArray(t.items))
    return null;
  const r = [];
  for (const o of t.items) {
    if (!j(o)) continue;
    const c = je(o.message);
    if (!c) continue;
    const m = o.parentId === null || o.parentId === void 0 ? null : `${o.parentId}`.trim() || null;
    r.push({ parentId: m, message: c });
  }
  if (!r.length) return null;
  const e = t.headId, n = e == null ? ((s = r[r.length - 1]) == null ? void 0 : s.message.id) ?? null : `${e}`.trim() || null, i = `${t.conversationId || ""}`.trim();
  return { version: 1, headId: n, items: r, ...i ? { conversationId: i } : {} };
}
function H(t, r) {
  if (!t) return null;
  try {
    const e = Me(JSON.parse(t));
    if (!e) return null;
    const n = `${e.conversationId || ""}`.trim();
    return n && r && n !== r ? null : e;
  } catch {
    return null;
  }
}
function re(t, r, e, n) {
  const i = {
    version: 1,
    headId: n.headId,
    items: n.items,
    ...e ? { conversationId: e } : {}
  };
  t.setItem($(r, e), JSON.stringify(i));
}
function W(t, r, e, n, i) {
  try {
    const s = $(r, e);
    re(t, r, e, n), i && i !== s && t.removeItem(i);
  } catch {
  }
}
function xe(t, r = "") {
  const e = U();
  if (!e) return null;
  try {
    const n = M(t), i = e.getItem($(n, r)), s = H(i, r);
    if (s) return s;
    if (n.documentId && n.jobId) {
      const c = { jobId: n.jobId }, m = H(
        e.getItem($(c, r)),
        r
      );
      if (m)
        return W(
          e,
          n,
          r,
          m,
          $(c, r)
        ), m;
    }
    if (!r) return null;
    const o = n.documentId ? [n, ...n.jobId ? [{ jobId: n.jobId }] : []] : [n];
    for (const c of o) {
      const m = H(
        e.getItem($(c)),
        r
      );
      if (!m) continue;
      const g = `${m.conversationId || ""}`.trim(), I = n.documentId ? E({ documentId: n.documentId }) || E({ jobId: n.jobId }) : E({ jobId: n.jobId });
      if (g ? g === r : I === r)
        return n.documentId && W(
          e,
          n,
          r,
          m,
          $(c)
        ), m;
    }
    return null;
  } catch {
    return null;
  }
}
function et(t, r, e = "") {
  const n = U();
  if (!n) return;
  const i = M(t);
  if (!(!i.documentId && !i.jobId || !r.items.length))
    try {
      re(n, i, e, r);
    } catch {
    }
}
function tt(t, r = "") {
  const e = U();
  if (e)
    try {
      const n = M(t);
      e.removeItem($(n, r)), n.documentId && n.jobId && e.removeItem($({ jobId: n.jobId }, r)), r || (e.removeItem($(n)), n.documentId && n.jobId && e.removeItem($({ jobId: n.jobId })));
    } catch {
    }
}
function rt(t) {
  const r = new Map(t.items.map((o) => [o.message.id, o])), e = t.headId && r.get(t.headId) || t.items[t.items.length - 1];
  if (!e) return [];
  const n = [];
  let i = e;
  const s = /* @__PURE__ */ new Set();
  for (; i && !s.has(i.message.id); )
    s.add(i.message.id), n.push(i.message), i = i.parentId ? r.get(i.parentId) : void 0;
  return n.reverse();
}
const V = 600;
function nt(t) {
  const r = `${t || ""}`.replace(/\r\n?/g, `
`).trim();
  return r ? `${(r.length > V ? `${r.slice(0, V).trimEnd()}…（已截断）` : r).split(`
`).map((i) => i.trim() ? `> ${i}` : ">").join(`
`)}

` : "";
}
function it(t, r) {
  const e = `${r || ""}`;
  if (!e) return `${t || ""}`;
  const n = `${t || ""}`.trimStart();
  return n ? `${e}${n}` : e;
}
const Ne = "/api/v1";
function Le() {
  throw new Error("ask not injected (provide ask impl via createReaderAskAnswerer)");
}
function ze() {
  return Promise.resolve(null);
}
const Oe = 240;
function Fe(t = "", r = Oe) {
  const e = `${t}`.replace(/\s+/g, " ").trim();
  return e.length <= r ? e : `${e.slice(0, r).trim()}…`;
}
function He({ question: t = "", scope: r = "document", context: e = null, resolveQuote: n = null } = {}) {
  const i = `${t}`.trim();
  if (!i)
    return "";
  if (r === "selection") {
    const s = typeof n == "function" && e ? n(e) : null, o = Fe((s == null ? void 0 : s.quoteText) || (e == null ? void 0 : e.quoteText) || "");
    if (o) {
      const c = (e == null ? void 0 : e.pane) === "translated" ? "译文" : "原文", m = (e == null ? void 0 : e.kind) === "formula" ? "公式" : (e == null ? void 0 : e.kind) === "table" ? "表格" : (e == null ? void 0 : e.kind) === "figure" ? "图片" : (e == null ? void 0 : e.kind) === "text" ? "文字" : "片段";
      return `（针对选中的${c}${m}：「${o}」）${i}`;
    }
    if (e != null && e.page)
      return `（针对第 ${Number(e.page)} 页的选区内容）${i}`;
  }
  return r === "page" && (e != null && e.page) ? `（当前第 ${Number(e.page)} 页）${i}` : i;
}
function ot({
  jobId: t = "",
  documentId: r = "",
  apiPrefix: e = Ne,
  ask: n = Le,
  documentByJobId: i = ze,
  resolveQuote: s = null,
  // 前端凭据设置里的模型 API Key(与翻译流程同源),按请求随问答一起传给后端
  llmConfig: o = ue
} = {}) {
  const c = `${r || ""}`.trim();
  let m = null, g = E({
    jobId: t,
    documentId: c
  });
  function I() {
    return m || (m = (async () => {
      if (c) return c;
      try {
        const l = await i(e, t);
        return `${(l == null ? void 0 : l.document_id) || ""}`.trim();
      } catch {
        return "";
      }
    })()), m;
  }
  function v(l, p = "") {
    const h = `${l || ""}`.trim();
    h && (g = h, Te({ jobId: t, documentId: p }, h));
  }
  async function S({
    question: l = "",
    scope: p = "document",
    context: h = null,
    onToolEvent: N = null,
    onProgressEvent: u = null,
    onAgentOperationEvent: a = null,
    onAgentConfirmationRequiredEvent: f = null,
    onAgentSessionEvent: d = null,
    onAnswerDelta: A = null,
    onCompress: k = null,
    parentId: ne = "",
    regenerate: ie = !1,
    userMessageId: oe = "",
    assistantMessageId: se = "",
    assistantMode: ae = "reading",
    /** 取消信号：中止 SSE；aborted 后不回写会话粘性（防旧流污染新会话） */
    signal: _ = null
  } = {}) {
    const G = He({ context: h, question: l, resolveQuote: s, scope: p });
    if (!G)
      throw new Error("请输入问题。");
    const L = typeof o == "function" ? o() : o || {}, ce = `${L.apiKey || ""}`.trim(), w = await I();
    if (!w && `${t || ""}`.trim())
      throw new Error("无法关联当前文档，暂不能做整本问答。请确认任务已绑定文档后重试。");
    g || (g = E({ jobId: t, documentId: w }));
    const C = await n({
      question: G,
      documentId: w,
      // document_id is the durable knowledge/operation identity. A job is an
      // immutable pipeline attempt and may be a retry/render child without
      // its own document.v1 or Markdown. Once the document is known, letting
      // the backend resolve its authoritative readable artifacts prevents the
      // Reader from pinning AI to a transient job directory.
      jobId: w ? "" : `${t || ""}`.trim(),
      conversationId: g,
      parentId: `${ne || ""}`.trim(),
      regenerate: !!ie,
      userMessageId: `${oe || ""}`.trim(),
      assistantMessageId: `${se || ""}`.trim(),
      assistantMode: ae,
      onToolEvent: N,
      onProgressEvent: u,
      onAgentOperationEvent: a,
      onAgentConfirmationRequiredEvent: f,
      onAgentSessionEvent: d,
      onAnswerDelta: A,
      onCompress: k,
      llmApiKey: ce,
      llmBaseUrl: `${L.baseUrl || ""}`.trim(),
      llmModel: `${L.model || ""}`.trim(),
      signal: _
    }), z = `${(C == null ? void 0 : C.conversationId) || ""}`.trim();
    return z && !(_ != null && _.aborted) && v(z, w), {
      ...C,
      conversationId: z || g,
      scope: p
    };
  }
  return {
    answer: S,
    getConversationId: () => g,
    setConversationId: (l, p = "") => {
      v(l, p);
    },
    clearConversationId: (l = "") => {
      g = "", F({ jobId: t, documentId: l }), l && F({ documentId: l }), F({ jobId: t });
    },
    getDocumentId: () => I(),
    ensureLoaded: async () => !!await I()
  };
}
const De = 20, Y = 18;
function Be(t = {}) {
  const e = (Array.isArray(t == null ? void 0 : t.messages) ? t.messages : []).find(
    (i) => (i == null ? void 0 : i.role) === "user" && `${(i == null ? void 0 : i.text) || ""}`.trim()
  ), n = `${(e == null ? void 0 : e.text) || (t == null ? void 0 : t.title) || ""}`.replace(/\s+/g, " ").trim();
  return n ? n.length > Y ? `${n.slice(0, Y).trim()}…` : n : "新对话";
}
function Z({
  sessions: t = [],
  activeId: r = ""
} = {}) {
  return (Array.isArray(t) ? t : []).map((e) => ({
    id: `${(e == null ? void 0 : e.id) || ""}`,
    title: Be(e),
    updatedAt: Number(e == null ? void 0 : e.updatedAt) || 0,
    messageCount: Array.isArray(e == null ? void 0 : e.messages) ? e.messages.length : 0,
    active: `${(e == null ? void 0 : e.id) || ""}` == `${r}`
  })).filter((e) => e.id).sort((e, n) => n.updatedAt - e.updatedAt);
}
function Pe({ sessions: t = [], activeId: r = "" } = {}, e = De) {
  const n = Array.isArray(t) ? [...t] : [];
  if (n.length <= e)
    return n;
  const s = n.sort(
    (o, c) => (Number(c == null ? void 0 : c.updatedAt) || 0) - (Number(o == null ? void 0 : o.updatedAt) || 0)
  ).slice(0, e);
  if (r && !s.some((o) => `${o == null ? void 0 : o.id}` == `${r}`)) {
    const o = n.find((c) => `${c == null ? void 0 : c.id}` == `${r}`);
    o && (s[s.length - 1] = o);
  }
  return s;
}
const Ke = "retainpdf-ai-chat-v1:";
function Xe(t) {
  return `${Ke}${`${t || ""}`.trim()}`;
}
function b() {
  try {
    return Date.now();
  } catch {
    return 0;
  }
}
function T(t, r) {
  return { id: t, title: "", createdAt: r, updatedAt: r, messages: [], history: [] };
}
function st({
  jobId: t = "",
  storage: r = globalThis.localStorage || null
} = {}) {
  const e = Xe(t), n = !!(`${t || ""}`.trim() && r);
  let i = 0;
  function s() {
    return i += 1, `s-${b().toString(36)}-${i}`;
  }
  function o() {
    var f;
    const u = { activeId: "", sessions: [] };
    if (!n)
      return u;
    let a = null;
    try {
      const d = r.getItem(e);
      a = d ? JSON.parse(d) : null;
    } catch {
      return u;
    }
    if (!a || typeof a != "object")
      return u;
    if (Array.isArray(a.sessions)) {
      const d = a.sessions.filter((k) => k && `${k.id || ""}`.trim());
      return { activeId: d.some((k) => `${k.id}` == `${a.activeId}`) ? `${a.activeId}` : `${((f = d[0]) == null ? void 0 : f.id) || ""}`, sessions: d };
    }
    if (Array.isArray(a.messages) || Array.isArray(a.history)) {
      const d = b(), A = {
        ...T(s(), d),
        messages: Array.isArray(a.messages) ? a.messages : [],
        history: Array.isArray(a.history) ? a.history : []
      };
      return { activeId: A.id, sessions: [A] };
    }
    return u;
  }
  function c(u) {
    var a;
    if (n)
      try {
        const f = Pe(u), d = f.some((A) => `${A.id}` == `${u.activeId}`) ? u.activeId : `${((a = f[0]) == null ? void 0 : a.id) || ""}`;
        r.setItem(e, JSON.stringify({ v: 2, activeId: d, sessions: f }));
      } catch {
      }
  }
  function m(u) {
    let a = u.sessions.find((f) => `${f.id}` == `${u.activeId}`);
    return a || (a = T(s(), b()), u.sessions.push(a), u.activeId = a.id), a;
  }
  function g() {
    if (!n)
      return { messages: [], history: [] };
    const u = o(), a = u.sessions.find((f) => `${f.id}` == `${u.activeId}`);
    return {
      messages: Array.isArray(a == null ? void 0 : a.messages) ? a.messages : [],
      history: Array.isArray(a == null ? void 0 : a.history) ? a.history : []
    };
  }
  function I({ messages: u = [], history: a = [] } = {}) {
    if (!n)
      return;
    const f = o(), d = m(f);
    d.messages = u.slice(-40), d.history = a.slice(-40), d.updatedAt = b(), c(f);
  }
  function v() {
    if (!n)
      return;
    const u = o(), a = m(u);
    a.messages = [], a.history = [], a.title = "", a.updatedAt = b(), c(u);
  }
  function S() {
    return n ? Z(o()) : [];
  }
  function l() {
    return n ? `${o().activeId || ""}` : "";
  }
  function p() {
    if (!n)
      return "";
    const u = o(), a = T(s(), b());
    return u.sessions.push(a), u.activeId = a.id, c(u), a.id;
  }
  function h(u) {
    if (!n)
      return { messages: [], history: [] };
    const a = o();
    return a.sessions.some((f) => `${f.id}` == `${u}`) && (a.activeId = `${u}`, c(a)), g();
  }
  function N(u) {
    if (!n)
      return { messages: [], history: [] };
    const a = o(), f = `${u || a.activeId}`;
    if (a.sessions = a.sessions.filter((d) => `${d.id}` !== f), `${a.activeId}` === f) {
      const d = Z(a)[0];
      a.activeId = d ? d.id : "";
    }
    if (!a.sessions.length) {
      const d = T(s(), b());
      a.sessions.push(d), a.activeId = d.id;
    }
    return c(a), g();
  }
  return {
    load: g,
    save: I,
    clear: v,
    enabled: n,
    listSessions: S,
    activeSessionId: l,
    newSession: p,
    switchSession: h,
    deleteSession: N
  };
}
export {
  Ot as CREDENTIALS_CHANGED_EVENT,
  V as MAX_QUOTE_CHARS,
  De as MAX_SESSIONS,
  Ft as MISSING_MODEL_API_KEY_MESSAGE,
  ut as answerDocumentIds,
  lt as armReaderAiClickShield,
  dt as buildMarkdownImageApiUrl,
  mt as buildPagePreviewUrl,
  nt as buildQuoteBlock,
  He as buildScopedQuestion,
  ft as clearReaderAiNavigationLock,
  F as clearStoredConversationId,
  tt as clearThreadBranchSnapshot,
  gt as clipSnippet,
  K as conversationStorageKey,
  st as createReaderAiHistoryStore,
  ot as createReaderAskAnswerer,
  Qe as createReaderMarkdownAnswerer,
  pt as decorateCitationMarkdown,
  Be as deriveSessionTitle,
  $t as findCitationForAnswerImage,
  Ht as hasModelApiKey,
  It as hydrateProtectedImages,
  yt as injectCitationMarkers,
  ht as installReaderWindowOpenGuard,
  At as isAgenticCitation,
  bt as isReaderAiNavigationLocked,
  E as loadStoredConversationId,
  xe as loadThreadBranchSnapshot,
  vt as lockReaderAiNavigation,
  it as mergeQuoteIntoDraft,
  St as mountAnswerHtml,
  kt as neutralizeMarkdownAnchors,
  wt as normalizeAiCitations,
  Dt as notifyCredentialsChanged,
  Ve as peekFinalAnswerHtmlCache,
  Et as pickCitationsForAnswer,
  he as protectNumericCitations,
  Bt as readSettingsModelApiKey,
  _t as renderCitationFooter,
  Ye as renderFinalAnswerHtml,
  We as renderStreamingPreviewHtml,
  Ct as resetAnswerEnhanceAdapters,
  Pt as resetReaderAiConfigAdapters,
  Tt as resolveAnswerImageUrl,
  Rt as resolveCitationPageIdx,
  jt as resolveCitationPageNumber,
  ue as resolveReaderAiConfig,
  Ae as restoreNumericCitations,
  Mt as revokeHydratedImageUrls,
  Ze as sanitizeAssistantAnswer,
  Te as saveStoredConversationId,
  et as saveThreadBranchSnapshot,
  Nt as setAnswerEnhanceAdapters,
  Kt as setReaderAiConfigAdapters,
  Lt as shouldIgnoreReaderAiNavEvent,
  Z as summarizeSessions,
  $ as threadBranchStorageKey,
  Pe as trimSessions,
  rt as visiblePathFromSnapshot
};
//# sourceMappingURL=ai.js.map
