"""开发记录（docs/ops/devlog/*.csv）的格式检查。

仓库里真实的记录必须过检查——CI 跑这个目录，所以写错的行进不了 main。
其余用例守检查脚本自己的规则。
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

SCRIPT = Path(__file__).resolve().parents[1] / "check_devlog.py"
_spec = importlib.util.spec_from_file_location("check_devlog", SCRIPT)
check_devlog = importlib.util.module_from_spec(_spec)
sys.modules["check_devlog"] = check_devlog
_spec.loader.exec_module(check_devlog)

HEADER = ",".join(f'"{column}"' for column in check_devlog.COLUMNS)


def row(**overrides: str) -> str:
    record = {
        "date": "2026-10-15",
        "branch": "fix/x",
        "commits": "abc1234",
        "area": "api",
        "type": "fix",
        "summary": "做了什么",
        "reason": "为什么",
        "validation": "测试全过",
        "status": "merged",
        "follow_up": "",
        "doc": "",
    }
    record.update(overrides)
    return ",".join(f'"{record[column]}"' for column in check_devlog.COLUMNS)


def errors_for(tmp_path: Path, *rows: str, name: str = "2026-10.csv") -> list[str]:
    path = tmp_path / name
    path.write_text("\n".join([HEADER, *rows]) + "\n", encoding="utf-8")
    errors, _ = check_devlog.check_file(path, docs_root=tmp_path)
    return errors


def test_the_real_devlog_passes() -> None:
    assert check_devlog.main([]) == 0


def test_a_good_row_passes(tmp_path: Path) -> None:
    assert errors_for(tmp_path, row(), row(area="api; docs", commits="abc1234; def5678")) == []


@pytest.mark.parametrize(
    ("overrides", "expected"),
    [
        ({"date": "2026/10/15"}, "YYYY-MM-DD"),
        ({"date": "2026-11-01"}, "放错文件"),
        ({"status": "done"}, "status"),
        ({"type": "feature"}, "type"),
        ({"area": "api; backend"}, "area"),
        ({"summary": ""}, "summary 不能空"),
        ({"commits": "HEAD"}, "不像提交号"),
        ({"doc": "ops/nope.md"}, "找不到"),
    ],
)
def test_bad_rows_are_reported(tmp_path: Path, overrides: dict[str, str], expected: str) -> None:
    errors = errors_for(tmp_path, row(**overrides))
    assert any(expected in error for error in errors), errors


def test_unfinished_rows_may_point_at_docs_not_merged_yet(tmp_path: Path) -> None:
    assert errors_for(tmp_path, row(status="in_progress", commits="", doc="ops/nope.md")) == []


def test_rows_must_be_in_date_order(tmp_path: Path) -> None:
    errors = errors_for(tmp_path, row(date="2026-10-15"), row(date="2026-10-14"))
    assert any("按日期排" in error for error in errors), errors


def test_wrong_column_count_and_header(tmp_path: Path) -> None:
    assert any("列" in error for error in errors_for(tmp_path, '"2026-10-15","only"'))
    path = tmp_path / "2026-10.csv"
    path.write_text('"date","branch"\n', encoding="utf-8")
    errors, _ = check_devlog.check_file(path, docs_root=tmp_path)
    assert any("列名" in error for error in errors), errors


def test_file_name_must_be_a_month(tmp_path: Path) -> None:
    assert any("YYYY-MM.csv" in error for error in errors_for(tmp_path, row(), name="october.csv"))
