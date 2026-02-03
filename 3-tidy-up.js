const { writeFileSync, readdirSync, unlinkSync } = require('fs');

const LineByLineReader = require('line-by-line');

const { 
    source_iso: sourceIso,
    target_iso: targetIso,
    kaikki_file: kaikkiFile,
    tidy_folder: writeFolder
} = /** @type {TidyEnv} */ (process.env);

const { sortTags, similarSort, mergePersonTags, consoleOverwrite, clearConsoleLine, logProgress, mapJsonReplacer } = require('./util/util');

/** @type {LemmaDict} */
const lemmaDict = {};

/** @type {FormsMap} */
const formsMap = new Map();

/** @type {AutomatedForms} */
const automatedForms = new Map();

/**
 * @param {string} string
 * @returns {string}
*/
function escapeRegExp(string) {
    return string.replace(/[.*+\-?^${}()|[\]\\]/g, '\\$&');
}

/**
 * @param {string[]} glosses 
 * @param {FormOf[]|undefined} formOf 
 * @returns {boolean}
 */
function isInflectionGloss(glosses, formOf) {
    const glossesString = JSON.stringify(glosses);
    switch (targetIso) {
        case 'de':
            if (glosses.some(gloss => /des (?:Verbs|Adjektivs|Substantivs|Demonstrativpronomens|Possessivpronomens|Pronomens)/.test(gloss))) return true;
        case 'en':
            if (glosses.some(gloss => /.*inflection of.*/.test(gloss))) return true;
            if(!Array.isArray(formOf)) return false;
            for (const {word: lemma} of formOf) {
                if(!lemma) continue;
                if (glosses.some(gloss => new RegExp(`of ${escapeRegExp(lemma)}($| \(.+?\)$)`).test(gloss))) return true;
            }
        case 'el':
            if (!Array.isArray(formOf)) return false;
            for (const { word: lemma } of formOf) {
                if (!lemma) continue;
                if (glosses.some(gloss => /του/.test(gloss))) return true;
            }
        case 'fr':
            if (/.*du verbe\s+((?:(?!\bdu\b).)*)$/.test(glossesString)) return true;
            if (/((?:(?:Masculin|Féminin)\s)?(?:(?:p|P)luriel|(?:s|S)ingulier)) de ([^\s]+)/.test(glossesString)) return true;
    }
    return false;
}

/**
 * @param {string} form 
 * @param {string} pos 
 * @param {string} lemma 
 * @param {string[]|Set<string>} inflections 
 */
function addDeinflections(form, pos, lemma, inflections) {
    const {inflected, uninflected} = normalizeInflectionPair(form, lemma);
    if (inflected === uninflected) return;

    const lemmaForms = formsMap.get(uninflected) || /** @type {Map<Form, Map<PoS, string[]>>} */ (new Map());
    formsMap.set(uninflected, lemmaForms);
    const formPOSs = lemmaForms.get(inflected) || /** @type {Map<PoS, string[]>} */ (new Map());
    lemmaForms.set(inflected, formPOSs);
    formPOSs.get(pos) || formPOSs.set(pos, []);

    try {
        const inflectionsSet = new Set(formPOSs.get(pos));
        for (const inflection of inflections) {
            inflectionsSet.add(inflection);
        }
    
        formPOSs.set(pos, Array.from(inflectionsSet));
    } catch(e) {
        console.log(e);
    }
}

const blacklistedTags = [
    'inflection-template',
    'table-tags',
    'canonical',
    'class',
    'error-unknown-tag',
    'error-unrecognized-form',
    'includes-article',
    'obsolete',
    'archaic',
    'used-in-the-form',
    'romanization',
    'dated',
    'auxiliary'
];

const identityTags = [
    'nominative',
    'singular',
    'infinitive',
]

const redundantTags = [
    'multiword-construction',
    'combined-form'
];

let lineCount = 0;
consoleOverwrite(`3-tidy-up.js started...`);

const lr = new LineByLineReader(kaikkiFile);

lr.on('line', (line) => {
    if (line) {
        lineCount += 1;
        logProgress("Processing lines", lineCount);
        handleLine(JSON.parse(line));
    }
});

/**
 * @param {KaikkiLine} parsedLine 
 */
