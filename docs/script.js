const languages = []

async function fetchLanguages() {
    try {
        const response = await fetch('https://raw.githubusercontent.com/yomidevs/kaikki-to-yomitan/refs/heads/master/languages.json')
        if (!response.ok) {
            throw new Error('Failed to fetch languages')
        }
        const data = await response.json()
        languages.push(...data)
        return data
    } catch (error) {
        console.error('Error fetching languages:', error)
        throw error
    }
}

const allLangs = []
const glossLangs = []
const langMap = {}

function updateLanguageData() {
    allLangs.length = 0
    glossLangs.length = 0
    Object.keys(langMap).forEach(key => delete langMap[key])
    
    allLangs.push(...languages.filter(l => l.language))
    glossLangs.push(...languages.filter(l => l.hasEdition))
    Object.assign(langMap, Object.fromEntries(allLangs.map(l => [l.iso, l])))
}

const dropdownOption = ({ iso, language, flag }) => `<option value="${iso}">${flag} ${language}</option>`

function populateDropdown(selector, items, includeMerged = false) {
    const options = [...items]
    if (includeMerged) {
        options.unshift({ iso: 'merged', language: 'Merged', flag: '🧬' })
    }
    $(selector).html(options.map(dropdownOption).join(''))
}

function updateDownloadLink(tgtSel, glossSel, linkSel, type) {
    const tgt = $(tgtSel).val()
    const gloss = $(glossSel).val()
    let url
    if (type === 'main') {
        url = `https://github.com/yomidevs/kaikki-to-yomitan/releases/latest/download/kty-${tgt}-${gloss}.zip`
    } else if (type === 'ipa') {
        url = gloss === 'merged'
            ? `https://github.com/yomidevs/kaikki-to-yomitan/releases/latest/download/kty-${tgt}-ipa.zip`
            : `https://github.com/yomidevs/kaikki-to-yomitan/releases/latest/download/kty-${tgt}-${gloss}-ipa.zip`
    } else if (type === 'translations') {
        url = `https://github.com/yomidevs/kaikki-to-yomitan/releases/latest/download/kty-${tgt}-${gloss}-gloss.zip`
    }
    $(linkSel).attr('href', url)
}


function setupDropdowns(sectionPrefix, type, isIPA = false) {
    populateDropdown(`#${sectionPrefix}-target`, allLangs)
    populateDropdown(`#${sectionPrefix}-gloss`, glossLangs, isIPA)
    updateDownloadLink(`#${sectionPrefix}-target`, `#${sectionPrefix}-gloss`, `#${sectionPrefix}-download`, type)
    $(`#${sectionPrefix}-target, #${sectionPrefix}-gloss`).on('change', () =>
        updateDownloadLink(`#${sectionPrefix}-target`, `#${sectionPrefix}-gloss`, `#${sectionPrefix}-download`, type)
    )
}

function makeTable(id, glosses, type = 'main', isIPA = false) {
    if (type === 'translations') {
        const headers = ['To \\ From', ...glosses.map(g => `${g.flag} ${g.language}`)]  // glosses = "from"
        const rows = allLangs.map(toLang => {  // allLangs = "to"
            const row = [`${toLang.flag} ${toLang.language}`]
            for (const fromLang of glosses) {
                if (fromLang.iso === toLang.iso) {
                    row.push('')  // skip monolingual entries
                } else {
                    const url = `https://github.com/yomidevs/kaikki-to-yomitan/releases/latest/download/kty-${fromLang.iso}-${toLang.iso}-gloss.zip`
                    row.push(`<a href="${url}" target="_blank">📥</a>`)
                }
            }
            return row
        })

        $(`#${id}`).DataTable({
            data: rows,
            columns: headers.map(h => ({ title: h })),
            paging: false,
            searching: true,
        })
        return
    }

    const headers = ['Target \\ Gloss', ...glosses.map(g => `${g.flag} ${g.language}`)]
    const rows = allLangs.map(rowLang => {
        const row = [`${langMap[rowLang.iso].flag} ${rowLang.language}`]
        for (const gloss of glosses) {
            let glossIso = gloss.iso

            if (glossIso === 'merged' && type === 'ipa') {
                const url = `https://github.com/yomidevs/kaikki-to-yomitan/releases/latest/download/kty-${rowLang.iso}-ipa.zip`
                row.push(`<a href="${url}" target="_blank">📥</a>`)
            } else if (type === 'ipa') {
                const url = `https://github.com/yomidevs/kaikki-to-yomitan/releases/latest/download/kty-${rowLang.iso}-${glossIso}-ipa.zip`
                row.push(`<a href="${url}" target="_blank">📥</a>`)
            } else {
                const url = `https://github.com/yomidevs/kaikki-to-yomitan/releases/latest/download/kty-${rowLang.iso}-${glossIso}.zip`
                row.push(`<a href="${url}" target="_blank">📥</a>`)
            }
        }
        return row
    })

    $(`#${id}`).DataTable({
        data: rows,
        columns: headers.map(h => ({ title: h })),
        paging: false,
        searching: true,
    })
}


$(document).ready(async function () {
    try {
        await fetchLanguages()
        updateLanguageData()
        
        setupDropdowns('main', 'main')
        setupDropdowns('ipa', 'ipa', true)
        populateDropdown('#trans-target', glossLangs)
        populateDropdown('#trans-gloss', allLangs)
        updateDownloadLink('#trans-target', '#trans-gloss', '#trans-download', 'translations')
        $('#trans-target, #trans-gloss').on('change', () =>
            updateDownloadLink('#trans-target', '#trans-gloss', '#trans-download', 'translations')
        )

        makeTable('mainTable', glossLangs, 'main')
        makeTable('ipaTable', [...glossLangs, { iso: 'merged', language: 'Merged', flag: '🧬' }], 'ipa', true)
        makeTable('translationTable', glossLangs, 'translations')

        $('.toggle-table').on('click', function () {
            const targetId = $(this).data('target')
            const isVisible = $(targetId).is(':visible')
            $(targetId).slideToggle(200)
            $(this).text(isVisible ? 'Show Table' : 'Hide Table')
        })
    } catch (error) {
        console.error('Error initializing page:', error)
        // You might want to show an error message to the user here
    }
})