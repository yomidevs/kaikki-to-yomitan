"""Keep tag_bank_term and tag_order jsons in sync."""

import json
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path

ASSETS_PATH = Path("assets")

DOCUMENTED_YOMITAN_TAG_CATEGORIES = {
    "name",
    "expression",
    "popular",
    "frequent",
    "archaism",
    "partOfSpeech",
    # These we don't care
    "dictionary",
    "frequency",
    "search",
    "pronunciation-dictionary",
}


# duplicated from build
@dataclass
class WhitelistedTag:
    short_tag: str
    category: str
    sort_order: int
    # if array, first element will be used, others are aliases
    long_tag_aliases: str | list[str]
    popularity_score: int


def main() -> None:
    jsons_root = ASSETS_PATH
    path_tag_order_json = jsons_root / "tag_order.json"
    path_tag_bank_json = jsons_root / "tag_bank_term.json"
    path_tag_bank_variety_json = jsons_root / "tag_bank_term_variety.json"
    with path_tag_bank_json.open("r", encoding="utf-8") as f:
        tag_bank = json.load(f)
    with path_tag_bank_variety_json.open("r", encoding="utf-8") as f:
        tag_bank_variety = json.load(f)
    with path_tag_order_json.open("r", encoding="utf-8") as f:
        tag_order = json.load(f)
    tag_bank.extend(tag_bank_variety)

    order_tags = []
    for group, tags in tag_order.items():
        for cats in tags:
            order_tags.append((group, cats))
    wtags = [WhitelistedTag(*row) for row in tag_bank]

    # Debug wtags categories
    # cf. https://github.com/yomidevs/yomitan/blob/master/docs/making-yomitan-dictionaries.md#tag-categories
    # Categories can be anything
    # cf. https://github.com/yomidevs/yomitan/blob/master/ext/data/schemas/dictionary-tag-bank-v3-schema.json#L4
    # But keep in mind that tags labels depend if they happen at top-level or not
    # [data-category="gender"] { display: none !important; }     < TOP-LEVEL
    # [data-sc-category="topic"] { display: none !important; }   < AT SENSE LEVEL
    unique_wtags_categories = Counter()
    for wtag in wtags:
        if wtag.category:
            unique_wtags_categories[wtag.category] += 1
    summary = {
        f"{'✔' if cat in DOCUMENTED_YOMITAN_TAG_CATEGORIES else '✖'} {cat}": count
        for cat, count in unique_wtags_categories.most_common()
    }
    print(summary)

    # Check if all tags in the same category have the same sort_order
    # ~ not really needed, but it makes sense
    category_sort_orders = {}
    for wtag in wtags:
        if wtag.category not in category_sort_orders:
            category_sort_orders[wtag.category] = Counter()
        category_sort_orders[wtag.category][wtag.sort_order] += 1

    for category, sort_orders in category_sort_orders.items():
        if category and len(sort_orders) > 1:
            min_count = min(sort_orders.values())
            bad_orders = {so for so, c in sort_orders.items() if c == min_count}
            offenders = [
                wt.short_tag
                for wt in wtags
                if wt.category == category and wt.sort_order in bad_orders
            ]
            print(
                f"Category '{category}' has inconsistent sort_orders: {dict(sort_orders)}, {offenders=}"
            )

    # Since we know that every category shares a sort_order, we can show
    # the categories that match every sort_order level
    sort_order_counter = Counter(wtag.sort_order for wtag in wtags)
    print(f"Sort order counts: {sort_order_counter.most_common()}")
    sort_order_categories = defaultdict(set)
    for wt in wtags:
        cat = wt.category or "None"
        sort_order_categories[wt.sort_order].add(cat)
    for so, categories in sorted(sort_order_categories.items()):
        cats = ", ".join(sorted(categories))
        print(f"  * {so:>3}: {cats}")

    # Quick diagnostic search
    # Compares tag_order groups with whitelisted tags categories
    # ~ even though it is fine if those two files are not perfectly in sync
    #
    # for group, otag in order_tags:
    #     for wtag in wtags:
    #         # otag is wtag.short_tag, or appears in wtag.long_tag_aliases
    #         if (
    #             otag == wtag.short_tag
    #             or (
    #                 isinstance(wtag.long_tag_aliases, list)
    #                 and otag in wtag.long_tag_aliases
    #             )
    #             or (
    #                 isinstance(wtag.long_tag_aliases, str)
    #                 and otag == wtag.long_tag_aliases
    #             )
    #         ):
    #             # print(f"       {otag} > {wtag.short_tag}")
    #             if wtag.category:
    #                 print(f"ordered_group: {group}, category: {wtag.category} ({otag})")
    #             break
    #     else:
    #         # Tag in tag_order.json but not in the bank
    #         # print(f"[miss] {otag}")
    #         pass


if __name__ == "__main__":
    main()
