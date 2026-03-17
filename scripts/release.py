"""Run the rust binary over a matrix of languages.

- The languages are collected from languages.json
- Generated dictionaries are stored @ data/release
- Then, use huggingface_hub API to:
  - update the huggingface metadata (README, logs etc.)
  - upload the data/release folder

Uploading to the hub requires:
pip install python-dotenv huggingface-hub

---

To modify the huggingface repo:
git clone https://huggingface.co/datasets/daxida/wty-release
...
changes
...
git push
(when it says enter password, actually type the token...)
"""

import argparse
import datetime
import json
import os
import re
import shutil
import subprocess
import time
from collections.abc import Callable
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path
from pprint import pprint
from typing import Any, Literal

REPO_ID_HF = "daxida/wty-release"
REPO_HF = f"https://huggingface.co/datasets/{REPO_ID_HF}"
REPO_ID_GH = "https://github.com/daxida/wty"

BINARY_PATH = "target/release/wty"

ANSI_ESCAPE_RE = re.compile(r"\x1B[@-_][0-?]*[ -/]*[@-~]")

type DictTy = Literal["main", "ipa", "ipa-merged", "glossary"]


@dataclass
class Args:
    verbose: int
    dry_run: bool
    jobs: int
    dtype: DictTy | None


def release_version() -> str:
    """The version of the release.

    Different from the crate semantic version. This uses calver.
    """
    return datetime.datetime.now().strftime("%Y-%m-%d")


class PathManager:
    def __init__(self, root_dir: Path) -> None:
        self.root_dir = root_dir

        self.release = self.root_dir / "release"
        self.dictionary = self.release / "dict"  # self.dict has messed highlighting
        self.index = self.release / "index"
        self.readme = self.release / "README.md"
        self.download = self.release / "kaikki"

        # These are at the "github repo root"
        self.assets = Path("assets")
        self.languages_json = self.assets / "languages.json"
        self.log = Path("log.txt")

        # Stage structure
        self.stage = self.release / "stage"
        self.version = self.stage / "versions" / release_version()
        self.latest = self.stage / "latest"

    def setup(self) -> None:
        self.release.mkdir(exist_ok=True)

    def check_dict_dir(self) -> None:
        if not self.dictionary.exists() or not any(self.dictionary.iterdir()):
            print(f"No files found in {self.dictionary}")
            exit(1)


PM = PathManager(Path("data"))
"""Global to simplify the argument passing. Should be read-only."""


def clean(line: str) -> str:
    return ANSI_ESCAPE_RE.sub("", line)


def double_check(msg: str = "") -> None:
    if msg:
        print(msg)
    if input("Proceed? [y/n] ") != "y":
        print("Exiting.")
        exit(1)


def human_size(size_bytes: float, precision: int = 2) -> str:
    for unit in ("B", "KB", "MB"):
        if size_bytes < 1024:
            return f"{size_bytes:.{precision}f} {unit}"
        size_bytes /= 1024
    return f"{size_bytes:.{precision}f} GB"


def stats(
    path: Path,
    *,
    file_pattern: str | None = None,
    endswith: str | None = None,
) -> tuple[int, str]:
    n_files = 0
    size_files = 0
    for f in path.rglob("*"):
        if f.is_file():
            if file_pattern is not None and not re.match(file_pattern, f.name):
                continue
            if endswith is not None and not f.name.endswith(endswith):
                continue
            n_files += 1
            size_files += f.stat().st_size
    return n_files, human_size(size_files)


def prepare_stage() -> None:
    """Take the release folder created with wty and structure it to comply with
    huggingface upload_large_folder.

    The resulting layout is:

        stage/
        ├── versions/
        │   └── {version}/
        │       ├── dict/
        │       ├── index/
        │       ├── README.md
        │       └── log.txt
        └── latest/
            ├── dict/
            ├── index/
            ├── README.md
            └── log.txt

    We copy (not move) files so the local release directory remains intact.

    The README and log shown on the Hugging Face repo root are handled
    separately and do not require upload_large_folder.
    """
    PM.stage.mkdir()  # Fail if exists

    for destination in (PM.version, PM.latest):
        print(f"Copying release to {destination}...")
        shutil.copytree(str(PM.dictionary), destination / "dict")
        shutil.copytree(str(PM.index), destination / "index")


