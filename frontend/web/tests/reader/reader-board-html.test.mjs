/** agent 写的 HTML 在左边打开 —— 以及拦住它的那三道。
 *
 * 为什么是 HTML：agent 手里有 shell，让它直接写浏览器认得的东西，比我们为每种
 * 可视化定 schema + 写渲染器 + 在 AGENTS.md 里教格式划算得多（画布那条路就是
 * 这么死的，三种产物的真实用量加起来是个位数）。
 *
 * 为什么危险：agent 读的是**不受信的 PDF**，一篇论文里埋一句提示注入就能让它
 * 写出任意脚本；而 localStorage 里存着用户的 API Key。所以「agent 写的 HTML
 * 跑在应用自己的源上」必须在结构上不可能，不能靠「它应该不会这么干」。
 *
 * 三道，缺一不可：后端强制附件、iframe 不同源、CSP 断网。这里守后两道
 * （第一道在 ai_board.rs 的 Rust 测试里）。
 */
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { JSDOM } from "jsdom";
import { region } from "../helpers/source-text.mjs";

import {
  BOARD_HTML_CSP,
  BOARD_HTML_SANDBOX,
  withBoardHtmlCsp,
} from "../../src/features/reader/domain/board-html.ts";
import {
  openableBoardItems,
  parseBoardListing,
} from "../../src/features/reader/domain/board.ts";
import { BoardHtmlPane } from "../../src/features/reader/ui/board-html.tsx";
import { hidingRulesFor } from "./helpers/reader-css.mjs";

const read = (p) => readFileSync(fileURLToPath(new URL(p, import.meta.url)), "utf8");
const PANE = read("../../src/features/reader/ui/board-html.tsx");

// ------------------------------------------------------------------ 沙箱

test("sandbox 里绝不能有 allow-same-origin", () => {
  // 它和 allow-scripts 同时出现时，iframe 里的脚本能拿到 parent.document，
  // 把自己的 sandbox 摘掉再重载 —— 沙箱等于没有。这是这个功能最容易犯的错。
  const values = BOARD_HTML_SANDBOX.split(/\s+/).filter(Boolean);
  assert.ok(!values.includes("allow-same-origin"), "沙箱被 allow-same-origin 打穿了");
  // allow-scripts 必须在：不给脚本就只能看静态 HTML，图表库全废。
  assert.deepEqual(values, ["allow-scripts"]);
});