function handleLine(parsedLine) {
    const { pos, sounds, forms, etymology_number = 0, etymology_text, word: backlink} = parsedLine;
    if(!pos) return;
    const word = getCanonicalWordForm(parsedLine);
    if (!word) return;
    
    processForms(forms, word, pos);

    const {senses, head_templates, tags} = parsedLine;
    if (!senses) return;
    
    const ipa = getPhoneticTranscriptions(sounds);
    
    /** @type {TidySense[]} */
    const sensesWithGlosses = /** @type {TidySense[]} */ (senses
        .filter(sense => sense.glosses || sense.raw_glosses || sense.raw_gloss)
        .map(sense => {
        const glosses = sense.raw_glosses || sense.raw_gloss || sense.glosses;
        const glossesArray = Array.isArray(glosses) ? glosses : [glosses];

        const glossTags = sense.tags || [];
        if(sense.raw_tags && Array.isArray(sense.raw_tags)) {
            glossTags.push(...sense.raw_tags);
        }
        if (['ru', 'es', 'pl'].includes(targetIso) && tags && Array.isArray(tags)) { //all languages that define grammar tags per-pos rather than per-definition (like enwikt does) should go here
            glossTags.push(...tags);
        }

        if (head_templates && targetIso === 'en') {
            const tagMatch = [
                ['pf', 'perfective'],
                ['impf', 'imperfective'],
                ['m', 'masculine'],
                ['f', 'feminine'],
                ['n', 'neuter'],
                ['inan', 'inanimate'],
                ['anim', 'animate'],
            ];

            for (const entry of head_templates) {
                if (entry.expansion) {
                    for (const [match, tag] of tagMatch) {
                        if (
                            entry.expansion.replace(/\(.+?\)/g, '').split(' ').includes(match) &&
                            !glossTags.includes(tag)
                        ) {
                            glossTags.push(tag);
                        }
                    }
                }
            }
        }

        return {...sense, glossesArray, 'tags': glossTags};
    }));

    const sensesWithoutInflectionGlosses = sensesWithGlosses.filter(sense => {
        const {glossesArray, form_of, glosses} = sense;
        if(!isInflectionGloss(glossesArray, form_of)) return true;
        processInflectionGlosses(glosses, word, pos, form_of, sense.tags);
        return false;
    });

    if (sensesWithoutInflectionGlosses.length === 0) return;
    
    const readings = getReadings(word, parsedLine);
    initializeWordResult(word, readings, pos, String(etymology_number));

    for (const ipaObj of ipa) {
        saveIpaResult(word, readings, pos, String(etymology_number), ipaObj);
    }

    for (const reading of readings) {
        const currentEntry = lemmaDict[word][reading][pos][etymology_number];

        if (etymology_text) {
            const morphemeText = getMorphemes(etymology_text);

            if (targetIso === 'en' && morphemeText) {
                if (morphemeText === etymology_text) {
                    currentEntry.morpheme_text = morphemeText;
                } else {
                    currentEntry.etymology_text = etymology_text;
                    currentEntry.morpheme_text = morphemeText;
                }
            } else {
                currentEntry.etymology_text = etymology_text;
            }
        }

        if (head_templates) {
            const headInfo = getHeadInfo(head_templates);

            if (headInfo) {
                lemmaDict[word][reading][pos][etymology_number].head_info_text = headInfo;
            }
        }
    }

    const glossTree = getGlossTree(sensesWithoutInflectionGlosses);
    
    for (const reading of readings) {
        const posDict = lemmaDict[word]?.[reading]?.[pos] || {};
        let etymNum = etymology_number;
        let result = posDict[String(etymNum)];
    
        while (result?.glossTree?.size > 0) {
            etymNum += 1;
            result = posDict[String(etymNum)];
        }
    
        result = /** @type {LemmaInfo} */ (ensureNestedObject(lemmaDict, [word, reading, pos, String(etymNum)]));
    
        result.ipa ??= ipa;
        result.glossTree = glossTree;
        result.backlink = backlink;
    }
    
}
/**
 * @param {Sound[]} sounds 
 * @returns {IpaInfo[]}
 */
