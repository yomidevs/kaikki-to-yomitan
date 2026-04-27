"""Write a test registry and update tests from there.

The registry must contain the original json lines with attributes that we may not
use in the making of a dictionary, in case we use them in the future.

The registry is only intended to be used via git diffs, to check for updates.
"""

import argparse
import json
import re
from difflib import SequenceMatcher
from pathlib import Path
from typing import Any, Literal, TypedDict, cast, get_args

import requests

PATH_TESTS_DIR = Path("tests")
PATH_TESTS_INPUT = PATH_TESTS_DIR / "kaikki"
PATH_REGISTRY = PATH_TESTS_DIR / "registry.json"
PATH_REGISTRY_TIMESTAMPS = PATH_TESTS_DIR / "registry_timestamps.json"

L = Literal[
    "ar",
    "cs",
    "de",
    "el",
    "en",
    "es",
    "fa",
    "fi",
    "fr",
    "ga",
    "grc",
    "is",
    "it",
    "ja",
    "ko",
    "la",
    "pl",
    "pt",
    "ru",
    "sq",
    "simple",
    "th",
    "zh",
]
"""A language code that appears in the testsuite."""

ALLOWED_LANGS = frozenset(get_args(L))

LangPairs = dict[L, list[L]]


class RegValue(TypedDict):
    url: str
    download_url: str
    json: Any


Reg = dict[L, dict[L, list[RegValue]]]
"""A registry that we will dump as JSON at REGISTRY_PATH."""

Timestamp = str
Timestamps = dict[L, dict[L, list[Timestamp]]]


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


def get_test_path(source: L, target: L) -> Path:
    return PATH_TESTS_INPUT / f"{source}-{target}-extract.jsonl"


def add_to_registry(registry: Reg, source: L, target: L, value: RegValue) -> None:
    if source not in registry:
        registry[source] = {target: []}
    registry[source][target].append(value)


def update_registry_for_pair(source: L, target: L) -> tuple[Reg, Timestamps]:
    """Get registry and timestamps for the source-target language pair.

    Timestamps are given separatedly so that they can be also writen as such. Preventing
    noise in the registry diffs.
    """
    print(f"Updating {source}-{target} (registry)", flush=True)
    tests_path = get_test_path(source, target)
    tests = read_jsonl(tests_path.read_text())

    registry: Reg = {}
    timestamps: Timestamps = {}

    for test in tests:
        word = test["word"]

        search_query = "/".join([word[0], word[:2], word])
        # We can replace the "All languages combined" with the source but it requires
        # knowing how to convert from an iso (en) to a long name (English)
        if target == "en":
            url = f"https://kaikki.org/dictionary/All%20languages%20combined/meaning/{search_query}.jsonl"
        else:
            url = f"https://kaikki.org/{target}wiktionary/All%20languages%20combined/meaning/{search_query}.jsonl"

        resp = requests.get(url)
        if not resp.ok:
            print(
                f"[WARN] (err. {resp.status_code}) Failed to fetch {word} @ {url}\n"
                "Ignore this if the given word is a custom testcase not in kaikki"
            )
            # If it is a custom testcase, while not ideal, we still store it in the registry
            # for simplicity. This way we just need to read the registry for updating tests.
            custom_test: RegValue = {
                "url": "none",
                "download_url": "none",
                "json": test,
            }
            add_to_registry(registry, source, target, custom_test)
            continue

        last_modified = resp.headers.get("Last-Modified", "None")
        if source not in timestamps:
            timestamps[source] = {target: []}
        timestamps[source][target].append(last_modified)

        text = resp.content.decode("utf-8")
        jsonl = read_jsonl(text)

        scores = [
            (i, json_similarity(test, cand), cand) for i, cand in enumerate(jsonl)
        ]
        scores.sort(reverse=True, key=lambda x: x[1])
        _, _, best_match = scores[0]

        # reorder keys for visibility
        best_match = {
            "word": best_match["word"],
            "pos": best_match["pos"],
            **{k: v for k, v in best_match.items() if k not in ("word", "pos")},
        }

        registry_value: RegValue = {
            "url": url.replace(".jsonl", ".html"),
            "download_url": url,
            "json": best_match,
        }
        add_to_registry(registry, source, target, registry_value)

    return (registry, timestamps)