# https://huggingface.co/new-dataset
# https://huggingface.co/settings/tokens
def upload_to_huggingface() -> None:
    from dotenv import load_dotenv
    from huggingface_hub import HfApi, whoami

    PM.check_dict_dir()

    try:
        # Requires an ".env" file with
        # HF_TOKEN="hf_..."
        load_dotenv()
        user_info = whoami()
        print(f"✓ Successfully logged in as: {user_info['name']}")
    except Exception as e:
        print(f"✗ Login failed: {e}")
        return

    dict_dir = PM.dictionary
    _, size = stats(dict_dir)
    stage_dir = PM.stage
    version = release_version()
    git_cmd = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=".")
    commit_sha = git_cmd.decode().strip()
    commit_sha_short = commit_sha[:7]

    kwargs = dict(
        folder_path=str(stage_dir),
        repo_id=REPO_ID_HF,
        repo_type="dataset",
    )

    print()
    print(commit_sha_short, commit_sha)
    pprint(kwargs)
    print(f"{version=}")
    print()
    print(f"Upload {dict_dir} ({size}) to {REPO_ID_HF}?")
    double_check()

    api = HfApi()

    # Upload dict + index (stage folder)
    prepare_stage()
    api.upload_large_folder(**kwargs)  # type: ignore
    print(f"Upload complete @ https://huggingface.co/datasets/{REPO_ID_HF}")

    # Upload README and logs at root, and also to latest and versions folders.
    readme_path = PM.readme
    update_readme_local(readme_path, commit_sha, version)

    for folder_in_repo in ("", f"versions/{release_version()}", "latest"):
        api.upload_file(
            path_or_fileobj=str(readme_path),
            path_in_repo=f"{folder_in_repo}/README.md",
            repo_id=REPO_ID_HF,
            repo_type="dataset",
            commit_message=f"[{version}] update README",
        )
        print(f"Uploaded README @ {folder_in_repo or 'root'}")

        # TODO: Logs are outdated and should be removed.
        # The only reason we don't delete it is in case we replace it with rust metadata
        if PM.log.exists():
            api.upload_file(
                path_or_fileobj=str(PM.log),
                path_in_repo=f"{folder_in_repo}/log.txt",
                repo_id=REPO_ID_HF,
                repo_type="dataset",
                commit_message=f"[{version}] update logs",
            )
            print(f"Uploaded logs @ {folder_in_repo or 'root'}")


def update_readme_local(readme_path: Path, commit_sha: str, version: str) -> None:
    """Write the README of the huggingface repo @ readme_path."""
    commit_sha_short = commit_sha[:7]
    commit_sha_link = f"{REPO_ID_GH}/commit/{commit_sha}"
    logs_link = f"{REPO_HF}/blob/main/log.txt"

    readme_content = f"""---
license: cc-by-sa-4.0
---
⚠️ **This dataset is automatically uploaded.**

For source code and issue tracking, visit the GitHub repo at [wty]({REPO_ID_GH})

version: {version}

commit: [{commit_sha_short}]({commit_sha_link})

logs: [link]({logs_link})
"""

    readme_path.write_text(readme_content, encoding="utf-8")


# duplicated from build
@dataclass
class Lang:
    iso: str
    language: str
    display_name: str
    flag: str
    # https://github.com/tatuylonen/wiktextract/tree/master/src/wiktextract/extractor
    has_edition: bool


# duplicated from build
def load_lang(item: Any) -> Lang:
    return Lang(
        item["iso"],
        item["language"],
        item["displayName"],
        item["flag"],
        item.get("hasEdition", False),
    )


def build_binary() -> None:
    subprocess.run(
        ["cargo", "build", "--release", "--quiet"],
        check=True,
    )


def binary_version() -> str:
    result = subprocess.run(
        [BINARY_PATH, "--version"],
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def run_cmd(
    root_dir: Path,
    cmd_name: str,
    # <source>-<target>, <source>-<source>, all, etc.
    # they are expected to be space separated
    params: str,
    args: Args,
    *,
    print_download_status: bool = False,
) -> tuple[int, list[str]]:
    cmd = [
        BINARY_PATH,
        cmd_name,
        *params.split(" "),
        f"--root-dir={root_dir}",
    ]
    # Return logs to guarantee some order
    logs = []

    if args.dry_run:
        line = " ".join(cmd)
        logs.append(clean(line))
        return 0, logs

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            check=True,  # check=False ignores errors
        )
    except subprocess.CalledProcessError as e:
        # Some pairs may not have a dump from the English edition.
        # If the language is rare, this is to be expected, but there are also languages
        # like Kurdish (ku), which have an edition but no dump from the English edition.
        #
        # Every language with a dump from the English edition can be found here:
        # https://kaikki.org/dictionary/
        #
        # We ignore the 404 that we get when requesting the dictionary
        #
        # NOTE: if we go with the database approach, this is pointless since we should never
        # use the preprocessed files.
        if (
            cmd_name in ("ipa", "main", "download") and params.split(" ")[1] == "en"
        ) or (cmd_name == "ipa-merged" and params.split(" ")[0] == "ku"):
            # print("[warn] Failed to run cmd:", clean(" ".join(cmd)))
            return 0, logs
        log("[err]", f"Command failed: {' '.join(cmd)}")
        log("[err-stdout]", e.stdout)
        log("[err-stderr]", e.stderr)
        raise

    match args.verbose:
        case 1:
            out = result.stdout.decode("utf-8")
            for line in out.splitlines():
                line = clean(line)
                if "Wrote yomitan dict" in line:
                    logs.append(line)
                if print_download_status and "ownload" in line:
                    logs.append(line)
        case 2:
            out = result.stdout.decode("utf-8")
            for line in out.splitlines():
                line = clean(line)
                print(line)
                logs.append(line)

    return (result.returncode, logs)


