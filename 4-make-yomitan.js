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
} = /** @type {MakeYomitanEnv} */(process.env);

const latestDownloadLink = 'https://github.com/yomidevs/kaikki-to-yomitan/releases/latest/download/';

const indexJson = {
    format: 3,
    revision: currentDate,
    sequenced: true,
    author: 'Kaikki-to-Yomitan contributors',
    url: 'https://github.com/yomidevs/kaikki-to-yomitan',
    description: 'Dictionaries for various language pairs generated from Wiktionary data, via Kaikki and Kaikki-to-Yomitan.',
    attribution: 'https://kaikki.org/',
    sourceLanguage: source_iso,
    targetLanguage: target_iso,
};

if (!existsSync(`data/language/${source_iso}/${target_iso}`)) {
    mkdirSync(`data/language/${source_iso}/${target_iso}`, {recursive: true});
}

const termDictStyles = readFileSync('data/styles.css', 'utf8');

/** @type {WhitelistedTag[]} */
const targetLanguageTermTags = loadJsonArray(`data/language/target-language-tags/${target_iso}/tag_bank_term.json`);
/** @type {WhitelistedTag[]} */
const languageTermTags = loadJsonArray(`data/language/${source_iso}/${target_iso}/tag_bank_term.json`);
const termTags = [...targetLanguageTermTags, ...languageTermTags];

const targetLanguageIpaTags = loadJsonArray(`data/language/target-language-tags/${target_iso}/tag_bank_ipa.json`);
const languageIpaTags = loadJsonArray(`data/language/${source_iso}/${target_iso}/tag_bank_ipa.json`);
const ipaTags = [...targetLanguageIpaTags, ...languageIpaTags];

const partsOfSpeech = loadJsonArray(`data/language/target-language-tags/${target_iso}/parts_of_speech.json`);

const multiwordInflections = loadJsonArray(`data/language/${source_iso}/${target_iso}/multiword_inflections.json`);
const tagStylesFile = `data/language/target-language-tags/${target_iso}/tag_styles.json`;
const tagStyles = existsSync(tagStylesFile) ? JSON.parse(readFileSync(tagStylesFile, 'utf8')) : {};

const tagModifiers = [
    ['chiefly', 'chief'],
    ['usually', 'usu'],
    ['often', 'oft'],
    ['sometimes', 'some'],
    ['now', 'now'],
    ['especially', 'esp'],
    ['slightly', 'sli'],
]

/**
 * @param {WhitelistedTag[]} tags 
 * @param {string} tag 
 * @returns {null|import('types').TagBank.TagInformation}
 */
function findTag(tags, tag) {
    const fullTag = tags.find((x) => {
        if (typeof x[3] === 'string') {
            return x[3] === tag;
        } else if (Array.isArray(x[3])) {
            return x[3].includes(tag);
        }
        return false;
    });

    if(!fullTag) return null;

    const result = [...fullTag];
    
    if(Array.isArray(result[3])){
        result[3] = result[3][0]; // this makes it fit the yomitan tag format
    }

    return /** @type {import('types').TagBank.TagInformation}*/ (result);
}

/**
 * @param {string} tag 
 * @returns {null|import('types').TagBank.TagInformation}
 */
