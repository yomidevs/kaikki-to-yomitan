const languages = [
    { "iso": "sq", "language": "Albanian", "flag": "🇦🇱" },
    { "iso": "grc", "language": "Ancient Greek", "flag": "🏺" },
    { "iso": "ar", "language": "Arabic", "flag": "🟩" },
    { "iso": "aii", "language": "Assyrian Neo-Aramaic", "flag": "🌞" },
    { "iso": "zh", "language": "Chinese", "flag": "🇨🇳", "hasEdition": true },
    { "iso": "cs", "language": "Czech", "flag": "🇨🇿" },
    { "iso": "da", "language": "Danish", "flag": "🇩🇰" },
    { "iso": "nl", "language": "Dutch", "flag": "🇳🇱", "hasEdition": true },
    { "iso": "en", "language": "English", "flag": "🇬🇧", "hasEdition": true },
    { "iso": "eo", "language": "Esperanto", "flag": "🌍" },
    { "iso": "fi", "language": "Finnish", "flag": "🇫🇮" },
    { "iso": "fr", "language": "French", "flag": "🇫🇷", "hasEdition": true },
    { "iso": "de", "language": "German", "flag": "🇩🇪", "hasEdition": true },
    { "iso": "el", "language": "Greek", "flag": "🇬🇷", "hasEdition": true },
    { "iso": "afb", "language": "Gulf Arabic", "flag": "🇦🇪" },
    { "iso": "he", "language": "Hebrew", "flag": "🇮🇱" },
    { "iso": "hi", "language": "Hindi", "flag": "🇮🇳" },
    { "iso": "hu", "language": "Hungarian", "flag": "🇭🇺" },
    { "iso": "id", "language": "Indonesian", "flag": "🇮🇩" },
    { "iso": "ga", "language": "Irish", "flag": "🇮🇪" },
    { "iso": "it", "language": "Italian", "flag": "🇮🇹", "hasEdition": true },
    { "iso": "ja", "language": "Japanese", "flag": "🇯🇵", "hasEdition": true },
    { "iso": "kn", "language": "Kannada", "flag": "🇮🇳" },
    { "iso": "kk", "language": "Kazakh", "flag": "🇰🇿" },
    { "iso": "km", "language": "Khmer", "flag": "🇰🇭" },
    { "iso": "ku", "language": "Kurdish", "flag": "🇮🇶", "hasEdition": true },
    { "iso": "ko", "language": "Korean", "flag": "🇰🇷", "hasEdition": true },
    { "iso": "la", "language": "Latin", "flag": "🏛" },
    { "iso": "lv", "language": "Latvian", "flag": "🇱🇻" },
    { "iso": "enm", "language": "Middle English", "flag": "🏰" },
    { "iso": "mn", "language": "Mongolian", "flag": "🇲🇳" },
    { "iso": "mt", "language": "Maltese", "flag": "🇲🇹" },
    { "iso": "nb", "language": "Norwegian Bokmål", "flag": "🇳🇴🏙️" },
    { "iso": "nn", "language": "Norwegian Nynorsk", "flag": "🇳🇴🌲" },
    { "iso": "ang", "language": "Old English", "flag": "🗡️" },
    { "iso": "sga", "language": "Old Irish", "flag": "🍀" },
    { "iso": "fa", "language": "Persian", "flag": "🇮🇷" },
    { "iso": "pl", "language": "Polish", "flag": "🇵🇱", "hasEdition": true },
    { "iso": "pt", "language": "Portuguese", "flag": "🇵🇹", "hasEdition": true },
    { "iso": "ro", "language": "Romanian", "flag": "🇷🇴" },
    { "iso": "ru", "language": "Russian", "flag": "🇷🇺", "hasEdition": true },
    { "iso": "sh", "language": "Serbo-Croatian", "flag": "🇷🇸🇭🇷" },
    { "iso": "scn", "language": "Sicilian", "flag": "🍋" },
    { "iso": "es", "language": "Spanish", "flag": "🇪🇸", "hasEdition": true },
    { "iso": "sv", "language": "Swedish", "flag": "🇸🇪" },
    { "iso": "tl", "language": "Tagalog", "flag": "🇵🇭" },
    { "iso": "th", "language": "Thai", "flag": "🇹🇭", "hasEdition": true },
    { "iso": "tr", "language": "Turkish", "flag": "🇹🇷" },
    { "iso": "uk", "language": "Ukrainian", "flag": "🇺🇦" },
    { "iso": "vi", "language": "Vietnamese", "flag": "🇻🇳" }
]

const allLangs = languages.filter(l => l.language)
const glossLangs = languages.filter(l => l.hasEdition)
const langMap = Object.fromEntries(allLangs.map(l => [l.iso, l]))

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



$(document).ready(function () {
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

})