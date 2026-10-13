"""模型把「草稿 + Wait — … Correct it. + 改好的译文」一起交回来时，校验要拦下来重译。

两个正例都是真实输出（qwen3.8-flash 翻一本量子化学教材）。"""

import pytest

from retainpdf_pipeline.translate.llm.result_validator import validate_batch_result
from retainpdf_pipeline.translate.llm.validation.errors import TranslationProtocolError
from retainpdf_pipeline.translate.llm.validation.self_correction import self_correction_leak
from retainpdf_pipeline.translate.services.quality import review_translation_item

ALLENE_SOURCE = "FIGURE 12.16 Two views of allene. The C=C=C axis is perpendicular to the plane of the paper in (a)."
ALLENE_OUTPUT = (
    "图12.16 烯丙烷（allene）的两种视图。图(a)中 C=C=C 轴垂直于纸面。\n\n"
    'Wait—"allene" should not be translated as 烯丙烷; allene is 丙二烯. Correct it.\n\n'
    "图12.16 丙二烯（allene）的两种视图。图(a)中 $\\mathrm{C=C=C}$ 轴垂直于纸面。"
)
REPEAT_SOURCE = "Thus we have the following correspondence:"
REPEAT_OUTPUT = (
    "Thus we have the following correspondence:\n\n"
    "Wait — I need to correct my reply: the correct output is the translation, not the source repeated. \n\n"
    "因此我们有如下对应关系："
)


@pytest.mark.parametrize(
    ("source", "translated", "expected"),
    [
        (ALLENE_SOURCE, ALLENE_OUTPUT, "Wait—"),
        (REPEAT_SOURCE, REPEAT_OUTPUT, "Wait — I need to correct my reply"),
        ("The apparatus is shown below.", "装置如下图所示。\n\n等等，上面的译文有误，应改为：装置见下图。", "等等，上面的译文有误"),
        ("Allene is shown.", "图中是丙二烯。注意：allene 不应译为烯丙烷。", "不应译为烯丙烷"),
    ],
)
def test_self_correction_is_detected(source, translated, expected):
    assert expected in self_correction_leak(source, translated)


@pytest.mark.parametrize(
    ("source", "translated"),
    [
        # 原文本身就有 Wait（对话、引文）：照译出来的不算
        ('"Wait — I need to correct my reply," she said.', '她说："Wait — I need to correct my reply."'),
        # 「等等」的正常用法
        ("Fruits such as apples, bananas, and so on.", "水果，如苹果、香蕉等等。"),
        ("Wait, he said, and listen.", "等等，他说，听我讲。"),
        # 正常的中文译文里提到「翻译」
        ("The paper was translated into Chinese.", "这篇论文被翻译成了中文。"),
        # 纯英文（没翻译）不归这里管，交给 english_residue
        ("Wait — correct it.", "Wait — correct it."),
    ],
)
def test_ordinary_translations_pass(source, translated):
    assert self_correction_leak(source, translated) == ""


def _caption_item(item_id: str, source: str) -> dict:
    return {
        "item_id": item_id,
        "block_type": "text",
        "metadata": {"structure_role": "caption"},
        "translation_unit_protected_source_text": source,
    }


def test_quality_review_flags_it_as_an_error():
    report = review_translation_item(
        _caption_item("p014-b008", ALLENE_SOURCE),
        {"decision": "translate", "translated_text": ALLENE_OUTPUT},
    )
    kinds = {(issue.kind, issue.severity) for issue in report.issues}
    assert ("self_correction_leak", "error") in kinds


def test_validation_raises_so_the_item_is_retranslated():
    item = _caption_item("p015-b022", REPEAT_SOURCE)
    with pytest.raises(TranslationProtocolError):
        validate_batch_result([item], {"p015-b022": {"decision": "translate", "translated_text": REPEAT_OUTPUT}})
