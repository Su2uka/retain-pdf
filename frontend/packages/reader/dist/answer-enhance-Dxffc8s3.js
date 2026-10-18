let k = 0, b = null, y = null, C = !1;
function v(t = Date.now()) {
  return t < k;
}
function T(t = 700) {
  const o = Date.now() + Math.max(0, t);
  o > k && (k = o);
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
  const o = (e) => {
    var r;
    e.preventDefault(), e.stopPropagation(), (r = e.stopImmediatePropagation) == null || r.call(e);
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
    t.addEventListener(e, o, { capture: !0, passive: !1 });
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
function q(t = 700, o = {}) {
  if (T(t), typeof document > "u") return;
  b == null || b(), b = null;
  const e = Date.now() + Math.max(0, t), r = Math.max(0, Number(o.overlayDelayMs) || 0);
  let n = null;
  r === 0 ? P() : n = setTimeout(() => {
    n = null, Date.now() < e && P();
  }, r);
  const i = (a) => {
    var d;
    if (Date.now() >= e) {
      s();
      return;
    }
    const l = a.target;
    l instanceof Element && l.closest(
      "[data-reader-ai-sessions], [data-reader-ai-actions], [data-reader-ai-composer], .aui-composer, input, textarea, select, [contenteditable='true']"
    ) || (a.preventDefault(), a.stopPropagation(), (d = a.stopImmediatePropagation) == null || d.call(a));
  }, c = { capture: !0, passive: !1 }, u = ["click", "auxclick", "dblclick", "pointerup", "mouseup"], s = () => {
    n != null && (clearTimeout(n), n = null);
    for (const a of u)
      document.removeEventListener(a, i, c);
    _(), b === s && (b = null);
  };
  for (const a of u)
    document.addEventListener(a, i, c);
  b = s, window.setTimeout(s, Math.max(0, t) + 48);
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
  window.open = ((e, r, n) => v() ? null : t(e, r, n));
  const o = (e) => {
    if (!v()) return;
    const r = e.target;
    if (!(r instanceof Element) || r.closest("[data-reader-ai-sessions]")) return;
    r.closest("a[href]") && (e.preventDefault(), e.stopPropagation());
  };
  return document.addEventListener("click", o, !0), () => {
    window.open = t, document.removeEventListener("click", o, !0), C = !1, j();
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
  const o = t.page_idx;
  if (o != null && `${o}`.trim() !== "") {
    const n = Number(o);
    if (Number.isFinite(n) && n >= 0) return Math.floor(n);
  }
  const e = t.page;
  if (e != null && `${e}`.trim() !== "") {
    const n = Number(e);
    if (Number.isFinite(n) && n >= 1) return Math.floor(n) - 1;
  }
  const r = `${t.block_id || ""}`.match(/(?:^|[^0-9])p0*([1-9]\d*)(?:-|_|\b)/i);
  if (r) {
    const n = Number(r[1]);
    if (Number.isFinite(n) && n >= 1) return n - 1;
  }
  return null;
}
function I(t) {
  const o = A(t);
  return o === null ? null : o + 1;
}
function K(t) {
  if (!Array.isArray(t)) return [];
  const o = [];
  for (const e of t) {
    if (!e || typeof e != "object") continue;
    const r = e, n = `${r.block_id || ""}`.trim();
    if (!n) continue;
    const i = A(r);
    o.push({
      ...r,
      block_id: n,
      ref: r.ref,
      page_idx: i === null ? void 0 : i,
      job_id: `${r.job_id || ""}`.trim(),
      document_id: `${r.document_id || ""}`.trim(),
      snippet: `${r.snippet || ""}`.trim()
    });
  }
  return o;
}
function x(t = "", o = 72) {
  const e = `${t}`.replace(/\s+/g, " ").trim();
  return e.length <= o ? e : `${e.slice(0, o).trim()}…`;
}
function S(t, o, { max: e = 5 } = {}) {
  const r = o.filter(M).map((a) => ({
    ...a,
    page_idx: A(a) ?? a.page_idx
  }));
  if (!r.length) return [];
  const n = /* @__PURE__ */ new Map();
  for (const a of r)
    n.set(`${a.ref}`, a);
  const i = [], c = /* @__PURE__ */ new Set();
  for (const a of `${t || ""}`.matchAll(/\[(\d+)\]/g)) {
    const l = a[1];
    c.has(l) || n.has(l) && (c.add(l), i.push(l));
  }
  if (i.length)
    return i.slice(0, e).map((a) => n.get(a));
  if (/\[\d+\]/.test(`${t || ""}`)) return [];
  const u = [], s = /* @__PURE__ */ new Set();
  for (const a of r) {
    const l = A(a);
    if (l !== null) {
      if (s.has(l)) continue;
      s.add(l);
    }
    if (u.push(a), u.length >= Math.min(3, e)) break;
  }
  return u;
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
function Q(t, o) {
  if (!o.size || !t) return t;
  const e = (r) => r.split(W).map((n, i) => i % 2 === 1 ? n : n.replace(new RegExp("(?<!!)\\[(\\d+)\\](?!\\s*\\()", "g"), (c, u) => o.has(u) ? `[${u}](#retainpdf-citation-${u})` : c)).join("");
  return t.split(/(```[\s\S]*?(?:```|$)|~~~[\s\S]*?(?:~~~|$))/g).map((r, n) => n % 2 === 1 ? r : e(r)).join("");
}
function V(t, o, e = "translated", r = {}) {
  const n = r.resolveResourceUrl ?? R, i = `${t || ""}`.trim(), c = `${r.documentId || ""}`.trim(), u = Math.max(1, Math.floor(Number(o) || 0) + 1);
  let s = "";
  if (e === "source" ? c && (s = `/api/v1/documents/${encodeURIComponent(c)}/files/pages/${u}.jpg?width=240`) : i && (s = `/api/v1/jobs/${encodeURIComponent(i)}/files/pages/${u}.jpg?width=240`), !s) return "";
  try {
    return n(s) || s;
  } catch {
    return s;
  }
}
function E(t, o, e = {}) {
  const r = e.resolveResourceUrl ?? R, n = `${t || ""}`.trim();
  let i = `${o || ""}`.trim().replace(/\\/g, "/").replace(/^\.\//, "");
  if (!n || !i || i.startsWith("/") || i.startsWith("//") || /^[a-z][a-z\d+.-]*:/i.test(i))
    return "";
  for (; i.startsWith("images/"); )
    i = i.slice(7);
  const c = [];
  for (const s of i.split("/")) {
    if (!s) continue;
    let a = s;
    try {
      a = decodeURIComponent(s);
    } catch {
      return "";
    }
    if (!a || a === "." || a === ".." || /[\\/]/.test(a)) return "";
    c.push(encodeURIComponent(a));
  }
  if (!c.length || !/^page-\d+$/i.test(decodeURIComponent(c[0]))) return "";
  const u = `/api/v1/jobs/${encodeURIComponent(n)}/files/output/ocr/images/${c.join("/")}`;
  try {
    return r(u) || u;
  } catch {
    return u;
  }
}
function N(t, o, e = {}, r = []) {
  var d;
  const n = `${t || ""}`.trim(), i = `${o || ""}`.trim();
  if (!n || !i || n.startsWith("//")) return "";
  if (/^(?:\.\/)?(?:images\/)?page-\d+\//i.test(n))
    return E(i, n, e);
  let c;
  try {
    c = new URL(n, ((d = globalThis.location) == null ? void 0 : d.href) || "http://localhost/");
  } catch {
    return "";
  }
  const u = c.pathname.match(/^\/api\/v1\/jobs\/([^/]+)\/(?:markdown|files\/output\/ocr)\/images\/(.+)$/i);
  if (!u) return "";
  let s = "";
  try {
    s = decodeURIComponent(u[1]);
  } catch {
    return "";
  }
  if (s === i) return E(i, u[2], e);
  const a = `${c.searchParams.get("doc") || ""}`.trim();
  return !a || !new Set(
    [...r].map((f) => `${f || ""}`.trim()).filter(Boolean)
  ).has(a) ? "" : E(s, u[2], e);
}
function z(t) {
  const o = [
    t.image_url,
    ...Array.isArray(t.image_urls) ? t.image_urls : [],
    ...Array.isArray(t.asset_image_urls) ? t.asset_image_urls : []
  ];
  if (Array.isArray(t.assets))
    for (const n of t.assets)
      n && typeof n == "object" && o.push(n.image_url);
  const e = /* @__PURE__ */ new Set(), r = [];
  for (const n of o) {
    const i = `${n || ""}`.trim();
    !i || e.has(i) || (e.add(i), r.push(i));
  }
  return r;
}
function F(t) {
  var n;
  let o = `${t || ""}`.trim();
  try {
    o = decodeURIComponent(new URL(o, ((n = globalThis.location) == null ? void 0 : n.href) || "http://localhost/").pathname);
  } catch {
    try {
      o = decodeURIComponent(o);
    } catch {
    }
  }
  const e = o.match(/(?:^|\/)page-(\d+)(?:\/|$)/i);
  if (!e) return null;
  const r = Number(e[1]);
  return Number.isFinite(r) && r >= 1 ? Math.floor(r) : null;
}
function O(t = []) {
  const o = /* @__PURE__ */ new Set();
  for (const e of t) {
    const r = `${(e == null ? void 0 : e.document_id) || ""}`.trim();
    r && o.add(r);
  }
  return [...o];
}
function X(t, o, e) {
  const r = O(o), n = N(t, e, {}, r);
  if (!n) return null;
  for (const c of o)
    for (const u of z(c))
      if (N(u, e, {}, r) === n) return c;
  const i = F(n);
  return i == null ? null : o.find((c) => I(c) === i) || null;
}
function Y(t, o, { jobId: e, documentRef: r = t.ownerDocument }) {
  const n = r.createElement("template");
  n.innerHTML = `${o || ""}`;
  let i = 0;
  return n.content.querySelectorAll("img").forEach((c) => {
    var a;
    const u = c.getAttribute("data-ai-src") || c.getAttribute("src") || "", s = N(u, e);
    if (c.removeAttribute("src"), c.removeAttribute("srcset"), !s) {
      const l = r.createElement("span");
      l.className = "aui-image-blocked", l.textContent = (a = c.getAttribute("alt")) != null && a.trim() ? `[图片不可用：${c.getAttribute("alt").trim()}]` : "[图片不可用]", c.replaceWith(l);
      return;
    }
    c.setAttribute("data-ai-src", s), c.setAttribute("loading", "lazy"), c.setAttribute("decoding", "async"), i += 1;
  }), t.replaceChildren(n.content), i;
}
function Z(t, {
  onOpen: o,
  documentRef: e = globalThis.document
} = {}) {
  var i, c;
  if (!t || !e) return;
  const r = [...((i = t.querySelectorAll) == null ? void 0 : i.call(t, "a[href]")) || []];
  for (const u of r) {
    const s = u, a = `${s.getAttribute("href") || ""}`.trim(), l = e.createElement("span");
    for (l.className = `aui-md-extlink${s.className ? ` ${s.className}` : ""}`.trim(); s.firstChild; )
      l.appendChild(s.firstChild);
    if (!((c = l.textContent) != null && c.trim()) && a && (l.textContent = a), a && !a.startsWith("#") && !/^\s*javascript:/i.test(a)) {
      l.dataset.href = a, l.setAttribute("role", "link"), l.tabIndex = 0, l.title = `打开链接：${a}`;
      const d = (f) => {
        f.preventDefault(), f.stopPropagation(), !D(f) && (v() || f instanceof MouseEvent && (f.button !== 0 || f.detail === 0) || o == null || o(a, f));
      };
      l.addEventListener("click", d), l.addEventListener("auxclick", (f) => {
        f.preventDefault(), f.stopPropagation();
      }), l.addEventListener("keydown", (f) => {
        f.key !== "Enter" && f.key !== " " || (f.preventDefault(), d(f));
      });
    } else
      l.removeAttribute("role");
    s.replaceWith(l);
  }
  const n = t;
  n instanceof Element && !n.dataset.auiLinkGuard && (n.dataset.auiLinkGuard = "1", n.addEventListener(
    "click",
    (u) => {
      const s = u.target;
      if (!(s instanceof Element)) return;
      const a = s.closest("a[href]");
      !a || !n.contains(a) || (u.preventDefault(), u.stopPropagation());
    },
    !0
  ));
}
function J(t, o, e, r = globalThis.document) {
  var c, u, s, a;
  if (!o.size || !t) return;
  (c = t.querySelectorAll) == null || c.call(t, "button.reader-ai-citation-ref").forEach((l) => {
    var f;
    const d = l.parentNode;
    d && (d.replaceChild(r.createTextNode(l.textContent || ""), l), (f = d.normalize) == null || f.call(d));
  });
  const n = ((u = r.createTreeWalker) == null ? void 0 : u.call(r, t, 4)) || null, i = [];
  if (n) {
    let l = n.nextNode();
    for (; l; )
      (a = (s = l.parentElement) == null ? void 0 : s.closest) != null && a.call(s, "code, pre, .reader-ai-citation-ref, button, a, .aui-msg-actions") || i.push(l), l = n.nextNode();
  }
  for (const l of i) {
    const d = `${l.textContent || ""}`;
    if (!/\[\d+\]/.test(d)) continue;
    const f = r.createDocumentFragment();
    for (const g of d.split(/(\[\d+\])/)) {
      if (!g) continue;
      const p = g.match(/^\[(\d+)\]$/), m = p ? o.get(p[1]) : null;
      if (m) {
        const h = r.createElement("button");
        h.type = "button", h.className = "reader-ai-citation-ref", h.textContent = g;
        const w = I(m);
        h.title = w ? `跳到第 ${w} 页 · ${x(m.snippet || "", 60)}` : x(m.snippet || "相关片段", 60), w != null && (h.dataset.page = `${w}`), h.addEventListener("click", ($) => {
          $.preventDefault(), $.stopPropagation(), e == null || e(m);
        }), f.appendChild(h);
      } else
        f.appendChild(r.createTextNode(g));
    }
    l.replaceWith(f);
  }
}
function tt(t) {
  var r, n;
  if (!t) return;
  const o = t, e = [
    ...(r = o.matches) != null && r.call(o, "img.is-hydrated") ? [o] : [],
    ...((n = o.querySelectorAll) == null ? void 0 : n.call(o, "img.is-hydrated")) || []
  ];
  for (const i of e) {
    const c = i, u = c.src || "";
    if (u.startsWith("blob:"))
      try {
        URL.revokeObjectURL(u);
      } catch {
      }
    c.classList.remove("is-hydrated"), u.startsWith("blob:") && c.removeAttribute("src");
  }
}
async function et(t, { fetchImpl: o = L, signal: e } = {}) {
  var i, c;
  const r = t, n = [
    ...(i = r.matches) != null && i.call(r, "img[data-ai-src]") ? [r] : [],
    ...((c = r.querySelectorAll) == null ? void 0 : c.call(r, "img[data-ai-src]")) || []
  ];
  await Promise.allSettled(n.map(async (u) => {
    var l;
    const s = u;
    if (e != null && e.aborted || !s.isConnected) return;
    const a = s.getAttribute("data-ai-src") || "";
    if (a)
      try {
        const d = await o(a, e ? { signal: e } : void 0);
        if (e != null && e.aborted || !s.isConnected) {
          try {
            const p = await ((l = d == null ? void 0 : d.blob) == null ? void 0 : l.call(d));
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
        if (e != null && e.aborted || !s.isConnected) {
          try {
            URL.revokeObjectURL(f);
          } catch {
          }
          return;
        }
        const g = s.src || "";
        if (g.startsWith("blob:"))
          try {
            URL.revokeObjectURL(g);
          } catch {
          }
        s.src = f, s.classList.add("is-hydrated"), s.classList.remove("is-missing");
      } catch {
        if (e != null && e.aborted || !s.isConnected) return;
        s.classList.add("is-missing"), s.alt = s.alt || "图片暂不可用";
      }
  }));
}
function rt(t, o, {
  onJump: e = null,
  answerText: r = "",
  max: n = 5,
  documentRef: i = globalThis.document
} = {}) {
  var l;
  (l = t.querySelector(".reader-ai-citations")) == null || l.remove();
  const c = S(r, o, { max: n });
  if (!c.length) return;
  const u = i.createElement("div");
  u.className = "reader-ai-citations", u.setAttribute("aria-label", "引用来源");
  const s = i.createElement("div");
  s.className = "reader-ai-citations-head", s.textContent = "来源", u.appendChild(s);
  const a = i.createElement("div");
  a.className = "reader-ai-citations-list";
  for (const d of c) {
    const f = I(d), g = f != null ? `p.${f}` : "", p = i.createElement("button");
    p.type = "button", p.className = "reader-ai-citation-item", f != null && (p.dataset.page = `${f}`), p.title = f != null ? `跳到第 ${f} 页` : "定位来源", p.addEventListener("click", ($) => {
      $.preventDefault(), $.stopPropagation(), e == null || e(d);
    });
    const m = i.createElement("span");
    m.className = "reader-ai-citation-refno", m.textContent = `[${d.ref ?? "?"}]`;
    const h = i.createElement("span");
    h.className = "reader-ai-citation-meta", h.textContent = g || "—";
    const w = i.createElement("span");
    w.className = "reader-ai-citation-copy", w.textContent = x(d.snippet || "相关片段", 64), p.append(m, h, w), a.appendChild(p);
  }
  u.appendChild(a), t.appendChild(u);
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
//# sourceMappingURL=answer-enhance-Dxffc8s3.js.map
