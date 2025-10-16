const languages = []
const latestUrl = 'https://pub-c3d38cca4dc2403b88934c56748f5144.r2.dev/releases/latest/'

async function fetchLanguages() {
    try {
        const resp = await fetch('https://raw.githubusercontent.com/yomidevs/kaikki-to-yomitan/refs/heads/master/languages.json')
        if (!resp.ok) throw new Error('Failed to fetch languages')
        const data = await resp.json()
        languages.push(...data)
        return data
    } catch (err) {
        console.error('Error fetching languages:', err)
        throw err
    }
}

const allLangs = []
const glossLangs = []
const langMap = {}

function updateLanguageData() {
    allLangs.length = 0
    glossLangs.length = 0
    for (const k in langMap) delete langMap[k]

    allLangs.push(...languages.filter(l => l.language))
    glossLangs.push(...languages.filter(l => l.hasEdition))
    for (const l of allLangs) langMap[l.iso] = l
}

function dropdownOptionNode({ iso, language, displayName, flag }) {
    const opt = document.createElement('option')
    opt.value = iso
    opt.textContent = `${flag} ${displayName || language}`
    return opt
}

function populateDropdown(selector, items, includeMerged = false) {
    const el = document.querySelector(selector)
    if (!el) return
    el.innerHTML = ''
    const sorted = [...items].sort((a, b) => {
        const nameA = a.displayName || a.language
        const nameB = b.displayName || b.language
        return nameA.localeCompare(nameB)
    })
    const list = includeMerged ? [{ iso: 'merged', language: 'Merged', displayName: 'Merged', flag: '🧬' }, ...sorted] : sorted
    for (const item of list) el.appendChild(dropdownOptionNode(item))
}

function updateDownloadLink(tgtSel, glossSel, linkSel, type) {
    const tgtEl = document.querySelector(tgtSel)
    const glossEl = document.querySelector(glossSel)
    const linkEl = document.querySelector(linkSel)
    if (!tgtEl || !glossEl || !linkEl) return
    const tgt = tgtEl.value
    const gloss = glossEl.value
    let url = ''
    if (type === 'main') {
        url = `${latestUrl}kty-${tgt}-${gloss}.zip`
    } else if (type === 'ipa') {
        url = gloss === 'merged'
            ? `${latestUrl}kty-${tgt}-ipa.zip`
            : `${latestUrl}kty-${tgt}-${gloss}-ipa.zip`
    } else if (type === 'translations') {
        url = `${latestUrl}kty-${tgt}-${gloss}-gloss.zip`
    }
    linkEl.setAttribute('href', url)
}

function setupDropdowns(sectionPrefix, type, isIPA = false) {
    populateDropdown(`#${sectionPrefix}-target`, allLangs)
    populateDropdown(`#${sectionPrefix}-gloss`, glossLangs, isIPA)
    updateDownloadLink(`#${sectionPrefix}-target`, `#${sectionPrefix}-gloss`, `#${sectionPrefix}-download`, type)

    const tgt = document.querySelector(`#${sectionPrefix}-target`)
    const gloss = document.querySelector(`#${sectionPrefix}-gloss`)
    if (tgt) tgt.addEventListener('change', () => updateDownloadLink(`#${sectionPrefix}-target`, `#${sectionPrefix}-gloss`, `#${sectionPrefix}-download`, type))
    if (gloss) gloss.addEventListener('change', () => updateDownloadLink(`#${sectionPrefix}-target`, `#${sectionPrefix}-gloss`, `#${sectionPrefix}-download`, type))
}

function validateTranslationsDropdowns() {

    /** @type {HTMLSelectElement | null} */
    const targetEl = document.querySelector('#trans-target');
    /** @type {HTMLSelectElement | null} */
    const glossEl = document.querySelector('#trans-gloss');

    if (!targetEl || !glossEl) return;

    const targetValue = targetEl.value;
    const glossValue = glossEl.value;

    if (targetValue === glossValue) {
        const availableGloss = allLangs.find(lang => lang.iso !== targetValue);
        if (availableGloss) {
            glossEl.value = availableGloss.iso;
        }
    }

    updateDownloadLink('#trans-target', '#trans-gloss', '#trans-download', 'translations');
}

document.addEventListener('DOMContentLoaded', async () => {
    try {
        await fetchLanguages()
        updateLanguageData()

        setupDropdowns('main', 'main')
        setupDropdowns('ipa', 'ipa', true)

        populateDropdown('#trans-target', glossLangs)
        populateDropdown('#trans-gloss', allLangs)

        validateTranslationsDropdowns();

        const transTarget = document.querySelector('#trans-target')
        const transGloss = document.querySelector('#trans-gloss')

        if (transTarget) {
            transTarget.addEventListener('change', () => {
                validateTranslationsDropdowns();
            });
        }

        if (transGloss) {
            transGloss.addEventListener('change', () => {
                validateTranslationsDropdowns();
            });
        }

    } catch (err) {
        console.error('Error initializing page:', err)
    }
})