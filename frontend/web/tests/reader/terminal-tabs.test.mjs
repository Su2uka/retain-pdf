/** 终端可以同时开几条。
 *
 * 起因：后端**一直支持并行**（`build_terminal_launch` 收 `busy_session_ids`，
 * 第二条连接会挑一个没被占用的 fx 会话），卡点一直在前端 —— dock 里只有一个
 * `<FxTerminal>`。于是想一边让它跑长任务、一边问别的，只能等。
 *
 * 这里守的两类东西，性质完全不同：
 * - **标签逻辑**（纯函数，直接跑）
 * - **切标签不能卸载**（源码门禁）。卸载 = 关 WebSocket = 杀 PTY = 那段对话没了。
 *   这条错了不会报错，只是你切回来发现 fx 忘了一切。
 */
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import {
  MAX_TERMINALS,
  activateTab,
  addTab,
  closeTab,
  initialTabs,
  loadTerminalTabs,
  nextTabSeed,
  normalizeTerminalTabs,
  saveTerminalTabs,
  setTabScope,
  tabLabel,
  terminalTabsStorageKey,
} from "../../src/features/reader/domain/terminal-tabs.ts";

const read = (relative) =>
  readFileSync(fileURLToPath(new URL(relative, import.meta.url)), "utf8");

const PANEL = read("../../src/features/reader/ui/terminal.tsx");
const CSS = read("../../src/features/reader/ui/terminal-chrome.css");

let counter = 1;
const nextId = () => `t${++counter}`;
const fresh = () => { counter = 1; return initialTabs("job-1"); };

// ---------------------------------------------------------------- 标签逻辑

test("能开不止一条 —— 这就是原来缺的那件事", () => {
  let state = fresh();
  assert.equal(state.tabs.length, 1);
  state = addTab(state, nextId);
  assert.equal(state.tabs.length, 2);
  assert.equal(state.activeId, state.tabs[1].id, "新开的没有被激活");
  assert.notEqual(state.tabs[0].id, state.tabs[1].id);
});

test("有上限，到顶了原样返回而不是抛", () => {
  // 每条 = 一个 WebSocket + 一个 PTY + 一个 fx 进程（还带一个回环 HTTP 桥）。
  // 不封顶的话点几下就能把机器点满，而失败的样子是「新终端起不来」。
  let state = fresh();
  for (let i = 0; i < MAX_TERMINALS + 3; i += 1) state = addTab(state, nextId);
  assert.equal(state.tabs.length, MAX_TERMINALS);
  // 按钮该是点不动，不是报错。
  assert.doesNotThrow(() => addTab(state, nextId));
});

test("最后一条关不掉", () => {
  // 关掉之后面板里空无一物，而用户想要的多半是「重开一条」。
  const state = fresh();
  assert.equal(closeTab(state, state.activeId).tabs.length, 1);
});

test("关掉当前这条，激活权交给左边那条", () => {
  let state = fresh();
  state = addTab(state, nextId);
  state = addTab(state, nextId);
  const [first, second, third] = state.tabs;
  assert.equal(state.activeId, third.id);
  const after = closeTab(state, third.id);
  assert.equal(after.activeId, second.id, "没退到左边那条");
  // 关掉不是当前的那条，当前的不该变。
  const other = closeTab(after, first.id);
  assert.equal(other.activeId, second.id);
});

test("关一条不存在的 id 不改变任何东西", () => {
  let state = fresh();
  state = addTab(state, nextId);
  assert.deepEqual(closeTab(state, "没有这个"), state);
  assert.deepEqual(activateTab(state, "没有这个"), state);
});

// ---------------------------------------------------------------- 作用域

test("作用域是每条各自的", () => {
  // 一条盯着这本书、另一条盯着整个文件夹 —— 这是这个功能最有用的形态。
  let state = fresh();
  state = addTab(state, nextId);
  const scope = { kind: "collection", collectionId: "c1", name: "化学", documentCount: 3 };
  state = setTabScope(state, state.tabs[1].id, scope);
  assert.equal(state.tabs[0].scope.kind, "job", "改一条把另一条也改了");
  assert.equal(state.tabs[1].scope.kind, "collection");
});

