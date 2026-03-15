// Simple javascript script to generate the download urls from the language selection.

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
        hidden.value = "";
        dropdown.querySelectorAll("div").forEach((div) => {
            div.style.display = div.textContent.toLowerCase().includes(q)
                ? ""
                : "none";
        });
    });
}

// Wires up a table row: initialises its comboboxes, listens for selections,
// and enables the download / copy-URL buttons when both languages are chosen.
function setupRow(row) {
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

        if (!target || (sourceHidden && !source)) {
            btn.disabled = true;
            info.textContent = "Select the language(s)";
            return;
        }

        // Glossary constraint
        if (type === "glossary" && target === source) {
            btn.disabled = true;
            info.textContent = "⚠️ Target and source must be different";
            return;
        }

        const url = buildUrl(type, source, target);
        if (!url) {
            btn.disabled = true;
            info.textContent = "";
            return;
        }

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

// I don't think this is ideal (it is called on every tab switch, and not only on the download's one),
// but it's the only thing I got working...
// cf. https://github.com/squidfunk/mkdocs-material/discussions/6788#discussioncomment-8498415
document$.subscribe(function () {
    // Mark table as loaded to fade it in
    const table = document.querySelector(".download-table");
    if (table) {
        table.classList.add("loaded");
    }

    document
        .querySelectorAll(".download-table tr[data-type]")
        .forEach(setupRow);
});