def log(*values, **kwargs) -> None:
    """Poor man's loguru"""
    line = ""

    match values:
        case []:
            pass
        case [one]:
            line = one
        case [label, msg]:
            label = f"[{label}]"
            line = f"{label:<15} {msg}"
        case _:
            raise RuntimeError

    print(line, **kwargs)

    # Bad bad bad
    with PM.log.open("a") as f:
        f.write(line + "\n")


# see path.rs::dict_name_expanded
def pattern(dict_ty: DictTy, sources: list[str], targets: list[str]) -> str:
    sources_re = "|".join(sources)
    targets_re = "|".join(targets)

    match dict_ty:
        case "main":
            fp = rf"wty-({targets_re})-({sources_re})\.zip"
        case "ipa":
            # dict/el/ja/wty-el-ja-ipa.zip
            fp = rf"wty-({targets_re})-({sources_re})-ipa\.zip"
        case "ipa-merged":
            # dict/ja/all/wty-ja-ipa.zip
            fp = rf"wty-({sources_re})-ipa\.zip"
        case "glossary":
            fp = rf"wty-({sources_re})-({targets_re})-gloss\.zip"

    return fp


# TODO: delete, this is done in rust
def run_matrix(langs: list[Lang], args: Args) -> None:
    start = time.perf_counter()

    odir = PM.release
    n_workers = min(args.jobs, os.cpu_count() or 1)

    log("info", f"n_workers {n_workers}")
    log("info", args)
    check_previous_files("info", odir)
    check_previous_files("info", odir, file_type="zip")
    log()

    # Clear logs
    with PM.log.open("w") as f:
        f.write("")

    run_prelude()

    isos = [lang.iso for lang in langs]
    # A subset for testing
    # isos = [
    #     # "sq",
    #     # "arz",
    #     "ku",
    #     "el",
    #     "en",
    # ]

    with_edition = [lang.iso for lang in langs if lang.has_edition]
    # A subset for testing
    # with_edition = [
    #     # "el",
    #     "en",
    #     # "ku",
    #     # "zh",
    #     # "ja",
    # ]

    isos_no_simple = [iso for iso in isos if iso != "simple"]
    with_edition_no_simple = [iso for iso in with_edition if iso != "simple"]

    matrix: list[tuple[DictTy, list[str], Callable[[str], list[str]]]] = [
        (
            "main",
            with_edition,
            lambda ed: isos_no_simple if ed != "simple" else ["simple"],
        ),
        ("ipa", with_edition_no_simple, lambda _: isos_no_simple),
        ("glossary", with_edition_no_simple, lambda _: isos_no_simple),
        ("ipa-merged", with_edition_no_simple, lambda _: ["__target"]),
    ]

    if args.dtype is not None:
        matrix = [run for run in matrix if run[0] == args.dtype]

    n_editions = len(with_edition)
    n_languages = len(isos)
    log("ALL", f"Editions ({n_editions}):  {' '.join(sorted(with_edition))}")
    log("ALL", f"Languages ({n_languages}): {' '.join(sorted(isos))}")
    # dictionary_types: list[DictTy] = [
    #     # The order is relevant to prevent multiple workers downloading
    #     # "ipa",
    #     # "main",
    #     # "ipa-merged",
    # ]
    # log("ALL", f"Dictionaries: {' '.join(sorted(dictionary_types))}")
    log("ALL", f"Dictionaries: {' '.join(run[0] for run in matrix)}")
    log()

    # We first download the jsonl because otherwise each worker will not find it in disk
    # and will try to download it itself... This way, we guarantee only one download happens.
    #
    # NOTE: when testing with subsets, if ipa-merged is in the matrix we will download all editions...
    run_download(odir, isos, with_edition, args)

    log("ALL", "Starting...")
    for dict_ty, sources, target_lambda in matrix:
        dict_start = time.perf_counter()
        log(dict_ty, "Making dictionaries...")

        for source in sources:
            targets = target_lambda(source)
            source_start = time.perf_counter()
            label = f"{source}-{dict_ty}"
            all_logs: list[str] = []

            def worker(target: str) -> tuple[int, list[str]]:
                match dict_ty:
                    case "main" | "ipa":
                        params = f"{target} {source}"
                    case "glossary":
                        # Ignore these
                        if source == target:
                            return 0, []
                        params = f"{source} {target}"
                    case "ipa-merged":
                        params = f"{source}"
                    case _:
                        raise RuntimeError("invalid dict_ty")
                return run_cmd(odir, dict_ty, params, args)

            with ThreadPoolExecutor(max_workers=n_workers) as executor:
                for _, logs in executor.map(worker, targets):
                    # log("DONE", f"{dict_ty} {source} {target}")
                    all_logs.extend(logs)

            for logline in sorted(all_logs):
                log(logline)

            elapsed = time.perf_counter() - source_start
            log(label, f"Finished dict ({elapsed:.2f}s)")

        fp = pattern(dict_ty, sources, targets)  # type: ignore
        _, total_size = stats(odir, file_pattern=fp)
        elapsed = time.perf_counter() - dict_start
        log(dict_ty, f"Finished dicts ({elapsed:.2f}s, {total_size})")

    n_dictionaries, total_size = stats(odir, endswith=".zip")
    elapsed = time.perf_counter() - start
    log("ALL", f"Finished! ({elapsed:.2f}s, {total_size}, {n_dictionaries} dicts)")