test("新开的一条继承当前的作用域", () => {
  // 多半是想在同一批书上另起一段。
  let state = fresh();
  const scope = { kind: "collection", collectionId: "c1", name: "化学", documentCount: 3 };
  state = setTabScope(state, state.activeId, scope);
  state = addTab(state, nextId);
  assert.equal(state.tabs[1].scope.collectionId, "c1");
});

test("标签上带作用域，不只带序号", () => {
  // 两条长得一模一样而跑在不同的书上，切错了没有任何提示，
  // 得到的答案却是另一批书的。
  assert.equal(tabLabel({ id: "a", scope: { kind: "job", jobId: "j" } }, 0), "1 书");
  assert.equal(
    tabLabel({ id: "b", scope: { kind: "collection", collectionId: "c", name: "化学", documentCount: 3 } }, 1),
    "2 化学",
  );
});

// ---------------------------------------------------------------- 不能卸载

test("非当前的终端留在树上，只是藏起来", () => {
  // 卸载 = 关 WebSocket = 杀 PTY = 那段对话没了。不报错，只是你切回来
  // 发现 fx 忘了一切。
  assert.match(PANEL, /tabs\.tabs\.map\(\(tab\) => \(\s*<TerminalInstance/,
    "按标签条件渲染了 —— 非当前的会被卸载");
  assert.doesNotMatch(PANEL, /active \?\s*<FxTerminal/, "只渲染当前那条了");
  assert.match(PANEL, /data-hidden=\{!active \? "" : undefined\}/);
});

test("藏起来不能用 display:none", () => {
  // xterm 自己量容器尺寸,量到 0 会把终端算成 1 行,切回来是一团乱码。
  const block = CSS.slice(CSS.indexOf(".reader-terminal-instance[data-hidden]"));
  const rule = block.slice(0, block.indexOf("}"));
  assert.doesNotMatch(rule, /display:\s*none/);
  assert.match(rule, /position:\s*absolute/);
  assert.match(rule, /left:\s*-\d+vw/);
  // 藏着也要有确定尺寸,否则 fit() 还是量到 0。
  assert.match(rule, /width:/);
  assert.match(rule, /height:/);
});

test("藏起来的不参与 Tab 键，也不被读屏念", () => {
  assert.match(PANEL, /inert=\{!active \? true : undefined\}/);
  assert.match(PANEL, /aria-hidden=\{!active \? true : undefined\}/);
});

test("作用域条和提示 chip 作用在当前这条上", () => {
  // 作用在别条上的话，你切到 A 改范围，实际改的是 B —— 而两边都不会提示。
  assert.match(PANEL, /scope=\{activeTab\.scope\}/);
  // 物化要几秒：结果写回点击时的那条（作用域条带着 tabId 回调），不是写回物化完那一刻的
  // state.activeId —— 那样在 A 点了、物化中切到 B，改的就成了 B。
  assert.match(PANEL, /tabId=\{activeTab\.id\}/);
  assert.match(PANEL, /setTabScope\(state, tabId, scope\)/);
  assert.doesNotMatch(PANEL, /setTabScope\(state, state\.activeId, scope\)/);
  assert.match(PANEL, /focusTerminal=\{focusActive\}/);
});

// ---------------------------------------------------------------- 存得住

/** 一份可断言的假 storage。不用 jsdom 的 localStorage：这条测的是纯逻辑，
 * 走不到浏览器的地方就别走 —— 挂住的测试在 CI 上表现成超时，而超时会被当成
 * 基础设施抖动重跑。 */
function fakeStorage(seed = {}) {
  const data = new Map(Object.entries(seed));
  return {
    data,
    getItem: (k) => (data.has(k) ? data.get(k) : null),
    setItem: (k, v) => { data.set(k, v); },
  };
}

const COLLECTION = { kind: "collection", collectionId: "c1", name: "化学", documentCount: 3 };

test("摆好的工作台活过刷新 —— 开了几条、每条在哪个作用域", () => {
  // 原来这份 state 只活在 useState 里：刷一次页面全没，回到一条默认的「这本书」。
  // 而摆出「一条盯这本书、一条盯整个文件夹」要点好几下。
  const storage = fakeStorage();
  let state = initialTabs("job-1");
  state = addTab(state, () => "t2");
  state = setTabScope(state, "t2", COLLECTION);
  saveTerminalTabs("job-1", state, storage);

  const back = loadTerminalTabs("job-1", storage);
  assert.equal(back.tabs.length, 2, "开了两条，回来只剩一条");
  assert.equal(back.activeId, "t2");
  assert.equal(back.tabs[0].scope.kind, "job");
  assert.deepEqual(back.tabs[1].scope, COLLECTION, "第二条的作用域没存住");
});

test("没存过 / 读不出来 / 存的是垃圾，都退回一条默认的，不抛", () => {
  assert.deepEqual(loadTerminalTabs("job-1", fakeStorage()), initialTabs("job-1"));
  assert.deepEqual(
    loadTerminalTabs("job-1", fakeStorage({ [terminalTabsStorageKey("job-1")]: "{不是 JSON" })),
    initialTabs("job-1"),
  );
  // 隐私模式下碰 localStorage 会抛 —— 终端面板不该因此整个打不开。
  const hostile = {
    getItem() { throw new Error("SecurityError"); },
    setItem() { throw new Error("SecurityError"); },
  };
  assert.deepEqual(loadTerminalTabs("job-1", hostile), initialTabs("job-1"));
  assert.doesNotThrow(() => saveTerminalTabs("job-1", initialTabs("job-1"), hostile));
  // 没有 sessionKey 就没有键可写，别写成一个全局的 `…:v1:`。
  assert.equal(terminalTabsStorageKey(""), "");
  assert.deepEqual(loadTerminalTabs("", fakeStorage()), initialTabs(""));
});

test("存下来的东西要过一遍校验 —— 上一个版本写的形状会变", () => {
  // 先证「合法的认得出来」，否则下面几条「认不出来」在任何情况下都绿。
  assert.deepEqual(
    normalizeTerminalTabs({ tabs: [{ id: "t1", scope: { kind: "job", jobId: "j" } }], activeId: "t1" }),
    { tabs: [{ id: "t1", scope: { kind: "job", jobId: "j" } }], activeId: "t1" },
  );
  assert.equal(normalizeTerminalTabs(null), null);
  assert.equal(normalizeTerminalTabs({ tabs: "不是数组" }), null);
  assert.equal(normalizeTerminalTabs({ tabs: [] }), null);
  // 缺 collectionId 的文件夹作用域：放进去的表现是终端连到一个不存在的会话，
  // 没有任何报错。
  assert.equal(normalizeTerminalTabs({ tabs: [{ id: "t1", scope: { kind: "collection" } }] }), null);
  // activeId 指向一条不存在的标签 → 回落到第一条，不是留一个指不到的值。
  assert.equal(
    normalizeTerminalTabs({ tabs: [{ id: "t1", scope: { kind: "job", jobId: "j" } }], activeId: "t9" }).activeId,
    "t1",
  );
  // 上限照样管着存下来的东西 —— 否则手改一下 localStorage 就能把机器点满。
  const many = {
    tabs: Array.from({ length: MAX_TERMINALS + 3 }, (_, i) => ({
      id: `t${i + 1}`, scope: { kind: "job", jobId: "j" },
    })),
    activeId: "t1",
  };
  assert.equal(normalizeTerminalTabs(many).tabs.length, MAX_TERMINALS);
});

test("恢复之后再点「+」，新 id 不能和已有的撞", () => {
  // 撞了就是两条终端共用一个 React key / DOM 节点。
  const storage = fakeStorage();
  let state = initialTabs("job-1");
  state = addTab(state, () => "t2");
  state = addTab(state, () => "t3");
  saveTerminalTabs("job-1", state, storage);

  const back = loadTerminalTabs("job-1", storage);
  let seed = nextTabSeed(back);
  assert.equal(seed, 3, "计数器没跳过恢复出来的序号");
  const grown = addTab(back, () => `t${(seed += 1)}`);
  const ids = grown.tabs.map((tab) => tab.id);
  assert.equal(new Set(ids).size, ids.length, `新 id 和已有的撞了: ${ids}`);
});

test("面板真的把这两件事接上了（读回来 + 写回去）", () => {
  // 纯函数全绿而面板压根没调用它们，表现就是「刷新还是丢」。
  assert.match(PANEL, /useState\(\(\) => loadTerminalTabs\(sessionKey\)\)/);
  assert.match(PANEL, /saveTerminalTabs\(sessionKey, tabs\)/);
  assert.match(PANEL, /useRef\(nextTabSeed\(tabs\)\)/);
});
