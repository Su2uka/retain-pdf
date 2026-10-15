# 开发记录

每次开发一行，按月一个文件：`2026-10.csv`、`2026-11.csv`……

这里只记 git 里没有的东西：**为什么做、怎么验证的、合没合、推没推、还欠什么**。
改了哪些文件、每个提交的说明，看 git；长的方案和决定写进 `docs/ops/planning/<主题>/YYYYMMDD-主题.md`，
这里在 `doc` 列放链接。

## 什么时候写

- 分支合并进 `main` 之前，在这个分支上加一行，状态写 `merged`。这一行和代码一起合进去。
- 推送以后，把那一行的状态改成 `pushed`。这个小改动不用单独推送，跟下一次的记录一起提交。
- 做到一半停下或放弃的，也加一行，状态写 `in_progress` 或 `abandoned`，`follow_up` 写停在哪里。
- 两个分支同时加行，合并时冲突了：两行都留下，按日期排。

## 列

所有格子都用双引号包起来（和 `docs/ops/planning/*.csv` 一样），格子里不换行。

| 列 | 写什么 |
|---|---|
| `date` | 合并（或停下）那天，`YYYY-MM-DD`，要和文件名的月份一致 |
| `branch` | 分支名 |
| `commits` | 分支上的提交号，用 `; ` 隔开；加这一行的那个提交自己不用写，所以只有一个提交的分支这里是空的 |
| `area` | 改到哪块，取值见下表；几块用 `; ` 隔开 |
| `type` | `feat` `fix` `refactor` `test` `docs` `chore` `perf` 之一，和提交前缀一致 |
| `summary` | 一句话：做了什么 |
| `reason` | 一句话：为什么做（用户怎么说的、哪里坏了） |
| `validation` | 跑了哪些检查、结果；没跑的写明 |
| `status` | `in_progress` `merged` `pushed` `abandoned` 之一 |
| `follow_up` | 还欠什么，没有就留空 |
| `doc` | 相关文档，相对 `docs/` 的路径，没有就留空；`merged`、`pushed` 的行要求文件存在 |

`area` 的取值：`api`（Rust API）、`jobs`（任务运行）、`db`、`pipeline`（Python 流水线）、`ai`（AI 服务）、
`cli`、`contracts`、`frontend`、`ops`（开发、部署、CI）、`docs`、`security`。

## 检查

```bash
python3 ops/development/check_devlog.py          # 列数、日期、取值、doc 路径是否存在
python3 ops/development/check_devlog.py --git    # 另外核对提交号在本地 git 里找得到
```

CI 是浅克隆，找不到老提交，所以 `--git` 只在本地跑。