def validate_lang(lang: str) -> L:
    if lang not in ALLOWED_LANGS:
        raise ValueError(f"String {lang} was not found in L")
    return cast(L, lang)


FNAME_RE = re.compile(r"^([a-zA-Z]+)-([a-zA-Z]+)-extract\.jsonl$")


def get_lang_pairs_to_update(
    source_filter: str | None, target_filter: str | None
) -> LangPairs:
    lang_pairs: LangPairs = {}

    for file in PATH_TESTS_INPUT.iterdir():
        m = FNAME_RE.match(file.name)
        if not m:
            raise ValueError(f"Unexpected filename in {PATH_TESTS_INPUT}: {file.name}")

        source_raw = m.group(1)
        target_raw = m.group(2)

        if source_filter and source_raw != source_filter:
            continue
        if target_filter and target_raw != target_filter:
            continue

        source = validate_lang(source_raw)
        target = validate_lang(target_raw)

        if source not in lang_pairs:
            lang_pairs[source] = []
        lang_pairs[source].append(target)

    lang_pairs = {src: sorted(lang_pairs[src]) for src in sorted(lang_pairs)}

    return lang_pairs


def update_registry(lang_pairs: LangPairs, load_prev_registry: bool) -> None:
    """Update the registry with the given language pairs.

    Note that this loads the previous registry, if any, to prevent deleting every
    entry that did not match the --target filter, it some was passed via the CLI.

    It has the drawback that if we ever deleted some test from the testsuite, it
    will still appear in the registry. This never happened, so we opt for a middle
    way solution, of only loading the registry if some filter was passed. Again, this
    is not infallible.

    NOTE: there is no guarantee that the registry/timestamps will be sorted when
    loading previous data.
    """
    registry: Reg = {}
    timestamps: Timestamps = {}

    if load_prev_registry:
        if PATH_REGISTRY.exists():
            with PATH_REGISTRY.open() as f:
                registry = json.load(f)
        if PATH_REGISTRY_TIMESTAMPS.exists():
            with PATH_REGISTRY_TIMESTAMPS.open() as f:
                timestamps = json.load(f)

    for source, targets in lang_pairs.items():
        if source not in registry:
            registry[source] = {}
        if source not in timestamps:
            timestamps[source] = {}

        for target in targets:
            pair_registry, pair_timestamps = update_registry_for_pair(source, target)
            registry[source][target] = pair_registry[source][target]
            timestamps[source][target] = pair_timestamps[source][target]

    PATH_REGISTRY.write_text(json.dumps(registry, indent=2, ensure_ascii=False))
    PATH_REGISTRY_TIMESTAMPS.write_text(
        json.dumps(timestamps, indent=2, ensure_ascii=False)
    )


def update_tests() -> None:
    if not PATH_REGISTRY.exists():
        print(f"{PATH_REGISTRY} not found. Run with flag '--update-registry' first.")
        return

    registry: Reg = json.loads(PATH_REGISTRY.read_text())

    for source, target in registry.items():
        for target, registry_values in target.items():
            new_tests: list[str] = [value["json"] for value in registry_values]
            tests_path = get_test_path(source, target)
            print(f"Updating {source}-{target} (test)", flush=True)
            with tests_path.open("w", encoding="utf-8") as f:
                for new_test in new_tests:
                    json.dump(new_test, f, ensure_ascii=False)
                    f.write("\n")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Update and manage Kaikki tests",
    )
    parser.add_argument(
        "--source",
        help="update the registry only with source languages matching this value",
    )
    parser.add_argument(
        "--target",
        help="update the registry only with target languages matching this value",
    )
    parser.add_argument(
        "--update-registry",
        action="store_true",
        help="update the registry before updating tests",
    )
    args = parser.parse_args()

    if args.update_registry:
        print(f"Updating registry at {PATH_REGISTRY}")
        lang_pairs = get_lang_pairs_to_update(args.source, args.target)
        # Load previous if there are filters
        load_prev_registry = args.source or args.target
        update_registry(lang_pairs, load_prev_registry)

    print(f"Updating tests at {PATH_TESTS_INPUT}")
    update_tests()


if __name__ == "__main__":
    main()