function findModifiedTag(tag){
    let modifiedTag = null;
    tagModifiers.forEach((modifier) => {
        const regex = new RegExp(`^${modifier[0]} `);
        if (regex.test(tag)){
            const fullTag = findTag(termTags, tag.replace(regex, ''));
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

/**
 * @param {StandardizedExample[]} examples 
 * @returns {import('types').TermBank.StructuredContent[]}
 */
function getStructuredExamples(examples) {
    return examples.map(({text, translation}) => {
        return {
            "tag": "div",
            "data": {
                "content": "extra-info"
            },
            "content": {
                "tag":"div",
                "data": {
                    "content": "example-sentence"
                },
                "content":[{
                    "tag": "div",
                    "data": {
                        "content": "example-sentence-a",
                    },
                    "content": text
                },
                {
                    "tag": "div",
                    "data": {
                        "content": "example-sentence-b"
                    },
                    "content": translation
                }
            ]}
        }
    });
}

/**
 * @param {string} type
 * @param {string} content
 * @returns {import('types').TermBank.StructuredContent}
 */
function buildDetailsEntry(type, content) {
    return {
        "tag": "details",
        "data": {
            "content": `details-entry-${type}`
        },
        "content": [
            {
                "tag": "summary",
                "data": {
                    "content": "summary-entry"
                },
                "content": type
            },
            {
                "tag": "div",
                "data": {
                    "content": `${type}-content`
                },
                "content": content
            }
        ]
    };
}

/**
 * @param {LemmaInfo} info 
 * @returns {import('types').TermBank.StructuredContent}
 */
function getStructuredDetails(info) {
    const result = [];

    const {
        etymology_text: etymology,
        morpheme_text: morphemes,
        head_info_text: headInfo
    } = info;
    
    for (const [title, content] of [
        ['mophemes', morphemes],
        ['etymology', etymology],
        ['head-info', headInfo],
    ]) {
        if (title && content) result.push(buildDetailsEntry(title, content));
    }

    return {
        "tag": "div",
        "data": {
            "content": "details-section"
        },
        "content": [...result]
    };
}

/**
 * @param {GlossTwig} glossTwig
 * @param {string[]} senseTags
 * @param {string} pos
 * @param {number} depth
 * @returns {{nestDefs: import('types').TermBank.StructuredContent[], recognizedTags: string[]}}
 */
function handleLevel(glossTwig, senseTags, pos, depth) {
    /** @type {import('types').TermBank.StructuredContent[]} */
    const nestDefs = [];
    /** @type {string[]} */
    let tags = [];

    for (const [def, children] of glossTwig) {                      
        let processedDef = def;
        
        if(depth === 0 && glossTwig.size === 1){
            const {gloss, recognizedTags} = processGlossTags(def, senseTags, pos);
            processedDef = gloss;
            tags = recognizedTags;
        }

        const examples = children.get('_examples') || [];
        children.delete('_examples');

        const tag = depth === 0 ? 'div' : 'li';

        nestDefs.push({ "tag": tag, "content": [
            processedDef,
            ...getStructuredExamples(examples)
        ] });

        if(children.size > 0) {
            const {nestDefs: childDefs} = handleLevel(children, senseTags, pos, depth + 1);

            nestDefs.push(
                { "tag": "ul", "content": childDefs }
            );
        }
    }

    return {nestDefs, recognizedTags: tags};
}

/**
 * @param {GlossTwig} glossTwig
 * @param {string[]} senseTags
 * @param {string} pos
 * @returns {{glosses: import('types').TermBank.DetailedDefinition[], recognizedTags: string[]}}
 */
function handleNest(glossTwig, senseTags, pos) {
    /** @type {import('types').TermBank.DetailedDefinition[]} */
    const glosses = [];

    const {nestDefs: nestedGloss, recognizedTags} = handleLevel(glossTwig, senseTags, pos, 0);

    if (nestedGloss.length > 0) {
        glosses.push({ "type": "structured-content", "content": nestedGloss });
    }

    return {glosses, recognizedTags};
}

/** @type {FormsMap}  */
const formsMap = new Map();

/**
 * @type {{ipa: Object<string, import('types').TagBank.TagInformation>, dict: Object<string, import('types').TagBank.TagInformation>}}
 */
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
    /** @type {import('types').TermBankMeta.TermPhoneticTranscription[]} */
    const ymtIpa = [];

    consoleOverwrite(`4-make-yomitan.js: reading lemmas...`);
    const lemmasFile = `${readFolder}/${source_iso}-${target_iso}-lemmas.json`;
    /** @type {LemmaDict} */
    const lemmaDict = JSON.parse(readFileSync(path.resolve(__dirname, lemmasFile), 'utf8'), mapJsonReviver);

    consoleOverwrite('4-make-yomitan.js: processing lemmas...');
    for (const [lemma, readings] of Object.entries(lemmaDict)) {
        for (const [reading, partsOfSpeechOfWord] of Object.entries(readings)) {
            const normalizedLemma = normalizeOrthography(lemma);
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
            
            /**
             * @param {any} word 
             */
            function debug(word) {
                if (normalizedLemma === DEBUG_WORD) {
                    console.log('-------------------');
                    console.log(word);
                }
            }

            const ipa = [];

            for (const [pos, etyms] of Object.entries(partsOfSpeechOfWord)) {
                for (const [etym_number, info] of Object.entries(etyms)) {
                    const foundPos = findPartOfSpeech(pos, partsOfSpeech, skippedPartsOfSpeech);
                    const {glossTree} = info;

                    const lemmaTags = [pos];
                    ipa.push(...info.ipa);

                    /** @type {Object<string, import('types').TermBank.TermInformation>} */
                    const entries = {};

                    for (const [gloss, branches] of glossTree.entries()) {
                        const tags = branches.get('_tags') || [];
                        branches.delete('_tags');

                        const senseTags = [...tags, ...lemmaTags];

                        /** @type {GlossBranch} */
                        const syntheticBranch = new Map();
                        syntheticBranch.set(gloss, branches);
                        const {glosses, recognizedTags} = handleNest(syntheticBranch, senseTags, pos);
                        const joinedTags = recognizedTags.join(' ');
                        
                        if(!glosses || !glosses.length) continue;

                        if (entries[joinedTags]) {
                            // entries[joinedTags][5].push(gloss);
                            entries[joinedTags][5].push(...glosses);
                        } else {
                            entries[joinedTags] = [
                                term, // term
                                reading !== normalizedLemma ? reading : '', // reading
                                joinedTags, // definition_tags
                                foundPos, // rules
                                0, // frequency
                                glosses, // definitions
                                0, // sequence
                                '', // term_tags
                            ];
                        }
                    }

                    debug(entries);
                    for (const [tags, entry] of Object.entries(entries)) {
                        if (info.etymology_text || info.head_info_text || info.morpheme_text) {
                            const lastDef = entry[5][entry[5].length - 1];

                            if (
                                lastDef &&
                                typeof lastDef === 'object' &&
                                'type' in lastDef &&
                                lastDef.type === 'structured-content' &&
                                Array.isArray(lastDef.content)
                            ) {
                                lastDef.content.push(getStructuredDetails(info));
                            }
                        }

                        ymtLemmas.push(entry);
                    }
                }
            }

            /** @type {{ ipa: string; tags?: string[]; }[]} */
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
                    existingIpa.tags = [
                        ...new Set([
                            ...(existingIpa.tags || []),
                            ...item.tags])
                    ];
                } else {
                    result.push(item);
                }
                return result;
            }, /** @type {{ ipa: string; tags?: string[]; }[]} */ ([]));

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
    writeStyles('dict', dictTagStyles);
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
    /** @type {CondensedFormEntries} */
    let ymtFormData = [];
    let formCounter = 0;

    consoleOverwrite('4-make-yomitan.js: Processing forms...');
    const formsFiles = readdirSync(readFolder).filter((file) => file.startsWith(`${source_iso}-${target_iso}-forms-`));
    for (const file of formsFiles) {
        /** @type {FormsMap} */
        const formsPart = JSON.parse(readFileSync(path.resolve(__dirname, readFolder, file), 'utf8'), mapJsonReviver);
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
                            hypotheses = gloss
                            .split(' and ')
                            .map((hypothesis) => hypothesis.split(' '));
                        }

                        if(target_iso === 'fr'){
                            hypotheses = hypotheses.map((hypothesis) => 
                                hypothesis.filter(inflection => !inflection.trim().startsWith('Voir la conjugaison'))
                            );
                        }

                        hypotheses = hypotheses
                            .map((hypothesis) => 
                                hypothesis
                                    .map((inflection) => inflection.trim())
                                    .filter(Boolean)
                            )
                            .filter(hypothesis => hypothesis.length)
                            .map((hypothesis) => 
                                hypothesis.map((inflection) => 
                                    inflection.replace(/\u00A0/g, ' ')
                                )
                            );


                        return hypotheses;
                    });

                    /** @type {string[][]} */
                    const uniqueHypotheses = [];

                    for (const hypothesis of inflectionHypotheses) {
                        const hypothesisStrings = uniqueHypotheses.map((hypothesis) => sortTags(target_iso, hypothesis).join(' '));
                        const hypothesisString = sortTags(target_iso, hypothesis).join(' ');
                        if (!hypothesisStrings.includes(hypothesisString)) {
                            uniqueHypotheses.push(hypothesis);
                        }
                    }

                    /** @type {[ uninflectedTerm: string, inflectionRules: string[]][]} */
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

/**
 * @param {string} gloss 
 * @param {string[]} senseTags 
 * @param {string} pos 
 * @returns {{gloss: string, recognizedTags: string[]}}
 */
function processGlossTags(gloss, senseTags, pos) {
    const regex = /^\(([^()]+)\) ?/;
    /** @type {string[]} */
    let parenthesesTags = [];

    if (typeof gloss === 'string') {
        const parenthesesContent = gloss.match(regex)?.[1];

        parenthesesTags = parenthesesContent
            ? parenthesesContent.replace(/ or /g, ', ').split(', ').filter(Boolean)
            : [];
    }

    const { leftoverTags, recognizedTags } = processTags(senseTags, parenthesesTags, pos);

    gloss = gloss.replace(regex, leftoverTags);

    return {gloss, recognizedTags};
}

/**
 * @param {CondensedFormEntries} ymtFormData 
 * @returns {CondensedFormEntries}
 */
function writeYmtFormData(ymtFormData) {
    /** @type {import('types').TermBank.TermInformation[]} */
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

/**
 * @param {string} folder 
 * @param {import('types').TermBank.DictionaryTermBankV3 | import('types').TermBankMeta.DictionaryTermMetaBankV3} data 
 * @param {number} bankIndex
 * @returns 
 */
function writeBanks(folder, data, bankIndex = 0) {
    if(folder === 'form') folder = 'dict';

    if(bankIndex === 0) {
        for (const file of readdirSync(`${writeFolder}/${folder}`)) {
            if (file.includes('term_')) { unlinkSync(`${writeFolder}/${folder}/${file}`); }
        }
    }

    const filename = folder === 'ipa' ? 'term_meta_bank_' : 'term_bank_';

    return writeInBatches(writeFolder, data, `${folder}/${filename}`, 25000, bankIndex);
}

/**
 * @param {'dict'|'ipa'} folder 
 */
function writeTags(folder) {
    writeFileSync(`${writeFolder}/${folder}/tag_bank_1.json`, JSON.stringify(Object.values(ymtTags[folder])));
}

/**
 * @param {'dict'|'ipa'} folder 
 * @param {string} styles 
 */
function writeStyles(folder, styles){
    if(folder === 'dict') {
        styles = styles + '\n' + termDictStyles;
    }
    if(!styles) return;
    writeFileSync(`${writeFolder}/${folder}/styles.css`, styles);
}

/**
 * @param {'dict'|'ipa'} folder 
 * @returns {string}
 */
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

/**
 * @param {'dict'|'ipa'} folder 
 */
function writeIndex(folder) {
    const title = `${DICT_NAME}-${source_iso}-${target_iso}` + (folder === 'dict' ? '' : '-ipa');
    writeFileSync(`${writeFolder}/${folder}/index.json`, JSON.stringify({
        ...indexJson,
        title,
        isUpdatable: true,
        indexUrl: `${latestDownloadLink}${title}-index.json`,
        downloadUrl: `${latestDownloadLink}${title}.zip`,
    }));        
}

/**
 * @param {string[]} senseTags 
 * @param {string[]} parenthesesTags 
 * @param {string} pos 
 * @returns 
 */
function processTags(senseTags, parenthesesTags, pos) {
    /** @type {string[]} */
    let recognizedTags = [];

    const allEntryTags = [...new Set([...senseTags, ...parenthesesTags])];
    termTagCount += allEntryTags.length;

    const unrecognizedTags = allEntryTags
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
    
    const leftoverTags = unrecognizedTags.length ? `(${unrecognizedTags.join(', ')}) ` : '';
    recognizedTags = [...new Set(recognizedTags)];

    return { leftoverTags, recognizedTags };
}

/**
 * @param {*} obj 
 * @returns 
 */
function sortBreakdown(obj){
    return Object.fromEntries(Object.entries(obj).sort((a, b) => b[1] - a[1]));
}

/**
 * @param {string} term 
 * @returns {string}
 */
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
        case 'it':
            return term.normalize('NFD').replace(/[\u0300-\u036f]/g, '');
        case 'tl':
            return term.normalize('NFD').replace(/[\u0300-\u036f\-']/g, '');
        case 'sh':
            return term.normalize('NFD').replace(/[aeiourAEIOUR][\u0300-\u036f]/g, (match) => match[0]);
        case 'uk':
        case 'ru':
            return term.replace(/́/g, '');
        default:
            return term;
    }
}

