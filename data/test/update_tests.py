import argparse
import json
from difflib import SequenceMatcher
from pathlib import Path
from typing import Any, Literal, TypedDict

import requests

TESTS_DIR_PATH = Path("data/test")
TESTS_PATH = TESTS_DIR_PATH / "kaikki"
REGISTRY_PATH = TESTS_DIR_PATH / "registry.json"

L = Literal[
    "cs", "de", "en", "es", "fa", "fr", "grc", "ja", "ko", "la", "ru", "sq", "th", "zh",
]
"""A language code."""


class RegItem(TypedDict):
    word_index: int
    url: str
    download_url: str
    json: Any


Reg = dict[L, dict[L, list[RegItem]]]
"""A registry that we will dump as JSON at REGISTRY_PATH."""

# TO_UPDATE: dict[L, list[L]] = {
#     # "de": ["de", "en"],
#     # # "fr": ["fr"],
#     # "en": ["de", "en"],
#     "grc": ["en"],
# }

TO_UPDATE: dict[L, list[L]] = {
    "cs": ["en"],
    "de": ["de", "en"],
    # Skip greek
    "en": ["de", "en", "es"],
    "es": ["en"],
    "fa": ["en"],
    "fr": ["en", "fr"],
    "grc": ["en"],
    "ja": ["en"],
    "ko": ["en"],
    "la": ["en"],
    "ru": ["en", "ru"],
    "sq": ["en"],
    "th": ["en"],
    "zh": ["en"],
}


def read_jsonl(text: str) -> list[Any]:
    return [json.loads(line) for line in text.strip().splitlines()]


def flatten_json(obj: Any, prefix: str = "") -> dict[str, str]:
    items = {}
    if isinstance(obj, dict):
        for k, v in obj.items():
            items.update(flatten_json(v, f"{prefix}.{k}" if prefix else k))
    elif isinstance(obj, list):
        for i, v in enumerate(obj):
            items.update(flatten_json(v, f"{prefix}[{i}]"))
    else:
        items[prefix] = str(obj)
    return items


def json_to_str(a: Any) -> str:
    return "\n".join(f"{k}: {v}" for k, v in sorted(flatten_json(a).items()))


def json_similarity(a: Any, b: Any) -> float:
    return SequenceMatcher(None, json_to_str(a), json_to_str(b)).ratio()


def get_test_path(fr_lang: L, to_lang: L) -> Path:
    # The extension should be JSONL really...
    return TESTS_PATH / f"{fr_lang}-{to_lang}.json"


def update_registry_for_pair(fr_lang: L, to_lang: L) -> Reg:
    print(f"Updating {fr_lang}-{to_lang} (registry)", flush=True)
    tests_path = get_test_path(fr_lang, to_lang)
    tests = read_jsonl(tests_path.read_text())

    registry: Reg = {}

    for test in tests:
        word = test["word"]

        search_query = "/".join([word[0], word[:2], word])
        if to_lang == "en":
            url = f"https://kaikki.org/dictionary/All%20languages%20combined/meaning/{search_query}.jsonl"
        else:
            url = f"https://kaikki.org/{to_lang}wiktionary/All%20languages%20combined/meaning/{search_query}.jsonl"

        resp = requests.get(url)
        resp.raise_for_status()
        text = resp.content.decode("utf-8")
        jsonl = read_jsonl(text)

        scores = [
            (i, json_similarity(test, cand), cand) for i, cand in enumerate(jsonl)
        ]
        scores.sort(reverse=True, key=lambda x: x[1])
        best_index, _, best_match = scores[0]

        if fr_lang not in registry:
            registry[fr_lang] = {to_lang: []}
        registry_item: RegItem = {
            "word_index": best_index,
            "url": url.replace(".jsonl", ".html"),
            "download_url": url,
            "json": best_match,
        }
        registry[fr_lang][to_lang].append(registry_item)

    return registry


def update_registry() -> None:
    registry: Reg = {}
    for fr_lang, to_langs in TO_UPDATE.items():
        if fr_lang not in registry:
            registry[fr_lang] = {}

        for to_lang in to_langs:
            registry_for_pair = update_registry_for_pair(fr_lang, to_lang)
            registry[fr_lang][to_lang] = registry_for_pair[fr_lang][to_lang]

    REGISTRY_PATH.write_text(json.dumps(registry, indent=2, ensure_ascii=False))


def update_tests() -> None:
    if not REGISTRY_PATH.exists():
        print(f"{REGISTRY_PATH} not found. Run with flag '--update-registry' first.")
        return

    registry: Reg = json.loads(REGISTRY_PATH.read_text())

    for fr_lang, to_langs in registry.items():
        for to_lang, registry_items in to_langs.items():
            tests_path = get_test_path(fr_lang, to_lang)
            new_tests: list[str] = [item["json"] for item in registry_items]
            print(f"Updating {fr_lang}-{to_lang} (test)", flush=True)
            with tests_path.open("w", encoding="utf-8") as f:
                for new_test in new_tests:
                    json.dump(new_test, f, ensure_ascii=False)
                    f.write("\n")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Update and manage Kaikki tests.",
    )
    parser.add_argument(
        "--update-registry",
        action="store_true",
        help="Update the registry before updating tests.",
    )
    args = parser.parse_args()

    if args.update_registry:
        print(f"Updating registry at {REGISTRY_PATH}")
        update_registry()

    print(f"Updating tests at {TESTS_PATH}")
    update_tests()


if __name__ == "__main__":
    main()
