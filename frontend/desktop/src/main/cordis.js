// Cordis 的唯一入口。代码按原样放在 vendor/cordis（版本、来源和改动见那里的 README），
// 主进程其余地方都从这里取，换版本只动 vendor 目录。
//
// vendor 里是 ESM；Electron 44 自带的 Node 24 支持 require() 一个 ESM 模块（不含顶层 await）。
module.exports = require("./vendor/cordis/cordis.mjs");
