const path = require('path');
const { readFileSync, writeFileSync, existsSync, readdirSync, mkdirSync, unlinkSync } = require('fs');
const { sortTags, writeInBatches, consoleOverwrite, 
    mapJsonReviver, logProgress, loadJsonArray, 
    findPartOfSpeech, incrementCounter, currentDate } = require('./util/util');

const {
    source_iso,
    target_iso,
    DEBUG_WORD,
    DICT_NAME,
    tidy_folder: readFolder,
    temp_folder: writeFolder
} = process.env;

const indexJson = {
    format: 3,
    revision: currentDate,
    sequenced: true,
    author: 'Kaikki-to-Yomitan contributors',
    url: 'https://github.com/themoeway/kaikki-to-yomitan',
    description: 'Dictionaries for various language pairs generated from Wiktionary data, via Kaikki and Kaikki-to-Yomitan.',
    attribution: 'https://kaikki.org/',
    sourceLanguage: source_iso,
    targetLanguage: target_iso,
};

if (!existsSync(`data/language/${source_iso}/${target_iso}`)) {
    mkdirSync(`data/language/${source_iso}/${target_iso}`, {recursive: true});
}

const targetLanguageTermTags = loadJsonArray(`data/language/target-language-tags/${target_iso}/tag_bank_term.json`);
const languageTermTags = loadJsonArray(`data/language/${source_iso}/${target_iso}/tag_bank_term.json`);
const termTags = [...targetLanguageTermTags, ...languageTermTags];

const targetLanguageIpaTags = loadJsonArray(`data/language/target-language-tags/${target_iso}/tag_bank_ipa.json`);
const languageIpaTags = loadJsonArray(`data/language/${source_iso}/${target_iso}/tag_bank_ipa.json`);
const ipaTags = [...targetLanguageIpaTags, ...languageIpaTags];

const partsOfSpeech = loadJsonArray(`data/language/target-language-tags/${target_iso}/parts_of_speech.json`);

const multiwordInflections = loadJsonArray(`data/language/${source_iso}/${target_iso}/multiword_inflections.json`);
const tagStylesFile = `data/language/target-language-tags/${target_iso}/tag_styles.json`;
const tagStyles = existsSync(tagStylesFile) ? JSON.parse(readFileSync(tagStylesFile)) : {};

const tagModifiers = [
    ['chiefly', 'chief'],
    ['usually', 'usu'],
    ['often', 'oft'],
    ['sometimes', 'some'],
    ['now', 'now'],
    ['especially', 'esp'],
    ['slightly', 'sli'],
]

function findTag(tags, tag) {
    const fullTag = tags.find((x) => {
        if (typeof x[3] === 'string') {
            return x[3] === tag;
        } else if (Array.isArray(x[3])) {
            return x[3].includes(tag);
        }
        return false;
    });

    const result = fullTag ? [...fullTag] : null;
    
    if(result && Array.isArray(result[3])){
        result[3] = result[3][0];
    }

    return result;
}

function findModifiedTag(tag){
    let modifiedTag = null;
    tagModifiers.forEach((modifier) => {
        const regex = new RegExp(`^${modifier[0]} `);
        if (regex.test(tag)){
            fullTag = findTag(termTags, tag.replace(regex, ''));
            if (fullTag){
                modifiedTag = [
                    `${modifier[1]}-${fullTag[0]}`,
                    fullTag[1],
                    fullTag[2],
                    `${modifier[0]} ${fullTag[3]}`,
                    fullTag[4]
                ]
            }
        }
    })

    return modifiedTag;
}

const formsMap = new Map();

const ymtTags = {
    ipa: {},
    dict: {}
};

const skippedIpaTags = {};
const skippedTermTags = {};
const skippedPartsOfSpeech = {};

let ipaCount = 0;
let termTagCount = 0;

let lastTermBankIndex = 0;

