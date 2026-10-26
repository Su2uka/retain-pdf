"""Body fixtures with both layout metadata and a pending translation payload."""
from continuation_test_support import payload_item


def body(name, page, text, zone, role, order=0, **kwargs):
    left = 40 if zone == "left_column" else 310
    top = 650 if role == "tail" else 50
    item = payload_item(
        item_id=name, page_idx=page, text=text, bbox=[left, top, left + 250, top + 80],
        layout_mode="double", layout_zone=zone, layout_boundary_role=role, **kwargs,
    )
    item.update(source_text=text, block_idx=order, reading_order=order,
                math_mode="direct_typst", should_translate=True, formula_map=[], protected_map=[])
    return item


def pages(items):
    result = {}
    for item in items:
        result.setdefault(item["page_idx"], []).append(item)
    return result