function getPhoneticTranscriptions(sounds) {
    if(!sounds) return [];
    switch(sourceIso) {
        case 'zh': {
            const ipaInfos = sounds.filter(sound => {
                if (!sound) return false;
                return sound.ipa || sound['zh-pron'];
            })
            .map(({ ipa, tags, note, 'zh-pron': zh_pron }) => {
                if (!tags) {
                    if (note) {
                        tags = [note];
                    } else {
                        tags = [];
                    }
                }
                return ({ ipa, tags, zh_pron });
            })
            
            /** @type {IpaInfo[]} */
            const ipaInfosWithStringIpa = []

            for (const ipaObj of ipaInfos) {
                if (typeof ipaObj.ipa === 'string') {
                    ipaInfosWithStringIpa.push(/** @type {IpaInfo} */ (ipaObj));
                } else if (Array.isArray(ipaObj.ipa)) {
                    for (const ipa of ipaObj.ipa) {
                        ipaInfosWithStringIpa.push({ ipa, tags: ipaObj.tags });
                    }
                } else if (ipaObj.zh_pron) {
                    ipaInfosWithStringIpa.push({ ipa: ipaObj.zh_pron, tags: ipaObj.tags });
                }
            }

            return ipaInfosWithStringIpa;
        }
        default: {
            const ipaInfos = sounds.filter(sound => {
                if (!sound) return false;
                return !!sound.ipa;
            })
            .map(({ ipa, tags, note }) => {
                if (!tags) {
                    if (note) {
                        tags = [note];
                    } else {
                        tags = [];
                    }
                }
                return ({ ipa, tags });
            })

            const ipaInfosWithStringIpa = /** @type {IpaInfo[]}*/ (ipaInfos.flatMap(ipaObj => typeof ipaObj.ipa === 'string' ? [ipaObj] : ipaObj?.ipa?.map(ipa => ({ ipa, tags: ipaObj.tags }))));
            
            return ipaInfosWithStringIpa;
        }
    }
}

/**
 * @param {string} text
 * @returns {string}
 * */
function getMorphemes(text) {
    for (const part of text.split(/(?<=\.)/g).map(item => item.trim())) {
        if (part.includes(' + ') && !/Proto|Inherited from/.test(part)) { return part; }
    }

    return '';
}

/**
 * @param {HeadTemplate[]} head_templates
 * @returns {string}
 * */
function getHeadInfo(head_templates) {
    for (const entry of head_templates) {
        if (entry.expansion) {
            if (/(?<=\().+?(?=\))/.test(entry.expansion)) return entry.expansion;
        }
    }

    return '';
}

/**
 * @param {Example} example
 * @returns {StandardizedExample}
 * */
function standardizeExample(example) {
    return { 
        text: example.text ? example.text.trim() : '',
        translation: getTranslationFromExample(example),
    };
}

/**
 * @param {Example} example
 * @returns {string}
 * */
function getTranslationFromExample(example) {
    if(example.translation) {
        return example.translation;
    }
    switch(targetIso) {
        case 'en':
            return example.english || example.roman || '';
        default:
            return '';
    }
}

/**
 * @param {TidySense[]} sensesWithoutInflectionGlosses 
 * @returns {GlossTree}
 */
function getGlossTree(sensesWithoutInflectionGlosses) {
    /** @type {GlossTree} */
    const glossTree = new Map();
    for (const sense of sensesWithoutInflectionGlosses) {
        const { glossesArray, tags } = sense;
        let { examples = [] } = sense;
        
        examples = examples
            .filter(example => example.text)
            .map(example => standardizeExample(example))
            .filter(({text}) => text.length <= 120)  // Filter out verbose examples
            .map((example, index) => ({ ...example, originalIndex: index }))  // Step 1: Decorate with original index
            .sort(({ translation: translationA, originalIndex: indexA }, { translation: translationB, originalIndex: indexB }) => {
                if (translationA && !translationB) return -1;   // translation items first
                if (!translationA && translationB) return 1;    // Non-translation items last
                return indexA - indexB;                 // Step 2: Stable sort by original index if equal
            })
            .map(({text, translation}) => ({text, translation}))  // Step 3: Pick only properties that will be used

        /** @type {GlossTree|GlossBranch} */
        let temp = glossTree;
        for (const [levelIndex, levelGloss] of glossesArray.entries()) {
            let curr = temp.get(levelGloss);
            if (!curr) {
                curr = new Map();
                temp.set(levelGloss, curr);
            }
            
            const branch = /** @type {GlossBranch} */ (curr);
            const filteredTags = curr.get('_tags') ? tags.filter(value => branch.get('_tags')?.includes(value)) : tags;
            branch.set('_tags', filteredTags);   
            
            if(levelIndex === glossesArray.length - 1) {
                curr.set('_examples', examples);
            }
            temp = curr;
        }
    }
    return glossTree;
}