{
    const ymtLemmas = [];
    const ymtIpa = [];

    consoleOverwrite(`4-make-yomitan.js: reading lemmas...`);
    const lemmasFile = `${readFolder}/${source_iso}-${target_iso}-lemmas.json`;
    const lemmaDict = JSON.parse(readFileSync(path.resolve(__dirname, lemmasFile)));

    consoleOverwrite('4-make-yomitan.js: processing lemmas...');
    for (const [lemma, readings] of Object.entries(lemmaDict)) {
        for (const [reading, partsOfSpeechOfWord] of Object.entries(readings)) {
            normalizedLemma = normalizeOrthography(lemma);
            let term = normalizedLemma;

            if(lemma !== normalizedLemma && lemma !== reading){
                term = lemma;
                const lemmaForms = formsMap.get(lemma) || new Map();
                const formPOSs = lemmaForms.get(normalizedLemma) || new Map();
                const anyForms = formPOSs.get("any") || [];
                formPOSs.set("any", anyForms);
                lemmaForms.set(normalizedLemma, formPOSs);
                formsMap.set(lemma, lemmaForms);

                const message = `${normalizedLemma}\u00A0≈\u00A0${lemma}`;
                if (!anyForms.includes(message)){
                    anyForms.push(message);
                }
            }

            function debug(word) {
                if (normalizedLemma === DEBUG_WORD) {
                    console.log('-------------------');
                    console.log(word);
                }
            }

            const ipa = [];

            for (const [pos, info] of Object.entries(partsOfSpeechOfWord)) {
                const {senses} = info;

                const lemmaTags = [pos, ...(info.tags || [])];
                ipa.push(...info.ipa);
                const entries = {};

                for (const sense of senses) {

                    const {glosses, tags} = sense;
                    const senseTags = [...lemmaTags, ...tags]

                    glosses.forEach((gloss) => {
                        debug(gloss);

                        function addGlossToEntries(joinedTags) {
                            if(!gloss) return;
                            if (entries[joinedTags]) {
                                entries[joinedTags][5].push(gloss);
                            } else {
                                entries[joinedTags] = [
                                    term, // term
                                    reading !== normalizedLemma ? reading : '', // reading
                                    joinedTags, // definition_tags
                                    findPartOfSpeech(pos, partsOfSpeech, skippedPartsOfSpeech), // rules
                                    0, // frequency
                                    [gloss], // definitions
                                    0, // sequence
                                    '', // term_tags
                                ];
                            }
                        }

                        if (typeof gloss !== 'string') { 
                            const { leftoverTags, recognizedTags } = processTags(lemmaTags, senseTags, [], pos);
                            addGlossToEntries(recognizedTags.join(' '));
                            return; 
                        }

                        const regex = /^\(([^()]+)\) ?/;
                        const parenthesesContent = gloss.match(regex)?.[1];

                        const parenthesesTags = parenthesesContent
                            ? parenthesesContent.replace(/ or /g, ', ').split(', ').filter(Boolean)
                            : [];

                        const { leftoverTags, recognizedTags } = processTags(lemmaTags, senseTags, parenthesesTags, pos);

                        gloss = gloss.replace(regex, leftoverTags);

                        addGlossToEntries(recognizedTags.join(' '));
                    });
                    
                }

                debug(entries);
                for (const [tags, entry] of Object.entries(entries)) {
                    ymtLemmas.push(entry);
                }
            }

            const mergedIpas = ipa.reduce((result, item) => {
                ipaCount++;
                item.tags = item.tags
                    .map((tag) => {
                        const fullTag = findTag(ipaTags, tag);
                        if (fullTag){
                            ymtTags.ipa[tag] = fullTag;
                            return fullTag[0];
                        } else {
                            incrementCounter(tag, skippedIpaTags)
                            return tag;
                        }
                    })

                const existingIpa = result.find((x) => x.ipa === item.ipa);

                if (existingIpa) {
                    existingIpa.tags = [...new Set([...existingIpa.tags, ...item.tags])];
                } else {
                    result.push(item);
                }
                return result;
            }, []);

            if (mergedIpas.length) {
                ymtIpa.push([
                    term,
                    'ipa',
                    {
                        reading,
                        transcriptions: mergedIpas
                    }
                ]);
            }
            delete readings[reading];
        }
        delete lemmaDict[lemma];
    }

    writeIndex('dict');
    writeTags('dict');
    const dictTagStyles = getTagStyles('dict');
    if(dictTagStyles){
        writeStyles('dict', dictTagStyles);
    }
    lastTermBankIndex = writeBanks('dict', ymtLemmas, lastTermBankIndex);
    writeIndex('ipa');
    writeTags('ipa');
    const ipaTagStyles = getTagStyles('ipa');
    if(ipaTagStyles){
        writeStyles('ipa', ipaTagStyles);
    }

    writeBanks('ipa', ymtIpa);
}

