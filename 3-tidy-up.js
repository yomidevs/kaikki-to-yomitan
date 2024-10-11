const { writeFileSync } = require('fs');

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

const formsMap = new Map();
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
                if (glosses.some(gloss => new RegExp(`of ${escapeRegExp(lemma)}$`).test(gloss))) return true;
            }
            
        case 'fr':
            if (/.*du verbe\s+((?:(?!\bdu\b).)*)$/.test(glossesString)) return true;
            if (/((?:(?:Masculin|Féminin)\s)?(?:(?:p|P)luriel|(?:s|S)ingulier)) de ([^\s]+)/.test(glossesString)) return true;
    }
    return false;
}

/**
 * @param {GlossTree} glossTree
 * @param {number} level
 * @returns {*}
 */
function handleLevel(glossTree, level) {
    const nestDefs = [];
    let defIndex = 0;

    for (const [def, children] of glossTree) {
        defIndex += 1;

        if(children.size > 0) {
            const nextLevel = level + 1;
            const childDefs = handleLevel(children, nextLevel);

            const listType = level === 1 ? "li" : "number";
            /** @type {StructuredContent[]} */
            const content = level === 1 ? def : [{ "tag": "span", "data": { "listType": "number" }, "content": `${defIndex}. ` }, def];

            nestDefs.push([
                { "tag": "div", "data": { "listType": listType }, "content": content },
                { "tag": "div", "data": { "listType": "ol" }, "style": { "marginLeft": level + 1 }, "content": childDefs }
            ]);
        } else {
            nestDefs.push({ "tag": "div", "data": { "listType": "li" }, "content": [{ "tag": "span", "data": { "listType": "number" }, "content": `${defIndex}. ` }, def] });
        }
    }

    return nestDefs;
}

/**
 * @param {GlossTree} glossTree
 * @param {SenseInfo} sense
 */
function handleNest(glossTree, sense) {
    const nestedGloss = handleLevel(glossTree, 1);

    if (nestedGloss.length > 0) {
        for (const entry of nestedGloss) {
            sense.glosses.push({ "type": "structured-content", "content": entry });
        }
    }
}
/**
 * @param {string} form 
 * @param {string} pos 
 * @param {string} lemma 
 * @param {string[]} inflections 
 */
