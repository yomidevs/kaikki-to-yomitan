const {readFileSync, writeFileSync, existsSync, readdirSync, mkdirSync, createWriteStream, unlinkSync} = require('fs');
const path = require('path');
const date = require('date-and-time');
const now = new Date();
const currentDate = date.format(now, 'YYYY.MM.DD');

const { sortTags, writeInBatches, consoleOverwrite } = require('./util/util');

const {
    source_iso, 
    target_iso, 
    DEBUG_WORD, 
    DICT_NAME,
    tidy_folder: readFolder,
    temp_folder: writeFolder
} = process.env;

consoleOverwrite(`4-make-yomitan.js: reading lemmas...`);
const lemmasFile = `${readFolder}/${source_iso}-${target_iso}-lemmas.json`;
const lemmaDict = JSON.parse(readFileSync(path.resolve(__dirname, lemmasFile)));
consoleOverwrite(`4-make-yomitan.js: reading forms...`);
const formsFile = `${readFolder}/${source_iso}-${target_iso}-forms.json`;
const formDict = JSON.parse(readFileSync(path.resolve(__dirname, formsFile)));

if (!existsSync(`data/language/${source_iso}/${target_iso}`)) {
    mkdirSync(`data/language/${source_iso}/${target_iso}`, {recursive: true});
}

function loadJsonArray(file) {
    return existsSync(file) ? JSON.parse(readFileSync(file)) : [];
}

const targetLanguageTermTags = loadJsonArray(`data/language/target-language-tags/${target_iso}/tag_bank_term.json`);
const languageTermTags = loadJsonArray(`data/language/${source_iso}/${target_iso}/tag_bank_term.json`);
const termTags = [...targetLanguageTermTags, ...languageTermTags];

const targetLanguageIpaTags = loadJsonArray(`data/language/target-language-tags/${target_iso}/tag_bank_ipa.json`);
const languageIpaTags = loadJsonArray(`data/language/${source_iso}/${target_iso}/tag_bank_ipa.json`);
const ipaTags = [...targetLanguageIpaTags, ...languageIpaTags];

const partsOfSpeech = loadJsonArray(`data/language/target-language-tags/${target_iso}/parts_of_speech.json`);

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