{
    let ymtFormData = [];
    let formCounter = 0;

    consoleOverwrite('4-make-yomitan.js: Processing forms...');
    const formsFiles = readdirSync(readFolder).filter((file) => file.startsWith(`${source_iso}-${target_iso}-forms-`));
    for (const file of formsFiles) {
        const formsPart = JSON.parse(readFileSync(path.resolve(__dirname, readFolder, file)), mapJsonReviver);
        for (const [lemma, forms] of formsPart.entries()) {
            formsMap.set(lemma, forms);
        }
    
        for(const [lemma, forms] of formsMap.entries()){
            logProgress('Processing forms...', formCounter, undefined, 100);
            formCounter++;
            for (const [form, POSs] of forms.entries()) {
                for (const [pos, glosses] of POSs.entries()) {
                    const inflectionHypotheses = glosses.flatMap((gloss) => {
                        if (!gloss) { return []; }

                        gloss = gloss
                            .replace(/multiword-construction /g, '')

                        for (const multiwordInflection of multiwordInflections) {
                            gloss = gloss.replace(new RegExp(multiwordInflection), multiwordInflection.replace(/ /g, '\u00A0'));
                        }

                        // TODO: decide on format for de-de
                        // if(target_iso === 'de'){
                        //     gloss = gloss
                        //         .replace(/^\s*\[\d\]\s*/g, '')
                        // }
                        
                        let hypotheses = [[gloss]];

                        // TODO: generalize this
                        if(target_iso === 'en'){
                            hypotheses = gloss.split(' and ') 
                            hypotheses = hypotheses.map((hypothesis) => hypothesis.split(' '));
                        }

                        if(target_iso === 'fr'){
                            hypotheses = hypotheses.map((hypothesis) => 
                                hypothesis.filter(inflection => !inflection.trim().startsWith('Voir la conjugaison'))
                            );
                        }

                        hypotheses = hypotheses
                            .map((hypothesis) => 
                                hypothesis
                                    .map((inflection) => (inflection).trim())
                                    .filter(Boolean)
                            ).filter(hypothesis => hypothesis.length);

                        return hypotheses;
                    });

                    const uniqueHypotheses = [];

                    for (const hypothesis of inflectionHypotheses) {
                        const hypothesisStrings = uniqueHypotheses.map((hypothesis) => sortTags(target_iso, hypothesis).join(' '));
                        const hypothesisString = sortTags(target_iso, hypothesis).join(' ');
                        if (!hypothesisStrings.includes(hypothesisString)) {
                            uniqueHypotheses.push(hypothesis);
                        }
                    }

                    const deinflectionDefinitions = uniqueHypotheses.map((hypothesis) => [
                        lemma,
                        hypothesis
                    ]);

                    if(deinflectionDefinitions.length > 0){
                        ymtFormData.push([
                            normalizeOrthography(form),
                            form !== normalizeOrthography(form) ? form : '',
                            // 'non-lemma',
                            // '',
                            // 0,
                            deinflectionDefinitions,
                            // 0,
                            // ''
                        ]);
                    }
                    POSs.delete(pos);
                }
            }

            formsMap.delete(lemma);

            const chunkSize = 20000;
            if(ymtFormData.length > chunkSize){
                ymtFormData = writeYmtFormData(ymtFormData);
            }
        }
    }
    if(ymtFormData.length){
        writeYmtFormData(ymtFormData);
    }
}

console.log('');
console.log(
    'total ipas',
    ipaCount,
    'skipped ipa tags',
    Object.values(skippedIpaTags).reduce((a, b) => a + b, 0),
    'total term tags',
    termTagCount,
    'skipped term tags',
    Object.values(skippedTermTags).reduce((a, b) => a + (parseInt(b) || 0), 0))
;
writeFileSync(`data/language/${source_iso}/${target_iso}/skippedIpaTags.json`, JSON.stringify(sortBreakdown(skippedIpaTags), null, 2));

writeFileSync(`data/language/${source_iso}/${target_iso}/skippedTermTags.json`, JSON.stringify(sortBreakdown(skippedTermTags), null, 2));

writeFileSync(`data/language/${source_iso}/${target_iso}/skippedPartsOfSpeech.json`, JSON.stringify(sortBreakdown(skippedPartsOfSpeech), null, 2));

console.log('4-make-yomitan.js: Done!')