/**
 * @param {FormInfo[]|undefined} forms
 * @param {string} word 
 * @param {string} pos 
 */
function processForms(forms, word, pos) {
    if(!forms) return;
    forms.forEach((formData) => {
        const { form } = formData;
        let { tags } = formData;
        if (!form) return;
        if (!tags) return;
        if (form === '-') return;
        tags = tags.filter(tag => !redundantTags.includes(tag));
        const isBlacklisted = tags.some(value => blacklistedTags.includes(value));
        if (isBlacklisted) return;
        const isIdentity = !tags.some(value => !identityTags.includes(value));
        if (isIdentity) return;

        /** @type {Map<Form, Map<PoS, string[]|Set<string>>>} */
        const wordMap = automatedForms.get(word) || new Map();
        /** @type {Map<string, Set<string>|string[]>} */
        const formMap = wordMap.get(form) || new Map();
        formMap.get(pos) || formMap.set(pos, new Set());
        wordMap.set(form, formMap);
        automatedForms.set(word, wordMap);

        const tagsSet = new Set((formMap.get(pos)));

        tagsSet.add(sortTags(targetIso, tags).join(' '));

        formMap.set(pos, similarSort(mergePersonTags(targetIso, Array.from(tagsSet))));
    });
}

/**
 * @param {string} word 
 * @param {string[]} readings 
 * @param {string} pos 
 * @param {string} etymology_number
 * @param {IpaInfo} ipaObj 
 */
function saveIpaResult(word, readings, pos, etymology_number, ipaObj) {
    for (const reading of readings) {
        const result = lemmaDict[word][reading][pos][etymology_number];
        const existingIpa = result.ipa.find(obj => obj.ipa === ipaObj.ipa);
        if (!existingIpa) {
            result.ipa.push(ipaObj);
        } else {
            existingIpa.tags = [...new Set([...existingIpa.tags, ...ipaObj.tags])];
        }
    }
}

/**
 * @param {string} word 
 * @param {string[]} readings 
 * @param {string} pos 
 * @param {string} etymology_number
 */
function initializeWordResult(word, readings, pos, etymology_number) {
    for (const reading of readings) {
        const result = ensureNestedObject(lemmaDict, [word, reading, pos, etymology_number]);
        result.ipa ??= [];
        result.glossTree ??= new Map();
    }
}

/**
 * @param {Glosses|undefined} glosses
 * @param {string} word 
 * @param {string} pos 
 * @param {FormOf[]} form_of
 * @param {string[]} senseTags
 * @returns 
 */
function processInflectionGlosses(glosses, word, pos, form_of, senseTags) {
    switch (targetIso) {
        case 'de':
            return processGermanInflectionGlosses(glosses, word, pos);
        case 'en':
            return processEnglishInflectionGlosses(glosses, word, pos);
        case 'el':
            // There can be multiple lemmas. Εχ. ήλιο from ήλιο (helium) / ήλιος (sun)
            const validTags = [
                "masculine", "feminine", "neuter",
                "singular", "plural",
                "nominative", "accusative", "genitive", "vocative"
            ];
            for (const { word: lemma } of form_of) {
                if (word === lemma) continue;
                let deinflections = senseTags.filter(tag => validTags.includes(tag));
                if (deinflections.length === 0) deinflections = [`από ${word}`];
                addDeinflections(word, pos, lemma, deinflections);
            }
        case 'fr':
            if(!glosses) return;
            /**
             * @type {string|undefined}
             */
            let inflection, lemma;

            const match1 = glosses[0].match(/(.*)du verbe\s+((?:(?!\bdu\b).)*)$/);
            const match2 = glosses[0].match(/^((?:(?:Masculin|Féminin)\s)?(?:(?:p|P)luriel|(?:s|S)ingulier)) de ([^\s]*)$/);

            if (match1) {
                inflection = match1[1];
                lemma = match1[2];
            } else if (match2) {
                inflection = match2[1];
                lemma = match2[2];
            }

            if (inflection && lemma) {
                inflection = inflection.trim();
                lemma = lemma.replace(/\.$/, '').trim();

                if (inflection && word !== lemma) {
                    addDeinflections(word, pos, lemma, [inflection]);
                }
            }
            break;
    }
}

