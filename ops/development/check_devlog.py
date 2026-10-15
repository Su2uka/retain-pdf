#!/usr/bin/env python3
"""检查 docs/ops/devlog/*.csv 的格式，规则见 docs/ops/devlog/README.md。

    python3 ops/development/check_devlog.py          # 列、日期、取值、doc 路径
    python3 ops/development/check_devlog.py --git    # 另外核对提交号在本地 git 里找得到

CI 是浅克隆，找不到老提交，所以 --git 只在本地跑。
"""

from __future__ import annotations

import argparse
import csv
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
DEVLOG_DIR = REPO_ROOT / "docs" / "ops" / "devlog"
DOCS_ROOT = REPO_ROOT / "docs"

COLUMNS = [
    "date",
    "branch",
    "commits",
    "area",
    "type",
    "summary",
    "reason",
    "validation",
    "status",
    "follow_up",
    "doc",
]
REQUIRED = ["date", "branch", "area", "type", "summary", "reason", "validation", "status"]
AREAS = {"api", "jobs", "db", "pipeline", "ai", "cli", "contracts", "frontend", "ops", "docs", "security"}
TYPES = {"feat", "fix", "refactor", "test", "docs", "chore", "perf"}
STATUSES = {"in_progress", "merged", "pushed", "abandoned"}
# 合进 main 的行，doc 指的文件必须已经在仓库里。
LANDED = {"merged", "pushed"}

FILE_NAME = re.compile(r"^(\d{4}-\d{2})\.csv$")
DATE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
COMMIT = re.compile(r"^[0-9a-f]{7,40}$")


def split_list(value: str) -> list[str]:
    return [item.strip() for item in value.split(";") if item.strip()]


def check_file(path: Path, docs_root: Path = DOCS_ROOT) -> tuple[list[str], list[str]]:
    """返回 (错误, 这个文件里出现的提交号)。"""
    errors: list[str] = []
    commits: list[str] = []
    name = path.name
    month_match = FILE_NAME.match(name)
    if not month_match:
        return [f"{name}: 文件名要是 YYYY-MM.csv"], commits
    month = month_match.group(1)

    raw = path.read_text(encoding="utf-8")
    if "\r" in raw:
        errors.append(f"{name}: 换行要用 LF")
    rows = list(csv.reader(raw.splitlines()))
    if not rows or rows[0] != COLUMNS:
        return errors + [f"{name}: 第 1 行的列名要是 {','.join(COLUMNS)}"], commits

    previous_date = ""
    for line_no, row in enumerate(rows[1:], start=2):
        where = f"{name}:{line_no}"
        if len(row) != len(COLUMNS):
            errors.append(f"{where}: 有 {len(row)} 列，应该是 {len(COLUMNS)} 列")
            continue
        record = dict(zip(COLUMNS, (cell.strip() for cell in row)))

        for column in REQUIRED:
            if not record[column]:
                errors.append(f"{where}: {column} 不能空")

        date = record["date"]
        if date:
            if not DATE.match(date):
                errors.append(f"{where}: date 要写成 YYYY-MM-DD，现在是 {date!r}")
            elif not date.startswith(month):
                errors.append(f"{where}: date {date} 不在 {month}，放错文件了")
            elif date < previous_date:
                errors.append(f"{where}: date {date} 比上一行 {previous_date} 早，按日期排")
            else:
                previous_date = date

        for area in split_list(record["area"]):
            if area not in AREAS:
                errors.append(f"{where}: area {area!r} 不认识，可选 {' '.join(sorted(AREAS))}")
        if record["type"] and record["type"] not in TYPES:
            errors.append(f"{where}: type {record['type']!r} 不认识，可选 {' '.join(sorted(TYPES))}")
        status = record["status"]
        if status and status not in STATUSES:
            errors.append(f"{where}: status {status!r} 不认识，可选 {' '.join(sorted(STATUSES))}")

        for commit in split_list(record["commits"]):
            if COMMIT.match(commit):
                commits.append(commit)
            else:
                errors.append(f"{where}: commits 里的 {commit!r} 不像提交号（7–40 位小写十六进制）")

        doc = record["doc"]
        if doc and status in LANDED:
            for item in split_list(doc):
                if not (docs_root / item).is_file():
                    errors.append(f"{where}: doc {item} 在 docs/ 下找不到")
    return errors, commits


def missing_commits(commits: list[str], repo_root: Path = REPO_ROOT) -> list[str]:
    missing = []
    for commit in commits:
        result = subprocess.run(
            ["git", "-C", str(repo_root), "cat-file", "-e", f"{commit}^{{commit}}"],
            capture_output=True,
        )
        if result.returncode != 0:
            missing.append(commit)
    return missing


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--git", action="store_true", help="核对提交号在本地 git 里找得到")
    args = parser.parse_args(argv)

    files = sorted(DEVLOG_DIR.glob("*.csv"))
    errors: list[str] = []
    commits: list[str] = []
    for path in files:
        file_errors, file_commits = check_file(path)
        errors += file_errors
        commits += file_commits
    if args.git:
        errors += [f"提交 {commit} 在本地 git 里找不到" for commit in missing_commits(commits)]

    if errors:
        print("开发记录格式不对（规则见 docs/ops/devlog/README.md）：", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        return 1
    print(f"开发记录：{len(files)} 个文件，格式正确")
    return 0


if __name__ == "__main__":
    sys.exit(main())