function findPartOfSpeech(pos) {
    for(const posAliases of partsOfSpeech){
        if (posAliases.includes(pos)){
            return posAliases[0];
        }
    }
    incrementCounter(pos, skippedPartsOfSpeech);
    return pos;
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

const ymt = {
    lemma: [],
    form: [],
    ipa: [],
    dict: []
};

const ymtTags = {
    ipa: {},
    dict: {}
};

const skippedIpaTags = {};
const skippedTermTags = {};
const skippedPartsOfSpeech = {};

let ipaCount = 0;
let termTagCount = 0;

consoleOverwrite('4-make-yomitan.js: processing lemmas...');
for (const [lemma, infoMap] of Object.entries(lemmaDict)) {
    normalizedLemma = normalizeOrthography(lemma);
    
    function debug(word) {
        if (normalizedLemma === DEBUG_WORD) {
            console.log('-------------------');
            console.log(word);
        }
    }

    const ipa = [];

    for (const [pos, info] of Object.entries(infoMap)) {
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
                    if (entries[joinedTags]) {
                        entries[joinedTags][5].push(gloss);
                    } else if (gloss) {
                        entries[joinedTags] = [
                            normalizedLemma, // term
                            normalizedLemma, // reading
                            joinedTags, // definition_tags
                            findPartOfSpeech(pos), // rules
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
            ymt.lemma.push(entry);
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
        ymt.ipa.push([
            normalizedLemma,
            'ipa',
            {
                reading: normalizedLemma,
                transcriptions: mergedIpas
            }
        ]);
    }
}

const multiwordInflections = [
    'subjunctive I', // de
    'subjunctive II', // de
    'Archaic form', // de
    'archaic form', // de
    'female equivalent', // de
];

consoleOverwrite('4-make-yomitan.js: Processing forms...');
for (const [form, allInfo] of Object.entries(formDict)) {
    for (const [lemma, info] of Object.entries(allInfo)) {
        for (const [pos, glosses] of Object.entries(info)) {
            const inflectionHypotheses = glosses.flatMap((gloss) => {
                if (!gloss) { return []; }

                gloss = gloss
                    .replace(/-automated- /g, '')
                
                if(target_iso === 'en'){
                    gloss = gloss
                        .replace(/multiword-construction /g, '')
                        .replace(new RegExp(`of ${escapeRegExp(lemma)}.*$`), '');

                    for (const multiwordInflection of multiwordInflections) {
                        gloss = gloss.replace(new RegExp(multiwordInflection), multiwordInflection.replace(' ', '-'));
                    }
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

            uniqueHypotheses = [];

            for (const hypothesis of inflectionHypotheses) {
                const hypothesisStrings = uniqueHypotheses.map((hypothesis) => sortTags(target_iso, hypothesis).join(' '));
                const hypothesisString = sortTags(target_iso, hypothesis).join(' ');
                if (!hypothesisStrings.includes(hypothesisString)) {
                    uniqueHypotheses.push(hypothesis);
                }
            }

            const deinflectionDefinitions = uniqueHypotheses.map((hypothesis) => [
                normalizeOrthography(lemma),
                hypothesis
            ]);

            if(deinflectionDefinitions.length > 0){
                ymt.form.push([
                    normalizeOrthography(form),
                    '',
                    'non-lemma',
                    '',
                    0,
                    deinflectionDefinitions,
                    0,
                    ''
                ]);
            }
        }
    }
}

ymt.dict = [...ymt.lemma, ...ymt.form];

const indexJson = {
    format: 3,
    revision: currentDate,
    sequenced: true
};

const folders = ['dict', 'ipa'];

for (const folder of folders) {
    consoleOverwrite(`4-make-yomitan.js: Writing ${folder}...`);
    for (const file of readdirSync(`${writeFolder}/${folder}`)) {
        if (file.includes('term_')) { unlinkSync(`${writeFolder}/${folder}/${file}`); }
    }

    writeFileSync(`${writeFolder}/${folder}/index.json`, JSON.stringify({
        ...indexJson,
        title: `${DICT_NAME}-${source_iso}-${target_iso}` + (folder === 'dict' ? '' : '-ipa'),
    }));

    writeFileSync(`${writeFolder}/${folder}/tag_bank_1.json`, JSON.stringify(Object.values(ymtTags[folder])));

    const filename = folder === 'dict' ? 'term_bank_' : 'term_meta_bank_';

    writeInBatches(writeFolder, ymt[folder], `${folder}/${filename}`, 25000);
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

function escapeRegExp(text) {
    return text.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, '\\$&');
}

function sortBreakdown(obj){
    return Object.fromEntries(Object.entries(obj).sort((a, b) => b[1] - a[1]));
}

function incrementCounter(key, counter) {
    counter[key] = (counter[key] || 0) + 1;
}

function normalizeOrthography(term) {
    switch (source_iso) {
        case 'ar':
            return term
                .replace(/[\u064E-\u0650]/g, '');
        case 'la':
            const diacriticMap = {
                'ā': 'a', 'ē': 'e', 'ī': 'i', 'ō': 'o', 'ū': 'u', 'ȳ': 'y',
                'Ā': 'A', 'Ē': 'E', 'Ī': 'I', 'Ō': 'O', 'Ū': 'U', 'Ȳ': 'Y',
                'á': 'a', 'é': 'e', 'í': 'i', 'ó': 'o', 'ú': 'u', 'ý': 'y',
                'Á': 'A', 'É': 'E', 'Í': 'I', 'Ó': 'O', 'Ú': 'U', 'Ý': 'Y'
            };
            return term.replace(/[āēīōūȳáéíóúýĀĒĪŌŪȲÁÉÍÓÚÝ]/g, (match) => diacriticMap[match] || match);
        case 'ru':
            return term.replace(/́/g, '');
        default:
            return term;
    }
}

