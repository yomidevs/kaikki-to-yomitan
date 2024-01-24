const { writeFileSync } = require('fs');

const LineByLineReader = require('line-by-line');

const { 
    source_iso: sourceIso,
    target_iso: targetIso,
    kaikki_file: kaikkiFile,
    tidy_folder: writeFolder
} = process.env;

const { sortTags, similarSort, mergePersonTags, consoleOverwrite, clearConsoleLine } = require('./util/util');

function isInflectionGloss(glosses) {
    if (targetIso === 'en') {
        return /.*inflection of.*/.test(JSON.stringify(glosses));
    } else if (targetIso === 'fr') {
        if (/.*du verbe\s+((?:(?!\bdu\b).)*)$/.test(JSON.stringify(glosses))) {
            return true;
        }
        if (/((?:(?:Masculin|Féminin)\s)?(?:(?:p|P)luriel|(?:s|S)ingulier)) de ([^\s]+)/.test(JSON.stringify(glosses))) {
            return true;
        }
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

function addDeinflections(formDict, word, pos, lemma, inflections) {
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
        // reserved keyword "constructor" causing issues

        // console.log(word, lemma, pos);
        // console.log(formDict[word][lemma][pos]);
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

const lemmaDict = {};
const formDict = {};
const formStuff = [];
const automatedForms = {};

consoleOverwrite(`3-tidy-up.js started...`);

const lr = new LineByLineReader(kaikkiFile);

lr.on('line', (line) => {
    if (line) {
        lineCount += 1;

        if (lineCount % printInterval === 0) {
            consoleOverwrite(`3-tidy-up.js: Processed ${lineCount} lines...`);
        }

        handleLine(line, lemmaDict, formDict, formStuff, automatedForms, `${sourceIso}-${targetIso}`);
    }
});

function handleLine(line, lemmaDict, formDict, formStuff, automatedForms) {
    const { word, pos, senses, sounds, forms } = JSON.parse(line);

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
                .map(sound => ({ ipa: sound.ipa, tags: sound.tags || [] }))
                .flatMap(ipaObj => typeof ipaObj.ipa === 'string' ? [ipaObj] : ipaObj.ipa.map(ipa => ({ ipa, tags: ipaObj.tags })) )
                .filter(ipaObj => ipaObj.ipa)
            : [];

        let nestedGlossObj = {};

        let senseIndex = 0;
        for (const sense of senses) {
            const glosses = sense.raw_glosses || sense.raw_gloss || sense.glosses;
            const glossesArray = Array.isArray(glosses) ? glosses : [glosses];

            const formOf = sense.form_of;
            const tags = sense.tags || [];

            if (glossesArray.length > 0) {
                if (formOf) {
                    formStuff.push([word, sense, pos]);
                } else {
                    if (!isInflectionGloss(glossesArray)) {
                        lemmaDict[word] ??= {};
                        lemmaDict[word][pos] ??= {};
                        lemmaDict[word][pos].ipa ??= [];

                        for (const ipaObj of ipa) {
                            if (!lemmaDict[word][pos].ipa.some(obj => obj.ipa === ipaObj.ipa)) {
                                lemmaDict[word][pos].ipa.push(ipaObj);
                            }
                        }

                        lemmaDict[word][pos].senses ??= [];

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
                            lemmaDict[word][pos].senses.push(currSense);
                        }
                    } else {
                        if (targetIso === 'en') {
                            if (!sense.glosses[0].includes('##')) {
                                let lemma = sense.glosses[0]
                                    .replace(/.+(?=inflection of)/, '')
                                    .replace(/ \(.+?\)/, '')
                                    .replace(/:$/, '')
                                    .replace(/:\n.+/, '')
                                    .replace(/inflection of /, '')
                                    .replace(/:.+/, '')
                                    .trim()

                                const inflection = sense.glosses[1] || '';

                                if (inflection && !inflection.includes('inflection of') && word !== lemma) {
                                    addDeinflections(formDict, word, pos, lemma, [inflection]);
                                }
                            }
                        } else if (targetIso === 'fr') {
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
                                    addDeinflections(formDict, word, pos, lemma, [inflection]);
                                }
                            }
                        }
                    }
                }
            }
            senseIndex += 1;
        }

    }
}

function handleForms(formStuff, formDict, automatedForms) {
    for (const [form, info, pos] of formStuff) {
        const glosses = info.glosses;
        const formOf = info.form_of;
        const lemma = formOf[0].word;

        if (form !== lemma && glosses) {
            if (!glosses[0].includes("##")) {
                addDeinflections(formDict, form, pos, lemma, [glosses[0]]);
            } else if (glosses.length > 1 && !glosses[1].includes('inflection of')) {
                addDeinflections(formDict, form, pos, lemma, [glosses[1]]);
            }
        }
    }

    if (automatedForms) {
        let missingForms = 0;

        for (const [form, info] of Object.entries(automatedForms)) {
            if (!(form in formDict)) {
                missingForms += 1;

                if (Object.keys(info).length < 5) {
                    for (const [lemma, parts] of Object.entries(info)) {
                        for (const [pos, glosses] of Object.entries(parts)) {
                            if (form !== lemma) {
                                const inflections = glosses.map(gloss => `-automated- ${gloss}`);
                                addDeinflections(formDict, form, pos, lemma, inflections);
                            }
                        }
                    }
                }
            }
        }

        console.log(`There were ${missingForms} missing forms that have now been automatically populated.`);
    }

}

lr.on('end', () => {
    clearConsoleLine();
    process.stdout.write(`Processed ${lineCount} lines...\n`);

    handleForms(formStuff, formDict, automatedForms);

    const lemmasFilePath = `${writeFolder}/${sourceIso}-${targetIso}-lemmas.json`;
    consoleOverwrite(`3-tidy-up.js: Writing lemma dict to ${lemmasFilePath}...`);
    writeFileSync(lemmasFilePath, JSON.stringify(lemmaDict));

    const formsFilePath = `${writeFolder}/${sourceIso}-${targetIso}-forms.json`;
    consoleOverwrite(`3-tidy-up.js: Writing form dict to ${formsFilePath}...`);
    writeFileSync(formsFilePath, JSON.stringify(formDict));

    consoleOverwrite('3-tidy-up.js finished.\n');
});