def check_previous_files(label: str, path: Path, file_type: str = "") -> None:
    if file_type:
        n_files, total_size = stats(path, endswith=file_type)
        file_msg = f"{file_type} files"
    else:
        n_files, total_size = stats(path)
        file_msg = "files"

    if n_files > 0:
        log(
            label,
            f"Found previous {file_msg} ({total_size}, {n_files} files) @ {path}",
        )
    else:
        log(label, f"Clean directory. No previous {file_msg} found @ {path}")


def run_prelude() -> None:
    log("prelude", "Building Rust binary...")
    build_binary()
    log("prelude", binary_version())
    rversion = release_version()
    log("prelude", f"dic {rversion}")
    log()


def run_download(
    odir: Path, isos: list[str], with_edition: list[str], args: Args
) -> None:
    start = time.perf_counter()

    log("dl", "Downloading editions...")
    check_previous_files("dl", PM.download)

    # Download editions (English only downloads the filtered en-en)
    for source in with_edition:
        label = f"dl-{source}"
        params = f"{source} {source}"
        _, logs = run_cmd(odir, "download", params, args, print_download_status=True)
        for logline in logs:
            log(logline)
        _, size = stats(PM.download, endswith=f"{source}-extract.jsonl")
        log(label, f"Finished download ({size})")

    # Download the rest of the filtered English jsonlines
    if "en" in with_edition:
        for source in isos:
            label = f"dl-{source}-en"
            params = f"{source} en"
            _, logs = run_cmd(
                odir, "download", params, args, print_download_status=True
            )
            for logline in logs:
                log(logline)
            _, size = stats(PM.download, endswith=f"{source}-en-extract.jsonl")
            log(label, f"Finished download ({size})")

    _, total_size = stats(PM.download)
    elapsed = time.perf_counter() - start
    log("dl", f"Finished downloads ({elapsed:.2f}s, {total_size})\n")


def build_release(args: Args) -> None:
    with PM.languages_json.open() as f:
        data = json.load(f)
        langs = [load_lang(row) for row in data]

    run_matrix(langs, args)


def parse_args() -> tuple[str, Args]:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["build", "publish", "index"])
    parser.add_argument("-v", "--verbose", action="count", default=0)
    parser.add_argument("-n", "--dry-run", action="store_true")
    parser.add_argument("-j", "--jobs", type=int, default=8)
    parser.add_argument(
        "-t", "--dictionary-type", choices=["main", "ipa", "ipa-merged"]
    )
    args = parser.parse_args()
    return args.command, Args(
        verbose=args.verbose,
        dry_run=args.dry_run,
        jobs=args.jobs,
        dtype=args.dictionary_type,
    )


def main() -> None:
    cmd, args = parse_args()

    PM.release.mkdir(exist_ok=True)

    match cmd:
        case "build":
            build_release(args)
        case "publish":
            upload_to_huggingface()
        case _:
            print(f"Unknown cmd: {cmd}")


if __name__ == "__main__":
    main()
