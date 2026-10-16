let k = 0, b = null, y = null, C = !1;
function v(t = Date.now()) {
  return t < k;
}
function T(t = 700) {
  const a = Date.now() + Math.max(0, t);
  a > k && (k = a);
}
function j() {
  k = 0, b == null || b(), b = null, _();
}
function P() {
  if (typeof document > "u") return null;
  if (y && y.isConnected) return y;
  const t = document.createElement("div");
  t.setAttribute("data-reader-ai-pointer-shield", "1"), t.setAttribute("aria-hidden", "true"), Object.assign(t.style, {
    position: "fixed",
    inset: "0",
    zIndex: "2147483000",
    cursor: "progress",
    background: "transparent",
    // 只作为切会话期间的状态标记，不再成为全屏命中目标。
    // 真正需要阻止的是引用/链接导航；编辑器、按钮和文本选择必须可用。
    pointerEvents: "none"
  });
  const a = (e) => {
    var n;
    e.preventDefault(), e.stopPropagation(), (n = e.stopImmediatePropagation) == null || n.call(e);
  };
  for (const e of [
    "pointerdown",
    "pointerup",
    "mousedown",
    "mouseup",
    "click",
    "auxclick",
    "dblclick",
    "contextmenu",
    "touchstart",
    "touchend"
  ])
    t.addEventListener(e, a, { capture: !0, passive: !1 });
  return document.documentElement.appendChild(t), y = t, t;
}
function _() {
  if (y) {
    try {
      y.remove();
    } catch {
    }
    y = null;
  }
}
function q(t = 700, a = {}) {
  if (T(t), typeof document > "u") return;
  b == null || b(), b = null;
  const e = Date.now() + Math.max(0, t), n = Math.max(0, Number(a.overlayDelayMs) || 0);
  let r = null;
  n === 0 ? P() : r = setTimeout(() => {
    r = null, Date.now() < e && P();
  }, n);
  const o = (i) => {
    var d;
    if (Date.now() >= e) {
      c();
      return;
    }
    const u = i.target;
    u instanceof Element && u.closest(
      "[data-reader-ai-sessions], [data-reader-ai-actions], [data-reader-ai-composer], .aui-composer, input, textarea, select, [contenteditable='true']"
    ) || (i.preventDefault(), i.stopPropagation(), (d = i.stopImmediatePropagation) == null || d.call(i));
  }, s = { capture: !0, passive: !1 }, l = ["click", "auxclick", "dblclick", "pointerup", "mouseup"], c = () => {
    r != null && (clearTimeout(r), r = null);
    for (const i of l)
      document.removeEventListener(i, o, s);
    _(), b === c && (b = null);
  };
  for (const i of l)
    document.addEventListener(i, o, s);
  b = c, window.setTimeout(c, Math.max(0, t) + 48);
}
function D(t) {
  return v() ? !0 : t ? typeof MouseEvent < "u" && t instanceof MouseEvent && t.isTrusted === !1 : !1;
}
function G() {
  if (typeof window > "u" || typeof window.open != "function")
    return () => {
    };
  if (C) return () => {
  };
  C = !0, j();
  const t = window.open.bind(window);
  window.open = ((e, n, r) => v() ? null : t(e, n, r));
  const a = (e) => {
    if (!v()) return;
    const n = e.target;
    if (!(n instanceof Element) || n.closest("[data-reader-ai-sessions]")) return;
    n.closest("a[href]") && (e.preventDefault(), e.stopPropagation());
  };
  return document.addEventListener("click", a, !0), () => {
    window.open = t, document.removeEventListener("click", a, !0), C = !1, j();
  };
}
let U = (t) => `${t ?? ""}`.trim(), L = globalThis.fetch ?? (async () => {
  throw new Error("fetch not available");
});
function H(t = {}) {
  t.resolveResourceUrl && (U = t.resolveResourceUrl), t.fetchProtected && (L = t.fetchProtected);
}
function B() {
  U = (t) => `${t ?? ""}`.trim(), L = globalThis.fetch ?? (async () => {
    throw new Error("fetch not available");
  });
}
function R(t) {
  try {
    return U(t) || `${t ?? ""}`.trim();
  } catch {
    return `${t ?? ""}`.trim();
  }
}
function M(t) {
  return !!t && typeof t == "object" && `${t.block_id || ""}`.trim() !== "";
}
function A(t) {
  if (!t || typeof t != "object") return null;
  const a = t.page_idx;
  if (a != null && `${a}`.trim() !== "") {
    const r = Number(a);
    if (Number.isFinite(r) && r >= 0) return Math.floor(r);
  }
  const e = t.page;
  if (e != null && `${e}`.trim() !== "") {
    const r = Number(e);
    if (Number.isFinite(r) && r >= 1) return Math.floor(r) - 1;
  }
  const n = `${t.block_id || ""}`.match(/(?:^|[^0-9])p0*([1-9]\d*)(?:-|_|\b)/i);
  if (n) {
    const r = Number(n[1]);
    if (Number.isFinite(r) && r >= 1) return r - 1;
  }
  return null;
}
function I(t) {
  const a = A(t);
  return a === null ? null : a + 1;
}
function K(t) {
  if (!Array.isArray(t)) return [];
  const a = [];
  for (const e of t) {
    if (!e || typeof e != "object") continue;
    const n = e, r = `${n.block_id || ""}`.trim();
    if (!r) continue;
    const o = A(n);
    a.push({
      ...n,
      block_id: r,
      ref: n.ref,
      page_idx: o === null ? void 0 : o,
      job_id: `${n.job_id || ""}`.trim(),
      document_id: `${n.document_id || ""}`.trim(),
      snippet: `${n.snippet || ""}`.trim()
    });
  }
  return a;
}
function x(t = "", a = 72) {
  const e = `${t}`.replace(/\s+/g, " ").trim();
  return e.length <= a ? e : `${e.slice(0, a).trim()}…`;
}
function S(t, a, { max: e = 5 } = {}) {
  const n = a.filter(M).map((i) => ({
    ...i,
    page_idx: A(i) ?? i.page_idx
  }));
  if (!n.length) return [];
  const r = /* @__PURE__ */ new Map();
  for (const i of n)
    r.set(`${i.ref}`, i);
  const o = [], s = /* @__PURE__ */ new Set();
  for (const i of `${t || ""}`.matchAll(/\[(\d+)\]/g)) {
    const u = i[1];
    s.has(u) || r.has(u) && (s.add(u), o.push(u));
  }
  if (o.length)
    return o.slice(0, e).map((i) => r.get(i));
  if (/\[\d+\]/.test(`${t || ""}`)) return [];
  const l = [], c = /* @__PURE__ */ new Set();
  for (const i of n) {
    const u = A(i);
    if (u !== null) {
      if (c.has(u)) continue;
      c.add(u);
    }
    if (l.push(i), l.length >= Math.min(3, e)) break;
  }
  return l;
}
const W = new RegExp(
  "(" + [
    "`+[^`\\n]*?`+",
    // 行内 code
    "\\$\\$[\\s\\S]*?\\$\\$",
    // 块级公式
    "(?<!\\\\)\\$(?:\\\\.|[^$\\\\\\n])+(?<!\\\\)\\$",
    // 行内公式
    "!?\\[(?:[^\\][]|\\[[^\\]]*\\])*\\]\\([^)]*\\)"
    // 图片与链接（含 URL）
  ].join("|") + ")",
  "g"
);
function Q(t, a) {
  if (!a.size || !t) return t;
  const e = (n) => n.split(W).map((r, o) => o % 2 === 1 ? r : r.replace(new RegExp("(?<!!)\\[(\\d+)\\](?!\\s*\\()", "g"), (s, l) => a.has(l) ? `[${l}](#retainpdf-citation-${l})` : s)).join("");
  return t.split(/(```[\s\S]*?(?:```|$)|~~~[\s\S]*?(?:~~~|$))/g).map((n, r) => r % 2 === 1 ? n : e(n)).join("");
}
function V(t, a, e = "translated", n = {}) {
  const r = n.resolveResourceUrl ?? R, o = `${t || ""}`.trim(), s = Math.max(1, Math.floor(Number(a) || 0) + 1);
  if (!o) return "";
  const l = e === "source" ? `/api/v1/jobs/${encodeURIComponent(o)}/preview/pages/${s}?kind=source&width=240` : `/api/v1/jobs/${encodeURIComponent(o)}/files/pages/${s}.jpg?width=240`;
  try {
    return r(l) || l;
  } catch {
    return l;
  }
}
function E(t, a, e = {}) {
  const n = e.resolveResourceUrl ?? R, r = `${t || ""}`.trim();
  let o = `${a || ""}`.trim().replace(/\\/g, "/").replace(/^\.\//, "");
  if (!r || !o || o.startsWith("/") || o.startsWith("//") || /^[a-z][a-z\d+.-]*:/i.test(o))
    return "";
  for (; o.startsWith("images/"); )
    o = o.slice(7);
  const s = [];
  for (const c of o.split("/")) {
    if (!c) continue;
    let i = c;
    try {
      i = decodeURIComponent(c);
    } catch {
      return "";
    }
    if (!i || i === "." || i === ".." || /[\\/]/.test(i)) return "";
    s.push(encodeURIComponent(i));
  }
  if (!s.length || !/^page-\d+$/i.test(decodeURIComponent(s[0]))) return "";
  const l = `/api/v1/jobs/${encodeURIComponent(r)}/files/output/ocr/images/${s.join("/")}`;
  try {
    return n(l) || l;
  } catch {
    return l;
  }
}
function N(t, a, e = {}, n = []) {
  var d;
  const r = `${t || ""}`.trim(), o = `${a || ""}`.trim();
  if (!r || !o || r.startsWith("//")) return "";
  if (/^(?:\.\/)?(?:images\/)?page-\d+\//i.test(r))
    return E(o, r, e);
  let s;
  try {
    s = new URL(r, ((d = globalThis.location) == null ? void 0 : d.href) || "http://localhost/");
  } catch {
    return "";
  }
  const l = s.pathname.match(/^\/api\/v1\/jobs\/([^/]+)\/(?:markdown|files\/output\/ocr)\/images\/(.+)$/i);
  if (!l) return "";
  let c = "";
  try {
    c = decodeURIComponent(l[1]);
  } catch {
    return "";
  }
  if (c === o) return E(o, l[2], e);
  const i = `${s.searchParams.get("doc") || ""}`.trim();
  return !i || !new Set(
    [...n].map((f) => `${f || ""}`.trim()).filter(Boolean)
  ).has(i) ? "" : E(c, l[2], e);
}
function z(t) {
  const a = [
    t.image_url,
    ...Array.isArray(t.image_urls) ? t.image_urls : [],
    ...Array.isArray(t.asset_image_urls) ? t.asset_image_urls : []
  ];
  if (Array.isArray(t.assets))
    for (const r of t.assets)
      r && typeof r == "object" && a.push(r.image_url);
  const e = /* @__PURE__ */ new Set(), n = [];
  for (const r of a) {
    const o = `${r || ""}`.trim();
    !o || e.has(o) || (e.add(o), n.push(o));
  }
  return n;
}
function F(t) {
  var r;
  let a = `${t || ""}`.trim();
  try {
    a = decodeURIComponent(new URL(a, ((r = globalThis.location) == null ? void 0 : r.href) || "http://localhost/").pathname);
  } catch {
    try {
      a = decodeURIComponent(a);
    } catch {
    }
  }
  const e = a.match(/(?:^|\/)page-(\d+)(?:\/|$)/i);
  if (!e) return null;
  const n = Number(e[1]);
  return Number.isFinite(n) && n >= 1 ? Math.floor(n) : null;
}
function O(t = []) {
  const a = /* @__PURE__ */ new Set();
  for (const e of t) {
    const n = `${(e == null ? void 0 : e.document_id) || ""}`.trim();
    n && a.add(n);
  }
  return [...a];
}
function X(t, a, e) {
  const n = O(a), r = N(t, e, {}, n);
  if (!r) return null;
  for (const s of a)
    for (const l of z(s))
      if (N(l, e, {}, n) === r) return s;
  const o = F(r);
  return o == null ? null : a.find((s) => I(s) === o) || null;
}
function Y(t, a, { jobId: e, documentRef: n = t.ownerDocument }) {
  const r = n.createElement("template");
  r.innerHTML = `${a || ""}`;
  let o = 0;
  return r.content.querySelectorAll("img").forEach((s) => {
    var i;
    const l = s.getAttribute("data-ai-src") || s.getAttribute("src") || "", c = N(l, e);
    if (s.removeAttribute("src"), s.removeAttribute("srcset"), !c) {
      const u = n.createElement("span");
      u.className = "aui-image-blocked", u.textContent = (i = s.getAttribute("alt")) != null && i.trim() ? `[图片不可用：${s.getAttribute("alt").trim()}]` : "[图片不可用]", s.replaceWith(u);
      return;
    }
    s.setAttribute("data-ai-src", c), s.setAttribute("loading", "lazy"), s.setAttribute("decoding", "async"), o += 1;
  }), t.replaceChildren(r.content), o;
}
function Z(t, {
  onOpen: a,
  documentRef: e = globalThis.document
} = {}) {
  var o, s;
  if (!t || !e) return;
  const n = [...((o = t.querySelectorAll) == null ? void 0 : o.call(t, "a[href]")) || []];
  for (const l of n) {
    const c = l, i = `${c.getAttribute("href") || ""}`.trim(), u = e.createElement("span");
    for (u.className = `aui-md-extlink${c.className ? ` ${c.className}` : ""}`.trim(); c.firstChild; )
      u.appendChild(c.firstChild);
    if (!((s = u.textContent) != null && s.trim()) && i && (u.textContent = i), i && !i.startsWith("#") && !/^\s*javascript:/i.test(i)) {
      u.dataset.href = i, u.setAttribute("role", "link"), u.tabIndex = 0, u.title = `打开链接：${i}`;
      const d = (f) => {
        f.preventDefault(), f.stopPropagation(), !D(f) && (v() || f instanceof MouseEvent && (f.button !== 0 || f.detail === 0) || a == null || a(i, f));
      };
      u.addEventListener("click", d), u.addEventListener("auxclick", (f) => {
        f.preventDefault(), f.stopPropagation();
      }), u.addEventListener("keydown", (f) => {
        f.key !== "Enter" && f.key !== " " || (f.preventDefault(), d(f));
      });
    } else
      u.removeAttribute("role");
    c.replaceWith(u);
  }
  const r = t;
  r instanceof Element && !r.dataset.auiLinkGuard && (r.dataset.auiLinkGuard = "1", r.addEventListener(
    "click",
    (l) => {
      const c = l.target;
      if (!(c instanceof Element)) return;
      const i = c.closest("a[href]");
      !i || !r.contains(i) || (l.preventDefault(), l.stopPropagation());
    },
    !0
  ));
}
function J(t, a, e, n = globalThis.document) {
  var s, l, c, i;
  if (!a.size || !t) return;
  (s = t.querySelectorAll) == null || s.call(t, "button.reader-ai-citation-ref").forEach((u) => {
    var f;
    const d = u.parentNode;
    d && (d.replaceChild(n.createTextNode(u.textContent || ""), u), (f = d.normalize) == null || f.call(d));
  });
  const r = ((l = n.createTreeWalker) == null ? void 0 : l.call(n, t, 4)) || null, o = [];
  if (r) {
    let u = r.nextNode();
    for (; u; )
      (i = (c = u.parentElement) == null ? void 0 : c.closest) != null && i.call(c, "code, pre, .reader-ai-citation-ref, button, a, .aui-msg-actions") || o.push(u), u = r.nextNode();
  }
  for (const u of o) {
    const d = `${u.textContent || ""}`;
    if (!/\[\d+\]/.test(d)) continue;
    const f = n.createDocumentFragment();
    for (const g of d.split(/(\[\d+\])/)) {
      if (!g) continue;
      const p = g.match(/^\[(\d+)\]$/), m = p ? a.get(p[1]) : null;
      if (m) {
        const h = n.createElement("button");
        h.type = "button", h.className = "reader-ai-citation-ref", h.textContent = g;
        const w = I(m);
        h.title = w ? `跳到第 ${w} 页 · ${x(m.snippet || "", 60)}` : x(m.snippet || "相关片段", 60), w != null && (h.dataset.page = `${w}`), h.addEventListener("click", ($) => {
          $.preventDefault(), $.stopPropagation(), e == null || e(m);
        }), f.appendChild(h);
      } else
        f.appendChild(n.createTextNode(g));
    }
    u.replaceWith(f);
  }
}
function tt(t) {
  var n, r;
  if (!t) return;
  const a = t, e = [
    ...(n = a.matches) != null && n.call(a, "img.is-hydrated") ? [a] : [],
    ...((r = a.querySelectorAll) == null ? void 0 : r.call(a, "img.is-hydrated")) || []
  ];
  for (const o of e) {
    const s = o, l = s.src || "";
    if (l.startsWith("blob:"))
      try {
        URL.revokeObjectURL(l);
      } catch {
      }
    s.classList.remove("is-hydrated"), l.startsWith("blob:") && s.removeAttribute("src");
  }
}
async function et(t, { fetchImpl: a = L, signal: e } = {}) {
  var o, s;
  const n = t, r = [
    ...(o = n.matches) != null && o.call(n, "img[data-ai-src]") ? [n] : [],
    ...((s = n.querySelectorAll) == null ? void 0 : s.call(n, "img[data-ai-src]")) || []
  ];
  await Promise.allSettled(r.map(async (l) => {
    var u;
    const c = l;
    if (e != null && e.aborted || !c.isConnected) return;
    const i = c.getAttribute("data-ai-src") || "";
    if (i)
      try {
        const d = await a(i, e ? { signal: e } : void 0);
        if (e != null && e.aborted || !c.isConnected) {
          try {
            const p = await ((u = d == null ? void 0 : d.blob) == null ? void 0 : u.call(d));
            if (p) {
              const m = URL.createObjectURL(p);
              URL.revokeObjectURL(m);
            }
          } catch {
          }
          return;
        }
        if (!(d != null && d.ok)) throw new Error(`HTTP ${(d == null ? void 0 : d.status) || 0}`);
        const f = URL.createObjectURL(await d.blob());
        if (e != null && e.aborted || !c.isConnected) {
          try {
            URL.revokeObjectURL(f);
          } catch {
          }
          return;
        }
        const g = c.src || "";
        if (g.startsWith("blob:"))
          try {
            URL.revokeObjectURL(g);
          } catch {
          }
        c.src = f, c.classList.add("is-hydrated"), c.classList.remove("is-missing");
      } catch {
        if (e != null && e.aborted || !c.isConnected) return;
        c.classList.add("is-missing"), c.alt = c.alt || "图片暂不可用";
      }
  }));
}
function rt(t, a, {
  onJump: e = null,
  answerText: n = "",
  max: r = 5,
  documentRef: o = globalThis.document
} = {}) {
  var u;
  (u = t.querySelector(".reader-ai-citations")) == null || u.remove();
  const s = S(n, a, { max: r });
  if (!s.length) return;
  const l = o.createElement("div");
  l.className = "reader-ai-citations", l.setAttribute("aria-label", "引用来源");
  const c = o.createElement("div");
  c.className = "reader-ai-citations-head", c.textContent = "来源", l.appendChild(c);
  const i = o.createElement("div");
  i.className = "reader-ai-citations-list";
  for (const d of s) {
    const f = I(d), g = f != null ? `p.${f}` : "", p = o.createElement("button");
    p.type = "button", p.className = "reader-ai-citation-item", f != null && (p.dataset.page = `${f}`), p.title = f != null ? `跳到第 ${f} 页` : "定位来源", p.addEventListener("click", ($) => {
      $.preventDefault(), $.stopPropagation(), e == null || e(d);
    });
    const m = o.createElement("span");
    m.className = "reader-ai-citation-refno", m.textContent = `[${d.ref ?? "?"}]`;
    const h = o.createElement("span");
    h.className = "reader-ai-citation-meta", h.textContent = g || "—";
    const w = o.createElement("span");
    w.className = "reader-ai-citation-copy", w.textContent = x(d.snippet || "相关片段", 64), p.append(m, h, w), i.appendChild(p);
  }
  l.appendChild(i), t.appendChild(l);
}
export {
  A as a,
  V as b,
  j as c,
  tt as d,
  I as e,
  N as f,
  O as g,
  et as h,
  G as i,
  X as j,
  M as k,
  Q as l,
  rt as m,
  q as n,
  E as o,
  x as p,
  J as q,
  B as r,
  H as s,
  v as t,
  T as u,
  Y as v,
  Z as w,
  K as x,
  S as y,
  D as z
};
//# sourceMappingURL=answer-enhance-d3OW6M6M.js.map