function addDeinflections(form, pos, lemma, inflections) {
    if (targetIso === 'fr') {
        form = form.replace(/(qu\')?(ils\/elles|il\/elle\/on)\s*/, '');
    }

    const lemmaForms = formsMap.get(lemma) || new Map();
    formsMap.set(lemma, lemmaForms);
    const formPOSs = lemmaForms.get(form) || new Map();
    lemmaForms.set(form, formPOSs);
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
    'romanization'
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
    const { pos, sounds, forms } = parsedLine;
    if(!pos) return;
    const word = getCanonicalWordForm(parsedLine);
    if (!word) return;
    
    processForms(forms, word, pos);

    const {senses} = parsedLine;
    if (!senses) return;
    
    /** @type {IpaInfo[]} */
    const ipa = /** @type {IpaInfo[]} */ (sounds 
        ? sounds
            .filter(sound => sound && sound.ipa)
            .map(({ipa, tags, note}) => {
                if(!tags) {
                    if (note) {
                        tags = [note];
                    } else {
                        tags = [];
                    }
                }
                return ({ipa, tags})
            })
            .flatMap(ipaObj => typeof ipaObj.ipa === 'string' ? [ipaObj] : ipaObj?.ipa?.map(ipa => ({ ipa, tags: ipaObj.tags })) )
            .filter(ipaObj => ipaObj?.ipa)
        : []);
    
    /** @type {TidySense[]} */
    const sensesWithGlosses = /** @type {TidySense[]} */ (senses
        .filter(sense => sense.glosses || sense.raw_glosses || sense.raw_gloss)
        .map(sense => {
        const glosses = sense.raw_glosses || sense.raw_gloss || sense.glosses;
        const glossesArray = Array.isArray(glosses) ? glosses : [glosses];

        const tags = sense.tags || [];
        if(sense.raw_tags && Array.isArray(sense.raw_tags)) {
            tags.push(...sense.raw_tags);
        }

        return {...sense, glossesArray, tags};
    }));

    const sensesWithoutInflectionGlosses = sensesWithGlosses.filter(sense => {
        const {glossesArray, form_of, glosses} = sense;
        if(!isInflectionGloss(glossesArray, form_of)) return true;
        processInflectionGlosses(glosses, word, pos);
        return false;
    });

    if (sensesWithoutInflectionGlosses.length === 0) return;
    
    const readings = getReadings(word, parsedLine);
    initializeWordResult(word, readings, pos);

    for (const ipaObj of ipa) {
        saveIpaResult(word, readings, pos, ipaObj);
    }

    /** @type {GlossTree} */
    const glossTree = new Map();
    for (const sense of sensesWithoutInflectionGlosses) {
        const { glossesArray, tags } = sense;
        let temp = glossTree;
        for (const [levelIndex, levelGloss] of glossesArray.entries()) {
            let curr = temp.get(levelGloss);
            if(!curr) {
                curr = new Map();
                temp.set(levelGloss, curr);
                if(levelIndex === 0) {
                    curr.set('_tags', tags);
                }
            } else if (levelIndex === 0) {
                curr.set('_tags', tags.filter(value => curr?.get('_tags')?.includes(value)));
            }
            temp = curr;
        }
    }
    
    for (const [gloss, children] of glossTree) {
        const tags = children.get('_tags') || [];
        children.delete('_tags');   

        /** @type {SenseInfo} */
        const currSense = { glosses: [], tags };
        if(children.size === 0) {
            currSense.glosses.push(gloss);
        } else {
            /** @type {GlossTree} */
            const branch = new Map();
            branch.set(gloss, children);
            handleNest(branch, currSense);
        }

        if (currSense.glosses.length > 0) {
            saveSenseResult(word, readings, pos, currSense);
        }
    }
}

/**
 * @param {Form[]|undefined} forms
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

        const wordMap = automatedForms.get(word) || new Map();
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
 * @param {SenseInfo} currSense 
 */
function saveSenseResult(word, readings, pos, currSense) {
    for (const reading of readings) {
        lemmaDict[word][reading][pos].senses.push(currSense);
    }
}

/**
 * @param {string} word 
 * @param {string[]} readings 
 * @param {string} pos 
 * @param {IpaInfo} ipaObj 
 */
function saveIpaResult(word, readings, pos, ipaObj) {
    for (const reading of readings) {
        const result = lemmaDict[word][reading][pos];
        if (!result.ipa.some(obj => obj.ipa === ipaObj.ipa)) {
            result.ipa.push(ipaObj);
        }
    }
}

/**
 * @param {string} word 
 * @param {string[]} readings 
 * @param {string} pos 
 */
function initializeWordResult(word, readings, pos) {
    for (const reading of readings) {
        const result = ensureNestedObject(lemmaDict, [word, reading, pos]);
        result.ipa ??= [];
        result.senses ??= [];
    }
}

/**
 * @param {Glosses|undefined} glosses
 * @param {string} word 
 * @param {string} pos 
 * @returns 
 */
function processInflectionGlosses(glosses, word, pos) {
    switch (targetIso) {
        case 'de':
            return processGermanInflectionGlosses(glosses, word, pos);
        case 'en':
            return processEnglishInflectionGlosses(glosses, word, pos);
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
        const lemmaMatch = piece.match(/of ([^\s]+)\s*$/);
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
            .trim();

        inflections.add(inflection); 
    }
    
    const lemma = lemmas.values().next().value;
    if (word !== lemma) {
        for (const inflection of [...inflections].filter(Boolean)) {
            addDeinflections(word, pos, lemma, [inflection]);
        }
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
        case 'de':
        // case 'fr': // canonical form sometimes just prepends the definite article, but many differ from the word in apostrophe variant. I don't know which is used in practice so leaving it until there's a yomitan preprocessor for french apostrophe usage. 
        case 'en': 
            return word; // canonical form is redundant, e.g. just prepends the definite article
        default:
            return getCanonicalForm(word, forms); // default could go either way. keeping existing behavior for now
    }
}

/**
 * @param {string|undefined} word 
 * @param {Form[]} forms 
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

    let counter = 0;
    let total = [...automatedForms.entries()].reduce((acc, [_, formInfo]) => acc + formInfo.size, 0);
    let missingForms = 0;

    for (const [lemma, formInfo] of automatedForms.entries()) {
        for (const [form, posInfo] of formInfo.entries()) {
            counter += 1;
            logProgress("Processing automated forms", counter, total);
            if (!formsMap.get(lemma)?.get(form)) {
                missingForms += 1;  
                for (const [pos, glosses] of posInfo.entries()) {
            
                    if (form !== lemma) {
                        addDeinflections(form, pos, lemma, glosses);
                    }
                    posInfo.delete(pos);
                }
            }
            formInfo.delete(form);
        }
        automatedForms.delete(lemma);
    }

    console.log(`There were ${missingForms} missing forms that have now been automatically populated.`);
}

lr.on('end', () => {
    clearConsoleLine();
    process.stdout.write(`Processed ${lineCount} lines...\n`);

    const lemmasFilePath = `${writeFolder}/${sourceIso}-${targetIso}-lemmas.json`;
    consoleOverwrite(`3-tidy-up.js: Writing lemma dict to ${lemmasFilePath}...`);
    writeFileSync(lemmasFilePath, JSON.stringify(lemmaDict));
    
    for (const prop of Object.getOwnPropertyNames(lemmaDict)) {
        delete lemmaDict[prop];
    }

    handleAutomatedForms();

    const formsFilePath = `${writeFolder}/${sourceIso}-${targetIso}-forms.json`;

    const mapChunks = Array.from(formsMap.entries()).reduce((acc, [key, value], index) => {
        logProgress("Chunking form dict", index, formsMap.size);
        const chunkIndex = Math.floor(index / 10000);
        acc[chunkIndex] ??= new Map();
        acc[chunkIndex].set(key, value);
        return acc;
    }, {});
    
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