test("iframe 真的带上了那个 sandbox，而且是从同一个常量来的", () => {
  // 只断言常量不够：组件里写死一个别的字符串，常量照样对。
  assert.match(PANE, /sandbox=\{BOARD_HTML_SANDBOX\}/, "iframe 没用那个常量");
  assert.doesNotMatch(PANE, /sandbox="/, "iframe 上写死了 sandbox 字面量");
});

// ------------------------------------------------------------------ CSP

test("CSP 默认什么都不许，出网整个关掉", () => {
  const directives = Object.fromEntries(
    BOARD_HTML_CSP.split(";").map((part) => {
      const [name, ...rest] = part.trim().split(/\s+/);
      return [name, rest];
    }),
  );
  assert.deepEqual(directives["default-src"], ["'none'"], "默认不是 none，取数据就没关住");
  // connect-src 不该被放行 —— default-src 兜住它。
  //
  // 原来这里还断言 form-action「由 default-src 兜住」——**那是错的**，
  // form-action 不在 default-src 的回退链里。表单提交目前是 sandbox 挡的
  // （没给 allow-forms），不是 CSP。
  assert.ok(!("connect-src" in directives), "connect-src 被单独放行了，脚本能把论文内容发出去");
  // 但图表要画得出来。
  assert.ok(directives["img-src"], "图片全禁了，图表库画不出东西");
  assert.ok(directives["script-src"]?.includes("'unsafe-inline'"), "内联脚本被禁，图表跑不起来");
});

test("doctype 在最前，CSP 紧随其后且在任何脚本之前", () => {
  const out = withBoardHtmlCsp('<!doctype html><script>fetch("//evil")</script>');
  // CSP 要早于脚本，否则脚本已经跑过了。
  assert.ok(
    out.indexOf("Content-Security-Policy") < out.indexOf("<script>"),
    "CSP 插在了脚本后面，对它无效",
  );
  // doctype 在 meta 之前。**这不是在防 quirks mode** —— 实测 srcdoc 文档永远是
  // 标准模式（见 tests/layout/board-html-sandbox.test.mjs 里那段实测记录）。
  // 守的是顺序本身的稳定：将来若改成 src= 指真实端点，那条路会认 doctype。
  assert.ok(/^<!doctype html>/i.test(out), "开头不是 doctype —— 页面会进 quirks mode");
  assert.ok(
    out.toLowerCase().indexOf("<!doctype") < out.indexOf("<meta"),
    "doctype 在 meta 后面，等于没有",
  );
  assert.ok(out.includes(BOARD_HTML_CSP), "注进去的不是那条 CSP");
});

test("原文一个字节都不改 —— 这不是消毒，是隔离", () => {
  // 想把危险标签删掉是错的路子：消毒永远漏，而隔离是结构性的。
  // 我们只在**前面**加 doctype + CSP，原文原样跟在后面。
  const html = '<!doctype html><p onclick="x">hi</p><script>1</script>';
  assert.ok(withBoardHtmlCsp(html).endsWith(html), "改动了 agent 写的内容");
});

test("注释不能再说「CSP 把出网整个关掉」—— 它管不住导航", () => {
  // `default-src 'none'` 是 fetch 指令族。一行 location.href 照样把内容送出去，
  // 而能管住 iframe 导航的是**父页面**的 frame-src，这个应用还没有文档级 CSP。
  // 这条守的是「别再让下一个人以为已经断网了」。
  const doc = read("../../src/features/reader/domain/board-html.ts");
  // **这里只能做正向断言。** 想写 `doesNotMatch(/CSP 把出网整个关掉/)` 是不行的
  // —— 那份注释里正引用着这句原话来说明它错在哪，负向断言会命中自己的引文
  // （写这条时当场撞上了，和今天在 CSS 注释、在 ICONS 表上踩的是同一个形状：
  // 断言的那个串在别处也出现）。
  assert.match(doc, /管不住导航/, "没写清 CSP 管不到的那一半");
  assert.match(doc, /frame-src/, "没指出该由谁来挡导航");
  assert.match(doc, /form-action`? 不在 default-src 的回退链里/,
    "没纠正「form-action 由 default-src 兜住」那句错话");
});

// ------------------------------------------------------------------ 列表

/** 端点真正返回的形状：`ApiResponse::ok(...)` 包成 {code, message, data}。 */
const envelope = (items) => ({ code: 0, message: "ok", data: { schema: "ai_board_v1", items, skipped: 0 } });

test("解析的是**真实响应信封**，不是裸对象", () => {
  // 第一版这里喂的是裸 `{items:[…]}`，而端点返回的是 {code,message,data:{items}}。
  // 于是 parseBoardListing 读 payload.items 恒为 undefined、列表永远是空的，
  // 而测试因为喂的是我自己臆想的形状所以全绿 —— 测试和代码犯的是同一个错。
  const parsed = parseBoardListing(envelope([{ name: "x.html", kind: "html", modified_ms: 1 }]));
  assert.ok(parsed, "真实信封解析不出来 —— 产物条永远不会出现");
  assert.deepEqual(parsed.map((i) => i.name), ["x.html"]);
});

test("信封的形状和后端保持一致 —— 漂了就是「我明明写进去了却不显示」", () => {
  // 对着 Rust 那边真正的返回语句核，不在这里凭记忆写。
  const rs = read("../../../../backend/api/src/routes/jobs/ai.rs");
  // 两个锚点，不是「往后数 400 个字符」—— 定长窗口会越过函数边界，把邻居的代码
  // 当成被守对象（reader-anchor-and-locks 那条就是这么假的）。
  const fn = region(rs, "pub async fn list_ai_board", /\n(?:\/\/\/|pub |#\[)/, "list_ai_board");
  assert.match(fn, /ApiResponse::ok\(/, "后端换了返回方式，前端的解包要跟着改");
});

test("只列得出 HTML —— 别的类型没有渲染器，列了点不开", () => {
  const items = parseBoardListing(envelope([
    { name: "a.png", kind: "image", modified_ms: 1 },
    { name: "b.html", kind: "html", modified_ms: 2 },
    { name: "c.pdf", kind: "pdf", modified_ms: 3 },
  ]));
  assert.deepEqual(openableBoardItems(items).map((i) => i.name), ["b.html"]);
});

test("html 是认得的 kind —— 否则后端发得出来、前端却过滤掉了", () => {
  const items = parseBoardListing(envelope([{ name: "x.html", kind: "html", modified_ms: 1 }]));
  assert.equal(items.length, 1, "html 被 KINDS 白名单挡掉了");
});

// ------------------------------------------------------------------ 关得掉

test("关闭按钮渲染得出来，而且没有 CSS 把它藏掉", () => {
  // 这块盖住了 PDF，关闭是唯一的退路 —— 底下的模式页签此刻够不到。
  // 和圆钮、面板工具条同一种死法：代码在、DOM 里有、看不见。
  // 渲染真组件（renderToStaticMarkup 不跑 effect，所以不会发请求），从 DOM 里
  // 拿到那个按钮 —— 拿不到就直接红，否则下面「没人藏它」在按钮不存在时永远绿。
  const markup = renderToStaticMarkup(createElement(BoardHtmlPane, {
    jobId: "job-1", name: "chart.html", onClose() {},
    baseUrl: "http://localhost:8000", apiKey: "k",
  }));
  const body = new JSDOM(`<body>${markup}</body>`).window.document.body;
  const button = body.querySelector(".reader-board-close");
  assert.ok(button, "关闭按钮没渲染出来");
  assert.equal(button.textContent.trim(), "关闭");
  assert.deepEqual(hidingRulesFor(button), [], "关闭按钮被 CSS 藏起来了 —— 这块就再也关不掉");
});

test("叠加层不能盖住顶栏和 AI 面板", () => {
  // 根容器是 position:relative 且占满视口。写 inset:0 会把顶栏（下载/关闭/模式
  // 页签）和右边的面板一起盖住 —— 那时既点不了别的产物、也关不掉这一块。
  const css = read("../../src/features/reader/ui/board-html.css");
  const rule = css.slice(css.indexOf(".reader-board-pane {"));
  const body = rule.slice(0, rule.indexOf("}"));
  assert.doesNotMatch(body, /inset:\s*0\s*;/, "叠加层盖住了整个阅读页");
  assert.match(body, /inset:\s*60px 0 0/, "几何没对齐 .reader-react-scroll-shell");
  assert.match(css, /is-assistant-open \.reader-board-pane[\s\S]{0,80}--reader-ai-split-width/,
    "开着助手面板时没让出右半边，产物会盖住 AI 面板");
  // 不透明：透出底下的 PDF 会和图表叠在一起。
  assert.match(body, /background:/, "叠加层没有背景色");
});