function writeYmtFormData(ymtFormData) {
    const ymtForms = ymtFormData.map((form, index) => {
        const [term, reading, definitions] = form;
        return [
            term,
            reading,
            'non-lemma',
            '',
            0,
            definitions,
            0,
            ''
        ];
    });

    lastTermBankIndex = writeBanks('form', ymtForms, lastTermBankIndex);
    ymtFormData = [];
    return ymtFormData;
}

function writeBanks(folder, data, bankIndex = 0) {
    if(folder === 'form') folder = 'dict';

    if(bankIndex === 0){
        for (const file of readdirSync(`${writeFolder}/${folder}`)) {
            if (file.includes('term_')) { unlinkSync(`${writeFolder}/${folder}/${file}`); }
        }
    }

    const filename = folder === 'ipa' ? 'term_meta_bank_' : 'term_bank_';

    return writeInBatches(writeFolder, data, `${folder}/${filename}`, 25000, bankIndex);
}

function writeTags(folder) {
    writeFileSync(`${writeFolder}/${folder}/tag_bank_1.json`, JSON.stringify(Object.values(ymtTags[folder])));
}

function writeStyles(folder, tagStyles){
    writeFileSync(`${writeFolder}/${folder}/styles.css`, tagStyles);
}

function getTagStyles(folder){
    let styles = "";
    for (const fullTag of Object.values(ymtTags[folder])) {
        const tag = fullTag[0];
        if (tagStyles[tag]) {
            styles += tagStyles[tag] + '\n';
        }
    }
    return styles;
}

function writeIndex(folder) {
    writeFileSync(`${writeFolder}/${folder}/index.json`, JSON.stringify({
        ...indexJson,
        title: `${DICT_NAME}-${source_iso}-${target_iso}` + (folder === 'dict' ? '' : '-ipa'),
    }));
}

function processTags(lemmaTags, senseTags, parenthesesTags, pos) {
    let recognizedTags = [];

    const allEntryTags = [...new Set([...lemmaTags, ...senseTags, ...parenthesesTags])];
    termTagCount += allEntryTags.length;

    unrecognizedTags = allEntryTags
        .map((tag) => {
            const fullTag = findTag(termTags, tag);

            if (fullTag) {
                recognizedTags.push(fullTag[0]);
                ymtTags.dict[tag] = fullTag;
                return null;
            } else {
                const modifiedTag = findModifiedTag(tag);
                if (modifiedTag) {
                    recognizedTags.push(modifiedTag[0]);
                    ymtTags.dict[tag] = modifiedTag;
                } else {
                    if (allEntryTags.some((otherTag) => otherTag !== tag && otherTag.includes(tag))) return null;
                    incrementCounter(tag, skippedTermTags);
                    if (tag === pos) incrementCounter("pos-" + tag, skippedTermTags);
                    if (parenthesesTags.includes(tag)) return tag;
                }
            }
        })
        .filter(Boolean);
    
    leftoverTags = unrecognizedTags.length ? `(${unrecognizedTags.join(', ')}) ` : '';
    recognizedTags = [...new Set(recognizedTags)];

    return { leftoverTags, recognizedTags };
}

function sortBreakdown(obj){
    return Object.fromEntries(Object.entries(obj).sort((a, b) => b[1] - a[1]));
}

function normalizeOrthography(term) {
    switch (source_iso) {
        case 'ar':
        case 'fa':
            const optionalDiacritics = [
                '\u0618', // Small Fatha
                '\u0619', // Small Damma
                '\u061A', // Small Kasra
                '\u064B', // Fathatan
                '\u064C', // Dammatan
                '\u064D', // Kasratan
                '\u064E', // Fatha
                '\u064F', // Damma
                '\u0650', // Kasra
                '\u0651', // Shadda
                '\u0652', // Sukun
                '\u0653', // Maddah
                '\u0654', // Hamza Above
                '\u0655', // Hamza Below
                '\u0656', // Subscript Alef
                '\u0670', // Dagger Alef
            ];
            
            const diacriticsRegex = new RegExp(`[${optionalDiacritics.join('')}]`, 'g');
            
            return term.replace(diacriticsRegex, '')
        case 'la':
        case 'ang':
        case 'sga':
        case 'grc':
        case 'ro':
        case 'tl':
            return term.normalize('NFD').replace(/[\u0300-\u036f]/g, '');
        case 'sh':
            return term.normalize('NFD').replace(/[aeiourAEIOUR][\u0300-\u036f]/g, (match) => match[0]);
        case 'uk':
        case 'ru':
            return term.replace(/́/g, '');
        default:
            return term;
    }
}

