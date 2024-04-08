const { writeFileSync } = require('fs');

const LineByLineReader = require('line-by-line');

const { 
    source_iso: sourceIso,
    target_iso: targetIso,
    kaikki_file: kaikkiFile,
    tidy_folder: writeFolder
} = process.env;

const { sortTags, similarSort, mergePersonTags, consoleOverwrite, clearConsoleLine } = require('./util/util');

const lemmaDict = {};
const formDict = {};
const automatedForms = {};

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

function addDeinflections(word, pos, lemma, inflections) {
    if (targetIso === 'fr') {
        word = word.replace(/(qu\')?(ils\/elles|il\/elle\/on)\s*/, '');
    }

    formDict[word] ??= {};
    formDict[word][lemma] ??= {};
    formDict[word][lemma][pos] ??= [];

    try {
        const inflectionsSet = new Set(formDict[word][lemma][pos]);
        for (const inflection of inflections) {
            inflectionsSet.add(inflection);
        }
    
        formDict[word][lemma][pos] = Array.from(inflectionsSet);
    } catch(e) {
        console.log(e);
    }
}

const blacklistedTags = [
    'inflection-template',
    'table-tags',
    'nominative',
    'canonical',
    'class',
    'error-unknown-tag',
    'error-unrecognized-form',
    'infinitive',
    'includes-article',
    'obsolete',
    'archaic',
    'used-in-the-form'
];

let lineCount = 0;
const printInterval = 1000;

consoleOverwrite(`3-tidy-up.js started...`);

const lr = new LineByLineReader(kaikkiFile);

lr.on('line', (line) => {
    if (line) {
        lineCount += 1;

        if (lineCount % printInterval === 0) {
            consoleOverwrite(`3-tidy-up.js: Processed ${lineCount} lines...`);
        }

        handleLine(line);
    }
});

function handleLine(line) {
    const parsedLine = JSON.parse(line);
    const { pos, senses, sounds, forms } = parsedLine;
    const word = getCanonicalForm(parsedLine);
    const reading = getReading(word, parsedLine);

    if (word && pos && senses) {
        if (forms) {
            forms.forEach((formData) => {
                const { form, tags } = formData;

                if (form && tags && !tags.some(value => blacklistedTags.includes(value)) && form !== '-') {
                    automatedForms[form] ??= {};
                    automatedForms[form][word] ??= {};
                    automatedForms[form][word][pos] ??= new Set();

                    const tagsSet = new Set(automatedForms[form][word][pos]);

                    tagsSet.add(sortTags(targetIso, tags).join(' '));

                    automatedForms[form][word][pos] = similarSort(mergePersonTags(targetIso, Array.from(tagsSet)));
                }
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

        let nestedGlossObj = {};

        let senseIndex = 0;
        for (const sense of senses) {
            const glosses = sense.raw_glosses || sense.raw_gloss || sense.glosses;
            const glossesArray = glosses
                ? Array.isArray(glosses) ? glosses : [glosses]
                : [];

            const formOf = sense.form_of;
            const tags = sense.tags || [];

            if (glossesArray.length > 0) {
                if (!isInflectionGloss(glossesArray, formOf)) {
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
                } else {
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
                }
            }
            senseIndex += 1;
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
    let missingForms = 0;

    for (const [form, info] of Object.entries(automatedForms)) {
        if (!(form in formDict)) {
            missingForms += 1;

            if (Object.keys(info).length < 5) {
                for (const [lemma, parts] of Object.entries(info)) {
                    for (const [pos, glosses] of Object.entries(parts)) {
                        if (form !== lemma) {
                            const inflections = glosses.map(gloss => `-automated- ${gloss}`);
                            addDeinflections(form, pos, lemma, inflections);
                        }
                    }
                }
            }
        }
    }

    console.log(`There were ${missingForms} missing forms that have now been automatically populated.`);
}

lr.on('end', () => {
    clearConsoleLine();
    process.stdout.write(`Processed ${lineCount} lines...\n`);

    handleAutomatedForms();

    const lemmasFilePath = `${writeFolder}/${sourceIso}-${targetIso}-lemmas.json`;
    consoleOverwrite(`3-tidy-up.js: Writing lemma dict to ${lemmasFilePath}...`);
    writeFileSync(lemmasFilePath, JSON.stringify(lemmaDict));

    const formsFilePath = `${writeFolder}/${sourceIso}-${targetIso}-forms.json`;
    consoleOverwrite(`3-tidy-up.js: Writing form dict to ${formsFilePath}...`);
    writeFileSync(formsFilePath, JSON.stringify(formDict));

    consoleOverwrite('3-tidy-up.js finished.\n');
});
