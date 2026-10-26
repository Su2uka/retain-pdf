"""Cross-page and cross-column boundaries must preserve whole translation units."""
import json

import pytest

from continuation_chain_test_support import body, pages
from retainpdf_translation.core.orchestration.units import finalize_orchestration_metadata_by_page
from retainpdf_translation.core.payload.parts.units import pending_translation_items
from retainpdf_translation.services.continuation.orchestrator import review_candidate_continuation_pairs
from retainpdf_translation.services.continuation.pairs import apply_candidate_pair_joins, candidate_continuation_pairs
from retainpdf_translation.services.continuation.rules import pair_decision, starts_like_heading_or_list
from retainpdf_translation.services.continuation.state import annotate_continuation_context


def members(items):
    finalize_orchestration_metadata_by_page(pages(items))
    return [unit["translation_unit_member_ids"] for unit in pending_translation_items(items)]


def review(items, decision="join"):
    seen = []

    def respond(messages, **kwargs):
        pairs = json.loads(messages[-1]["content"])["pairs"]
        seen.extend(pairs)
        return json.dumps({"decisions": [
            {"pair_id": pair["pair_id"], "decision": decision} for pair in pairs
        ]})

    review_candidate_continuation_pairs(
        page_payloads=pages(items), translation_paths={}, api_key="offline",
        model="offline", base_url="https://unused.invalid", workers=1,
        save_pages_fn=lambda *args: None, request_chat_content_fn=respond,
    )
    return seen


@pytest.mark.parametrize("acronym", ["DFT", "TDDFT", "CCSD"])
def test_acronym_body_start_remains_a_review_candidate(acronym):
    items = [
        body("a", 0, "Accurate excitation energies for the optical response were calculated using time-dependent",
             "right_column", "tail"),
        body("b", 1, f"{acronym} calculations with the same basis set for all molecular geometries.",
             "left_column", "head"),
    ]
    assert not starts_like_heading_or_list(items[1]["source_text"])
    annotate_continuation_context(items)
    assert len(candidate_continuation_pairs(items)) == 1
    assert len(review(items)) == 1
    assert members(items) == [["a", "b"]]


@pytest.mark.parametrize("heading", ["RESULTS AND DISCUSSION", "2.2.1 Results", "• First result"])
def test_heading_and_list_signals_are_retained(heading):
    assert starts_like_heading_or_list(heading)


@pytest.mark.parametrize("approve", [True, False])
def test_candidate_after_joined_column_is_reviewed(approve):
    items = [
        body("a", 0, "We performed a systematic analysis of the", "left_column", "tail"),
        body("b", 0, "electronic properties of the molecular system using time-dependent", "right_column", "single", 1),
        body("c", 1, "Hartree-Fock calculations to obtain accurate excitation energies.", "left_column", "head"),
    ]
    assert [pair_decision(a, b) for a, b in zip(items, items[1:])] == ["join", "candidate"]
    annotate_continuation_context(items)
    pairs = candidate_continuation_pairs(items)
    assert [(p["prev_item_id"], p["next_item_id"]) for p in pairs] == [("b", "c")]
    assert len(review(items, "join" if approve else "break")) == 1
    assert members(items) == ([["a", "b", "c"]] if approve else [["c"], ["a", "b"]])


def test_review_extends_group_on_following_page():
    items = [
        body("a", 0, "We investigated the electronic response using time-dependent", "right_column", "tail"),
        body("b", 1, "Hartree-Fock calculations and compared the resulting spectra with", "left_column", "single"),
        body("c", 1, "experimental observations obtained under identical conditions.", "right_column", "head", 1),
    ]
    annotate_continuation_context(items)
    assert members(items) == [["a"], ["b", "c"]]
    assert len(review(items)) == 1
    assert members(items) == [["a", "b", "c"]]


@pytest.mark.parametrize("review_required", [False, True])
def test_provider_group_can_continue_onto_next_page(review_required):
    items = [
        body("a", 0, "We performed a systematic analysis of the", "left_column", "tail",
             ocr_source="provider", ocr_group_id="provider-p0", ocr_scope="intra_page", ocr_order=0),
        body("b", 0, "electronic properties using " + ("time-dependent" if review_required else "the"),
             "right_column", "single", 1,
             ocr_source="provider", ocr_group_id="provider-p0", ocr_scope="intra_page", ocr_order=1),
        body("c", 1, "Hartree-Fock calculations to obtain accurate excitation energies." if review_required
             else "experimental observations obtained under identical conditions.", "left_column", "head"),
    ]
    annotate_continuation_context(items)
    if review_required:
        assert len(review(items)) == 1
    assert members(items) == [["a", "b", "c"]]


@pytest.mark.parametrize("reverse", [False, True])
def test_review_unions_existing_groups_without_collisions_or_lost_external_candidates(reverse):
    items = [body(name, i, f"Source fragment {name}", "left_column", "single") for i, name in enumerate("abcdefg")]
    for item in items[:2]:
        item["continuation_group"] = "left"
    for item in items[2:4]:
        item["continuation_group"] = "right"
    for item in items[5:]:
        item["continuation_group"] = "cg-review-1001"
    items[4]["continuation_candidate_next_id"] = "f"
    items[5]["continuation_candidate_prev_id"] = "e"
    pairs = [("b", "c"), ("d", "e")]
    assert apply_candidate_pair_joins(items, pairs[::-1] if reverse else pairs) == 5
    assert members(items) == [list("abcde"), list("fg")]
    assert items[4]["continuation_candidate_next_id"] == "f"
    assert items[5]["continuation_candidate_prev_id"] == "e"
    before = [dict(item) for item in items]
    assert apply_candidate_pair_joins(items, pairs) == 0
    assert items == before
