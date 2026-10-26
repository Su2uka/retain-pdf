import pytest

from continuation_chain_test_support import body, pages
from retainpdf_translation.core.context.models import build_item_context
from retainpdf_translation.core.orchestration.units import finalize_orchestration_metadata_by_page
from retainpdf_translation.core.payload.parts.units import pending_translation_items
from retainpdf_translation.llm.shared.prompt_protocols import direct_typst_single_user_prompt
from retainpdf_translation.services.context.windows import annotate_translation_context_windows
from retainpdf_translation.services.continuation.state import annotate_continuation_context


@pytest.mark.parametrize("mode", ["needed", "all", "off"])
def test_group_prompt_retains_only_external_context(mode):
    items = [
        body("prev", 0, "The reference method is the coupled cluster approach.", "left_column", "head"),
        body("a", 0, "Its predictions depend on the", "left_column", "tail", 1),
        body("b", 1, "accuracy of the optimized", "left_column", "single"),
        body("c", 1, "geometries.", "right_column", "head", 1),
        body("next", 1, "This sensitivity explains the deviation from experimental spectra.", "right_column", "tail", 2),
    ]
    for item in items[1:4]:
        item["continuation_group"] = "context-group"
    finalize_orchestration_metadata_by_page(pages(items))
    annotate_translation_context_windows(pages(items), mode=mode)
    group = next(unit for unit in pending_translation_items(items) if unit["translation_unit_kind"] == "group")
    context = build_item_context(group)
    before, after = context.context_before_for_prompt(), context.context_after_for_prompt()
    if mode == "off":
        assert before == after == ""
    else:
        assert before == items[0]["source_text"]
        assert after == items[-1]["source_text"]
        prompt = direct_typst_single_user_prompt(context, mode="fast")
        assert before in prompt and after in prompt
        batch = context.as_batch_payload()
        assert before in batch["context_before"] and after in batch["context_after"]


def test_context_before_keeps_nearest_tail_and_after_keeps_nearest_head():
    items = [
        body("earlier", 0, "Distant context. " * 40, "left_column", "head"),
        body("prev", 0, "Earlier results were discussed in detail. " * 16 + "The reference method is CCSD(T).",
             "right_column", "tail", 1),
        body("current", 1, "With this reference method, the errors are reduced.", "left_column", "head"),
        body("next", 1, "This explains the residual error. " + "Additional observations. " * 40, "left_column", "tail", 1),
    ]
    annotate_translation_context_windows(pages(items))
    context = build_item_context(items[2])
    assert context.context_before_for_prompt().endswith("The reference method is CCSD(T).")
    assert context.context_after_for_prompt().startswith("This explains the residual error.")
    assert len(context.context_before_for_prompt()) <= 360
    assert len(context.context_after_for_prompt()) <= 360


def test_unresolved_acronym_candidate_keeps_context_in_needed_mode():
    items = [
        body("a", 0, "Accurate excitation energies for the optical response were calculated using time-dependent",
             "right_column", "tail"),
        body("b", 1, "DFT calculations with the same basis set for all molecular geometries.", "left_column", "head"),
    ]
    annotate_continuation_context(items)
    annotate_translation_context_windows(pages(items))
    assert build_item_context(items[0]).context_after_for_prompt() == items[1]["source_text"]
    assert build_item_context(items[1]).context_before_for_prompt() == items[0]["source_text"]
