// Simple javascript script to generate the download urls from the language selection.

function _availableTargets(metadata, type, source) {
    return Object.keys(metadata[type]?.sources?.[source]?.targets ?? {});
}

// group by sources containing this target
function _availableSources(metadata, type, target) {
    const sources = metadata[type]?.sources ?? {};

    return Object.keys(sources).filter(source =>
        Object.keys(sources[source]?.targets ?? {}).includes(target)
    );
}

function availableTargets(metadata, type, source) {
    switch (type) {
        case "main":
        case "ipa":
            return _availableTargets(metadata, type, source);
        case "glossary":
            return _availableSources(metadata, type, source);
        case "ipa-merged":
            console.warn(`availableTargets called for ipa-merged with source=${source}`);
        default:
            return null;
    }
}

function availableSources(metadata, type, target) {
    switch (type) {
        case "main":
        case "ipa":
            return _availableSources(metadata, type, target);
        case "glossary":
            return _availableTargets(metadata, type, target);
        case "ipa-merged":
            console.warn(`availableSources called for ipa-merged with target=${target}`);
        default:
            return null;
    }
}

function filterDropdown(box, allowed) {
    const set = new Set(allowed);
    box.querySelectorAll("div[data-value]").forEach(div => {
        div.style.display = set.has(div.dataset.value) ? "" : "none";
    });
}

// Cf. src/path.rs::dict_name_expanded
function buildUrl(type, source, target) {
    const BASE_URL =
        "https://huggingface.co/datasets/daxida/wty-release/resolve/main/latest/dict";
    switch (type) {
        case "main":
            return `${BASE_URL}/${source}/${target}/wty-${source}-${target}.zip`;

        case "ipa":
            return `${BASE_URL}/${source}/${target}/wty-${source}-${target}-ipa.zip`;

        case "ipa-merged":
            return `${BASE_URL}/all/${target}/wty-${target}-ipa.zip`;

        case "glossary":
            return `${BASE_URL}/${source}/${target}/wty-${source}-${target}-gloss.zip`;

        default:
            return null;
    }
}

// Converts a combobox wrapper into an interactive searchable dropdown.
// Replaces <option> tags with clickable <div>s and wires up filtering on input.
function setupCombobox(box) {
    if (!box) return;
    const search = box.querySelector(
        "input[type=text], input:not([type=hidden])",
    );
    const dropdown = box.querySelector(".dl-source-dropdown, .dl-target-dropdown");
    const hidden = box.querySelector("input[type=hidden]");
    const items = Array.from(dropdown.querySelectorAll("option"));

    function clearSelection() {
        hidden.value = "";
        // Reset dropdown (show everything again)
        dropdown.querySelectorAll("div").forEach(div => {
            div.style.display = "";
        });
        hidden.dispatchEvent(new Event("change", { bubbles: true }));
    }

    // Render items as divs for clicking
    dropdown.innerHTML = "";
    items.forEach((opt) => {
        const div = document.createElement("div");
        div.textContent = opt.textContent;
        div.dataset.value = opt.value;
        div.addEventListener("mousedown", () => {
            search.value = opt.textContent;
            hidden.value = opt.value;
            dropdown.style.display = "none";
            hidden.dispatchEvent(new Event("change", { bubbles: true }));
        });
        dropdown.appendChild(div);
    });

    search.addEventListener("focus", () => (dropdown.style.display = "block"));
    search.addEventListener("blur", () =>
        setTimeout(() => (dropdown.style.display = "none"), 150),
    );
    search.addEventListener("input", () => {
        const q = search.value.toLowerCase();
        dropdown.style.display = "block";

        dropdown.querySelectorAll("div").forEach((div) => {
            div.style.display = div.textContent.toLowerCase().includes(q)
                ? ""
                : "none";
        });

        if (!q) {
            clearSelection();
        }
    });

    // Complete with first match when clicking the "Enter" key
    search.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
            e.preventDefault();

            const firstVisible = Array.from(
                dropdown.querySelectorAll("div")
            ).find(div => div.style.display !== "none");

            if (firstVisible) {
                search.value = firstVisible.textContent;
                hidden.value = firstVisible.dataset.value;
                dropdown.style.display = "none";

                hidden.dispatchEvent(new Event("change", { bubbles: true }));
            } 
        }
    });
}



// Wires up a table row: initialises its comboboxes, listens for selections,
// and enables the download / copy-URL buttons when both languages are chosen.
function setupRow(row, metadata) {
    const type = row.dataset.type;
    const sourceHidden = row.querySelector(".dl-source");
    const targetHidden = row.querySelector(".dl-target");
    const btn = row.querySelector(".dl-btn");
    const info = row.querySelector(".dl-info");

    row.querySelectorAll(".dl-source-combobox, .dl-target-combobox").forEach(
        setupCombobox,
    );

    function update() {
        const source = sourceHidden?.value;
        const target = targetHidden?.value;

        // console.log(`[download-${type}] source=${source} target=${target}`);

        // Filter targets based on selected source
        if (source) {
            const allowedTargets = availableTargets(metadata, type, source);
            filterDropdown(row.querySelector(".dl-target-combobox"), allowedTargets);
        }

        // Filter sources based on selected target
        if (target && type !== "ipa-merged") {
            const allowedSources = availableSources(metadata, type, target);
            filterDropdown(row.querySelector(".dl-source-combobox"), allowedSources);
        }

        // Can't do (!target || !source) because of ipa-merged
        if (!target || (sourceHidden && !source)) {
            btn.disabled = true;
            info.textContent = "Select the language(s)";
            return;
        }

        const url = buildUrl(type, source, target);
        if (!url) return;

        const downloadUrl = `${url}?download=true`;

        btn.disabled = false;

        // Copy URL button logic
        info.innerHTML = "";
        const copyBtn = document.createElement("button");
        copyBtn.type = "button";
        copyBtn.textContent = "📋 Copy URL";
        copyBtn.className = "copy-url-btn";

        copyBtn.onclick = async () => {
            try {
                await navigator.clipboard.writeText(downloadUrl);
                copyBtn.textContent = "✅ Copied!";
                setTimeout(() => (copyBtn.textContent = "📋 Copy URL"), 1500);
            } catch {
                copyBtn.textContent = "❌ Failed";
            }
        };
        info.appendChild(copyBtn);

        btn.onclick = () => {
            window.location.href = downloadUrl;
        };
    }

    sourceHidden?.addEventListener("change", update);
    targetHidden?.addEventListener("change", update);

    update();
}

// Mojo so that fetching works both locally and in a project repo
// There MUST be a better way to do this...
const REPO_NAME = "wiktionary-to-yomitan";
const BRANCH = "gh-pages"; // branch that serves the site
const base = document.querySelector('base')?.href || `https://yomidevs.github.io/${REPO_NAME}/`;
const metadataPromise = fetch(base + "release_metadata.json")
    .then(res => res.json())
    .then(json => json["dicts"]);

// I don't think this is ideal (it is called on every tab switch, and not only on the download's one),
// but it's the only thing I got working...
// cf. https://github.com/squidfunk/mkdocs-material/discussions/6788#discussioncomment-8498415
document$.subscribe(function () {
    metadataPromise.then((metadata) => {
        const table = document.querySelector(".download-table");
        if (!table) return;

        // Mark table as loaded to fade it in
        table.classList.add("loaded");

        table.querySelectorAll("tr[data-type]").forEach(row => {
            setupRow(row, metadata);
        });
    });
});