/**
 * @param {Glosses|undefined} glosses 
 * @param {string} word 
 * @param {string} pos 
 * @returns 
 */
function processGermanInflectionGlosses(glosses, word, pos) {
    if (!glosses || !Array.isArray(glosses)) return;
    const match1 = glosses[0].match(/(.*)des (?:Verbs|Adjektivs|Substantivs|Demonstrativpronomens|Possessivpronomens|Pronomens) (.*)$/);
    if (!match1 || match1.length < 3) return;
    const inflection = match1[1].trim();
    const lemma = match1[2].trim();
    if (inflection && word !== lemma) {
        addDeinflections(word, pos, lemma, [inflection]);
    }
}

/**
 * @param {NestedObject} obj
 * @param {string[]} keys 
 * @returns {NestedObject}
 */
function ensureNestedObject(obj, keys) {
    for (const key of keys) {
        obj[key] ??= {};
        obj = obj[key];
    }
    return obj;
}

/**
 * @param {Glosses|undefined} glosses
 * @param {string} word 
 * @param {string} pos 
 */
function processEnglishInflectionGlosses(glosses, word, pos) {
    if(!glosses || !Array.isArray(glosses)) return;
    const glossPieces = glosses.flatMap(gloss => gloss.split('##').map(piece => piece.trim()));
    /**  @type {Set<string>} */
    const lemmas = new Set();
    /**  @type {Set<string>} */
    const inflections = new Set();
    for (const piece of glossPieces) {
        const lemmaMatch = piece.match(/of ([^\s]+)\s*(\(.+?\))?$/);
        if (lemmaMatch) {
            lemmas.add(lemmaMatch[1].replace(/:/g, '').trim());
        }

        if (lemmas.size > 1) {
            // console.warn(`Multiple lemmas in inflection glosses for word '${word}'`, lemmas);
            return;
        }

        const lemma = lemmas.values().next().value;

        if(!lemma) continue;

        const escapedLemma = escapeRegExp(lemma);

        const inflection = piece
            .replace(/inflection of /, '')
            .replace(new RegExp(`of ${escapedLemma}`), '')
            .replace(new RegExp(`${escapedLemma}`), '')
            .replace(new RegExp(`\\s+`), ' ')
            .replace(/:/g, '')
            .replace(/\s*\(.+?\)$/, '')
            .trim();

        inflections.add(inflection); 
    }
    
    const lemma = lemmas.values().next().value;
    
    for (const inflection of [...inflections].filter(Boolean)) {
        addDeinflections(word, pos, lemma, [inflection]);
    }
}

/**
 * @param {string} inflected
 * @param {string} uninflected
 * @returns {{inflected: string, uninflected: string}}
 */
