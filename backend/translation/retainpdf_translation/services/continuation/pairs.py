from retainpdf_translation.services.continuation.rules import bbox
from retainpdf_translation.services.continuation.rules import normalize_text


def candidate_continuation_pairs(payload: list[dict]) -> list[dict]:
    item_by_id = {item.get("item_id", ""): item for item in payload}
    pairs: list[dict] = []
    seen: set[tuple[str, str]] = set()
    for item in payload:
        next_id = item.get("continuation_candidate_next_id", "") or ""
        item_id = item.get("item_id", "") or ""
        if not item_id or not next_id:
            continue
        pair_key = (item_id, next_id)
        if pair_key in seen:
            continue
        next_item = item_by_id.get(next_id)
        if not next_item:
            continue
        seen.add(pair_key)
        pairs.append(
            {
                "prev_item_id": item_id,
                "next_item_id": next_id,
                "prev_text": normalize_text(item.get("protected_source_text", "")),
                "next_text": normalize_text(next_item.get("protected_source_text", "")),
                "prev_page_idx": item.get("page_idx", -1),
                "next_page_idx": next_item.get("page_idx", -1),
                "prev_bbox": bbox(item),
                "next_bbox": bbox(next_item),
            }
        )
    return pairs


def merge_continuation_pairs(
    payload: list[dict],
    joined_pairs: list[tuple[str, str]],
    *,
    decision: str,
    group_prefix: str,
) -> int:
    """Union accepted edges with existing groups without losing any members.

    Payload order is the book's reading order, independent of review completion
    order. Only changed components are rewritten; repeated approvals are no-ops.
    """
    if not joined_pairs:
        return 0
    index_by_id = {item["item_id"]: index for index, item in enumerate(payload) if item.get("item_id")}
    parents = list(range(len(payload)))

    def root(index: int) -> int:
        while parents[index] != index:
            parents[index] = parents[parents[index]]
            index = parents[index]
        return index

    def union(left: int, right: int) -> bool:
        left, right = root(left), root(right)
        if left == right:
            return False
        parents[max(left, right)] = min(left, right)
        return True

    group_heads: dict[str, int] = {}
    for index, item in enumerate(payload):
        group_id = str(item.get("continuation_group", "") or "")
        if group_id:
            union(group_heads.setdefault(group_id, index), index)

    changed: set[int] = set()
    for prev_id, next_id in joined_pairs:
        prev_index, next_index = index_by_id.get(prev_id), index_by_id.get(next_id)
        if prev_index is None or next_index is None or prev_index >= next_index:
            continue
        if union(prev_index, next_index):
            changed.add(prev_index)
    changed_roots = {root(index) for index in changed}
    components: dict[int, list[dict]] = {}
    for index, item in enumerate(payload):
        component = root(index)
        if component in changed_roots:
            components.setdefault(component, []).append(item)

    used_group_ids = set(group_heads)
    group_index = 1000
    annotated = 0
    for chain in components.values():
        group_index += 1
        group_id = f"{group_prefix}-{group_index:04d}"
        while group_id in used_group_ids:
            group_index += 1
            group_id = f"{group_prefix}-{group_index:04d}"
        used_group_ids.add(group_id)
        member_ids = {item.get("item_id", "") for item in chain}
        for pos, item in enumerate(chain):
            item["continuation_group"] = group_id
            item["continuation_decision"] = decision
            # Unresolved edges outside this component must survive later joins.
            for field in ("continuation_candidate_prev_id", "continuation_candidate_next_id"):
                if item.get(field) in member_ids:
                    item[field] = ""
            item["continuation_prev_text"] = normalize_text(chain[pos - 1].get("protected_source_text", "")) if pos > 0 else ""
            item["continuation_next_text"] = normalize_text(chain[pos + 1].get("protected_source_text", "")) if pos < len(chain) - 1 else ""
        annotated += len(chain)
    return annotated


def apply_candidate_pair_joins(payload: list[dict], approved_pairs: list[tuple[str, str]]) -> int:
    return merge_continuation_pairs(
        payload, approved_pairs, decision="review_joined", group_prefix="cg-review",
    )

