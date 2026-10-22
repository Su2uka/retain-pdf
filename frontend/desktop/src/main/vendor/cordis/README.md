# vendor/cordis

桌面壳主进程用的 [Cordis](https://github.com/cordiverse/cordis) 插件框架，按原样放在这里，不走 npm 依赖。
主进程只经 `src/main/cordis.js` 引用它。为什么这样放、用在哪，见
`docs/ops/planning/desktop-cordis-pilot/20261022-桌面壳主进程用Cordis试点.md`。

## 版本和来源

| 文件 | 来自 | npm integrity | 上游文件 sha256 |
|---|---|---|---|
| `cordis.mjs` | `cordis@4.0.0-rc.12` 的 `lib/index.js` | `sha512-/ZtRivYxr5sffTULPewHuX5kEy5sWi8oADHSpnt0KjAd8uKRIj+kv4fh3lxtutIfVlgwJVFVXWOQp/gYa+kIOQ==` | `cabbe279876480eddd84e344c2fdc8e7e5eb32580a4e1ea24b0e05a46ad0c2ed` |
| `cosmokit.mjs` | `cosmokit@1.8.1` 的 `lib/index.mjs` | `sha512-PDBv4l90xZKrUsZ0vtoycgZpO/j4iFsqJXrAxsyBDsnQRI7ZMJXIjgDJsKNjd5L8jnVnnlrDCdhkFbTncgCVjQ==` | `8c493c6ab7e44ab7608e407e84a79e86dfb6c17b8067cbccfcf29c011285f72a` |
| `LICENSE` | cordiverse/cordis 仓库的 LICENSE（MIT，cosmokit 同一作者同一许可） | | |

## 改动

只有一处：`cordis.mjs` 里 7 个 `from "cosmokit";` 改成 `from "./cosmokit.mjs";`，让它不经 node_modules 就能找到
cosmokit。`src/main/vendor-cordis.test.mjs` 把这处改动还原后对上游 sha256，别的字节变了测试就红。

`cordis` 的另一个依赖 `@standard-schema/spec` 只有类型，运行时不需要。

## 升级

```sh
npm pack cordis@<版本> cosmokit@<版本>
# 解开后：
cp <cosmokit>/package/lib/index.mjs cosmokit.mjs
sed 's#from "cosmokit";#from "./cosmokit.mjs";#' <cordis>/package/lib/index.js > cordis.mjs
```

再把上表和 `vendor-cordis.test.mjs` 里的版本、sha256 一起改掉。Cordis 4 还是候选版，上游 README 写着接口随时可能变，
升级后要跑 `npm test` 并实际启动一次桌面版。