function normalizeInflectionPair(inflected, uninflected) {
    switch(targetIso){
        case 'fr':
            inflected = inflected.replace(/(qu\')?(ils\/elles|il\/elle\/on)\s*/, '');
            break;
        default:
            break;
    }

    switch(sourceIso){
        case 'grc': {
            const articles = [ 
                "ὁ", "ἡ", "τό", "τώ", "τὼ", "οἱ", "αἱ", "τᾰ",
                "τόν", "τήν", "τούς", "τᾱ́ς",
                "τοῦ", "τῆς", "τοῦ", "τοῖν", "τῶν",
                "τῷ", "τῇ", "τῷ", "τοῖς", "ταῖς", "τοῖς"
            ];
            for (const article of articles) {
                const diacriticTest = new RegExp(`^${article}\\s`);
                const noDiacriticTest = new RegExp(`^${article.normalize('NFD').replace(/[\u0300-\u036f]/g, '')}\\s`);

                if (diacriticTest.test(inflected)){
                    inflected = inflected.replace(diacriticTest, '');
                } else if (noDiacriticTest.test(inflected)){
                    inflected = inflected.replace(noDiacriticTest, '');
                }
                if (diacriticTest.test(uninflected)){
                    uninflected = uninflected.replace(diacriticTest, '');
                } else if (noDiacriticTest.test(uninflected)){
                    uninflected = uninflected.replace(noDiacriticTest, '');
                }
            }
            return {inflected, uninflected};
        }
        default:
            return {inflected, uninflected};
    }
}

/**
 * @param {KaikkiLine} line
 * @returns {string|undefined}
 */
function getCanonicalWordForm({word, forms}) {
    if(!forms) return word;

    switch(sourceIso) {
        case 'ar':
        case 'fa':
        case 'la':
        case 'ru':
            return getCanonicalForm(word, forms); // canonical form is known to contain accent marks and such
        case 'grc': // the accent marks in the canonical form are not commonly used
        case 'de':
        case 'fr':
        case 'en': 
            return word; // canonical form is redundant, e.g. just prepends the definite article
        default:
            return getCanonicalForm(word, forms); // default could go either way. keeping existing behavior for now
    }
}

/**
 * @param {string|undefined} word 
 * @param {FormInfo[]} forms 
 * @returns {string|undefined}
 */
function getCanonicalForm(word, forms) {
    const canonicalForm = forms.find(form => form.tags &&
        form.tags.includes('canonical')
    );
    if (canonicalForm && canonicalForm.form) {
        word = canonicalForm.form;

        if (word.includes('{{#ifexist:Wiktionary')) { // TODO: remove once fixed in kaikki
            word = word.replace(/ {{#if:.+/, '').trim();
        }

        const bracketsRegex = /\[.*\]$/;
        if (bracketsRegex.test(word)) {
            word = word.replace(bracketsRegex, '').trim();
        }
    }
    return word;
}

/**
 * @param {string} word 
 * @param {KaikkiLine} line 
 * @returns {string[]}
 */
function getReadings(word, line){
    switch(sourceIso){
        case 'fa': return [getPersianReading(word, line)];
        case 'ja': return getJapaneseReadings(word, line);
        default:
            return [word];
    }
}

/**
 * @param {string} word 
 * @param {KaikkiLine} line 
 * @returns {string}
 */
function getPersianReading(word, line){
    const {forms} = line;
    if(!forms) return word;
    const romanization = forms.find(({form, tags}) => tags && tags.includes('romanization') && tags.length === 1 && form);
    return romanization?.form || word;
}

/**
 * @param {string} word 
 * @param {KaikkiLine} line 
 * @returns {string[]}
 */
function getJapaneseReadings(word, line){
    const {head_templates} = line;
    if(!head_templates) {
        return [word]; // among others, happens on kanji and alt forms
    }
    if(!Array.isArray(head_templates) || head_templates.length === 0) {
        return [word]; // never happens
    }
    const readings = [];
    for (const template of head_templates) {
        let reading;
        switch(template.name) {
            case 'ja-noun':
            case 'ja-adj':
            case 'ja-verb':
            case 'ja-verb form':
            case 'ja-verb-form':
            case 'ja-phrase':
                reading = template?.args?.[1];
                break;
            case 'ja-pos':
                reading = template?.args?.[2];
                break;
            case 'head':
            case 'ja-def':
            case 'ja-syllable':
                continue;
            default:
                // console.log('Unknown head_template:', word, head_templates);
        }
        if(reading) {
            readings.push(reading.replace(/\^| /g, ''));
        }
    }

    return readings.length > 0 ? readings : [word];
}

function handleAutomatedForms() {
    consoleOverwrite('3-tidy-up.js: Handling automated forms...');

    for (const [lemma, formInfo] of automatedForms.entries()) {
        for (const [form, posInfo] of formInfo.entries()) {
            for (const [pos, glosses] of posInfo.entries()) {
                addDeinflections(form, pos, lemma, glosses);
                
                posInfo.delete(pos);
            }
            
            formInfo.delete(form);
        }
        automatedForms.delete(lemma);
    }

    consoleOverwrite('3-tidy-up.js: Handled automated forms...');
}

/**
 * @param {Map<string, Array<{lemma: string, pos: string, glosses: string[]}>>} formPointer
 * @param {string} originalForm
 * @param {string} originalLemma
 * @param {string} form
 * @param {string} pos
 * @param {string[]} glosses
 */
function handleRecursiveForms(formPointer, originalForm, originalLemma, form, pos, glosses, level = 1, visited = new Set()) {
    if (visited.has(form)) return;

    visited.add(form);

    if (lemmaDict[form] && originalForm !== form) {
        if (level > 1) {
            const lemma = form;

            if (!formsMap.has(lemma)) {
                formsMap.set(lemma, new Map());
            }

            if (!formsMap.get(lemma).has(originalForm)) {
                formsMap.get(lemma).set(originalForm, new Map());
            }

            formsMap.get(lemma).get(originalForm).set(pos, glosses);

            formsMap.get(originalLemma)?.delete(originalForm);
        }
    }

    if (!lemmaDict[form] && formPointer.has(form)) {
        for (const { lemma, pos: subPos, glosses: subGlosses } of formPointer.get(form)) {
            if (level < 5) {
                if (pos === subPos) {
                    glosses.push(...subGlosses);
                    handleRecursiveForms(formPointer, originalForm, originalLemma, lemma, subPos, glosses, level + 1, visited);
                }
            }
        }
    }
}

lr.on('end', () => {
    clearConsoleLine();
    process.stdout.write(`Processed ${lineCount} lines...\n`);

    for (const file of readdirSync(writeFolder)) {
        if (file.includes(`${sourceIso}-${targetIso}`)) {
            unlinkSync(`${writeFolder}/${file}`);
        }
    }

    const lemmasFilePath = `${writeFolder}/${sourceIso}-${targetIso}-lemmas.json`;
    consoleOverwrite(`3-tidy-up.js: Writing lemma dict to ${lemmasFilePath}...`);
    writeFileSync(lemmasFilePath, JSON.stringify(lemmaDict, mapJsonReplacer));

    handleAutomatedForms();

    consoleOverwrite('Handling recursive forms...');

    /** @type {Map<string, Array<{lemma: string, pos: string, glosses: string[]}>>} */
    const formPointer = new Map();

    for (const [lemma, formMap] of formsMap.entries()) {
        for (const [form, formInfo] of formMap.entries()) {
            for (const [pos, glosses] of formInfo.entries()) {
                if (!formPointer.has(form)) {
                    formPointer.set(form, []);
                }

                formPointer.get(form).push({ lemma, pos, glosses });
            }
        }
    }

    for (const [form, entries] of formPointer.entries()) {
        for (const { lemma, pos, glosses } of entries) {
            handleRecursiveForms(formPointer, form, lemma, lemma, pos, [...glosses]);
        }
    }

    for (const prop of Object.getOwnPropertyNames(lemmaDict)) {
        delete lemmaDict[prop];
    }

    const formsFilePath = `${writeFolder}/${sourceIso}-${targetIso}-forms.json`;

    /** @type {{[chunkIndex: string]: FormsMap}} */
    const mapChunks = Array.from(formsMap.entries()).reduce((acc, [key, value], index) => {
        logProgress("Chunking form dict", index, formsMap.size);
        const chunkIndex = Math.floor(index / 10000);
        acc[chunkIndex] ??= new Map();
        acc[chunkIndex].set(key, value);
        return acc;
    }, /** @type {{[chunkIndex: string]: FormsMap}} */ ({}));
    
    if(!mapChunks['0']) {
        mapChunks['0'] = new Map();
    }

    for (const [index, chunk] of Object.entries(mapChunks)) {
        logProgress("Writing form dict chunks", index, Object.keys(mapChunks).length);
        consoleOverwrite(`3-tidy-up.js: Writing form dict ${index} to ${formsFilePath}...`);
        writeFileSync(`${formsFilePath.replace('.json', '')}-${index}.json`, JSON.stringify(chunk, mapJsonReplacer));
    }

    consoleOverwrite('3-tidy-up.js finished.\n');
});
