const { writeFileSync } = require('fs');

const LineByLineReader = require('line-by-line');

const { 
    source_iso: sourceIso,
    target_iso: targetIso,
    kaikki_file: kaikkiFile,
    tidy_folder: writeFolder
} = process.env;

const { sortTags, similarSort, mergePersonTags, consoleOverwrite, clearConsoleLine, logProgress, mapJsonReplacer } = require('./util/util');

const lemmaDict = {};
const formsMap = new Map();
const automatedForms = new Map();

function escapeRegExp(string) {
    return string.replace(/[.*+\-?^${}()|[\]\\]/g, '\\$&');
}

function isInflectionGloss(glosses, formOf) {
    glossesString = JSON.stringify(glosses);
    switch (targetIso) {
        case 'en':
            if (/.*inflection of.*/.test(glossesString)) return true;
            if(Array.isArray(formOf)) {
                for (const {word: lemma} of formOf) {
                    if (new RegExp(`of ${escapeRegExp(lemma)}`).test(glossesString)) return true;
                }
            }
        case 'fr':
            if (/.*du verbe\s+((?:(?!\bdu\b).)*)$/.test(glossesString)) return true;
            if (/((?:(?:Masculin|Féminin)\s)?(?:(?:p|P)luriel|(?:s|S)ingulier)) de ([^\s]+)/.test(glossesString)) return true;
    }
    return false;
}


function handleLevel(nest, level) {
    const nestDefs = [];
    let defIndex = 0;

    for (const [def, children] of Object.entries(nest)) {
        defIndex += 1;

        if (Object.keys(children).length > 0) {
            const nextLevel = level + 1;
            const childDefs = handleLevel(children, nextLevel);

            const listType = level === 1 ? "li" : "number";
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

function handleNest(nestedGlossObj, sense) {
    const nestedGloss = handleLevel(nestedGlossObj, 1);

    if (nestedGloss.length > 0) {
        for (const entry of nestedGloss) {
            sense.glosses.push({ "type": "structured-content", "content": entry });
        }
    }
}

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
        handleLine(line);
    }
});

function handleLine(line) {
    const parsedLine = JSON.parse(line);
    const { pos, sounds, forms } = parsedLine;
    if(!pos) return;
    const word = getCanonicalForm(parsedLine);
    if (!word) return;
    const reading = getReading(word, parsedLine);
    
    if (forms) {
        forms.forEach((formData) => {
            const { form } = formData;
            let { tags } = formData;
            if(!form) return;
            if(!tags) return;
            if(form === '-') return;
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
    
    const ipa = sounds 
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
            .flatMap(ipaObj => typeof ipaObj.ipa === 'string' ? [ipaObj] : ipaObj.ipa.map(ipa => ({ ipa, tags: ipaObj.tags })) )
            .filter(ipaObj => ipaObj.ipa)
        : [];

    
    const {senses} = parsedLine;
    if (!senses) return;
    let nestedGlossObj = {};
    
    const sensesWithGlosses = senses.filter(sense => sense.glosses || sense.raw_glosses || sense.raw_gloss);
    sensesWithGlosses.map(sense => {
        const glosses = sense.raw_glosses || sense.raw_gloss || sense.glosses;
        const glossesArray = Array.isArray(glosses) ? glosses : [glosses];
        sense.glossesArray = glossesArray;
    });
    
    for (const [senseIndex, sense] of sensesWithGlosses.entries()) {
        const glossesArray = sense.glossesArray;

        const formOf = sense.form_of;
        const tags = sense.tags || [];
        if(sense.raw_tags && Array.isArray(sense.raw_tags)) {
            tags.push(...sense.raw_tags);
        }

        if(glossesArray.length === 0) {
            continue;
        }

        if(isInflectionGloss(glossesArray, formOf)) {
            switch (targetIso) {
                case 'en':
                    processEnglishInflectionGlosses(sense, word, pos);
                    break;
                case 'fr':
                    let inflection, lemma;

                    const match1 = sense.glosses[0].match(/(.*)du verbe\s+((?:(?!\bdu\b).)*)$/);
                    const match2 = sense.glosses[0].match(/^((?:(?:Masculin|Féminin)\s)?(?:(?:p|P)luriel|(?:s|S)ingulier)) de ([^\s]*)$/);

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
            continue;
        }

        lemmaDict[word] ??= {};
        lemmaDict[word][reading] ??= {};
        lemmaDict[word][reading][pos] ??= {};
        lemmaDict[word][reading][pos].ipa ??= [];

        for (const ipaObj of ipa) {
            if (!lemmaDict[word][reading][pos].ipa.some(obj => obj.ipa === ipaObj.ipa)) {
                lemmaDict[word][reading][pos].ipa.push(ipaObj);
            }
        }

        lemmaDict[word][reading][pos].senses ??= [];

        const currSense = { glosses: [], tags };

        if (glossesArray.length > 1) {
            let nestedObj = nestedGlossObj;

            for (const level of glossesArray) {
                nestedObj[level] ??= {};
                nestedObj = nestedObj[level];
            }

            if (senseIndex === senses.length - 1 && nestedGlossObj) {
                try {
                    handleNest(nestedGlossObj, currSense);
                } catch (error) {
                    console.log(`Recursion error on word '${word}', pos '${pos}'`);
                    continue;
                }
                nestedGlossObj = {};
            }
        } else if (glossesArray.length === 1) {
            if (nestedGlossObj) {
                handleNest(nestedGlossObj, currSense);
                nestedGlossObj = {};
            }

            const gloss = glossesArray[0];

            if (!JSON.stringify(currSense.glosses).includes(gloss)) {
                currSense.glosses.push(gloss);
            }
        }

        if (currSense.glosses.length > 0) {
            lemmaDict[word][reading][pos].senses.push(currSense);
        }
    }
}

function processEnglishInflectionGlosses(sense, word, pos) {
    if (sense.glosses) {
        glossPieces = sense.glosses.flatMap(gloss => gloss.split('##').map(piece => piece.trim()));
        const lemmas = new Set();
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
}

function getCanonicalForm({word, forms}) {
    if(!forms) return word;

    const canonicalForm = forms.find(form => 
        form.tags &&
        form.tags.includes('canonical')
    );
    if (canonicalForm) {
        word = canonicalForm.form;

        if (word && word.includes('{{#ifexist:Wiktionary')) { // TODO: remove once fixed in kaikki
            word = word.replace(/ {{#if:.+/, '');
        }
    }
    return word;
}

function getReading(word, line){
    switch(sourceIso){
        case 'fa':
            return getPersianReading(word, line);
        default:
            return word;
    }
}

function getPersianReading(word, line){
    const {forms} = line;
    if(!forms) return word;
    const romanization = forms.find(({form, tags}) => tags && tags.includes('romanization') && tags.length === 1 && form);
    return romanization ? romanization.form : word;
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
