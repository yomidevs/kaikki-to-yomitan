const { writeFileSync, existsSync, readFileSync, readdirSync } = require('fs');
const path = require('path');

const LineByLineReader = require('line-by-line');

const sourceIso = process.env.source_iso;
const targetIso = process.env.target_iso;
const kaikkiFile = process.env.kaikki_file;

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

let lemmaDict = {};
let formDict = {};
let formStuff = [];
let automatedForms = {};

const testLemmas = {
    "de-en": ["pflegen", "trinken"],
    "es-en": ["soliviantar", "vivir"]
}

const testForms = {
    "de-en": ["trinkt", "gepflegt"],
    "es-en": ["quiero", "tonterías"]
}

let testResults = {};

const lr = new LineByLineReader(`data/kaikki/${kaikkiFile}`);

lr.on('line', (line) => {
    if (line) {
        lineCount += 1;

        if (lineCount % printInterval === 0) {
            process.stdout.clearLine();
            process.stdout.cursorTo(0);
            process.stdout.write(`Processed ${lineCount} lines...`);
        }

        handleLine(line, lemmaDict, formDict, formStuff, automatedForms, `${sourceIso}-${targetIso}`);
    }
});

function handleLine(line, lemmaDict, formDict, formStuff, automatedForms, langPair) {
    const { word, pos, senses, sounds, forms } = JSON.parse(line);

    if (word && pos && senses) {
        if (forms) {
            forms.forEach((formData) => {
                const { form, tags } = formData;

                if (form && tags && !tags.some(value => blacklistedTags.includes(value))) {
                    automatedForms[form] ??= {};
                    automatedForms[form][word] = {};
                    automatedForms[form][word][pos] ??= new Set();

                    const tagsSet = new Set(automatedForms[form][word][pos]);

                    tagsSet.add(tags.join(' '));

                    automatedForms[form][word][pos] = Array.from(tagsSet);
                }
            });
        }

        const ipa = sounds ? sounds.filter(sound => sound && sound.ipa).map(sound => ({ ipa: sound.ipa, tags: sound.tags || [] })) : [];

        // if (word === 'akull') {
        //     console.log(sounds);
        //     console.log(ipa);
        // }

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
                            let lemma = sense.glosses[0].replace(/.+(?=inflection of)/, '');
                            lemma = lemma.replace(/ \(.+?\)/, '');
                            lemma = lemma.replace(/:$/, '');
                            lemma = lemma.replace(/:\n.+/, '');
                            lemma = lemma.replace(/inflection of /, '');
                            lemma = lemma.replace(/:.+/, '');
                            lemma = lemma.trim();

                            const inflection = sense.glosses[1] || '';

                            if (inflection && !inflection.includes('inflection of') && word !== lemma) {
                                addDeinflections(formDict, word, pos, lemma, [inflection]);
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

        for (const [testWords, dict] of [[testLemmas, lemmaDict], [testForms, formDict]]) {
            if (testWords[langPair] && testWords[langPair].includes(word)) {
                testResults[word] ??= { dictPointer: dict, lines: [] };
                testResults[word].lines.push(JSON.parse(line));
            }
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
            } else if (glosses.length > 1) {
                addDeinflections(formDict, form, pos, lemma, [glosses[1]]);
            }
        }
    }

    // avoid automated forms during extra lang testing
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
    process.stdout.clearLine();
    process.stdout.cursorTo(0);
    process.stdout.write(`Processed ${lineCount} lines...\n`);

    handleForms(formStuff, formDict, automatedForms);

    const lemmasFilePath = `data/tidy/${sourceIso}-${targetIso}-lemmas.json`;
    console.log(`Writing lemma dict to ${lemmasFilePath}...`);
    writeFileSync(lemmasFilePath, JSON.stringify(lemmaDict));

    const formsFilePath = `data/tidy/${sourceIso}-${targetIso}-forms.json`;
    console.log(`Writing form dict to ${formsFilePath}...`);
    writeFileSync(formsFilePath, JSON.stringify(formDict));

    if (testResults) {
        let validResults = {};

        for (const [word, { dictPointer, lines }] of Object.entries(testResults)) {
            testResults[word].result = dictPointer[word];
            testResults[word].lines = lines;
            delete testResults[word].dictPointer;
        }

        validResults = JSON.parse(JSON.stringify(testResults));

        for (const [word] of Object.entries(testResults)) {
            delete testResults[word].lines;
        }

        const validTestFilePath = `data/test/${sourceIso}-${targetIso}-valid.json`;
        const latestTestFilePath = `data/test/${sourceIso}-${targetIso}-latest.json`;

        if (!existsSync(validTestFilePath)) {
            writeFileSync(validTestFilePath, JSON.stringify(validResults));
        }

        writeFileSync(latestTestFilePath, JSON.stringify(testResults));
    }

    // run code on other lang lines for testing
    const filePairs = {};

    const testFolder = 'data/test';

    for (const file of readdirSync(testFolder)) {
        const filePath = path.join(testFolder, file);
        const fileName = path.basename(filePath, path.extname(filePath));
        const langPair = fileName.replace('-latest', '').replace('-valid', '');

        filePairs[langPair] ??= {};

        if (fileName.includes('valid')) {
            filePairs[langPair].validFile = filePath;
        } else if (fileName.includes('latest')) {
            filePairs[langPair].latestFile = filePath;
        }
    }

    for (const [langPair, { validFile }] of Object.entries(filePairs)) {
        if (langPair !== `${sourceIso}-${targetIso}`) {
            const valid = JSON.parse(readFileSync(validFile));

            lemmaDict = {};
            formDict = {};
            formStuff = [];
            automatedForms = {};

            testResults = {};

            for (const [word, { lines }] of Object.entries(valid)) {
                for (const line of lines) {
                    handleLine(JSON.stringify(line), lemmaDict, formDict, formStuff, automatedForms, langPair);
                }

                handleForms(formStuff, formDict);
            }

            if (testResults) {
                for (const [word, { dictPointer }] of Object.entries(testResults)) {
                    testResults[word].result = dictPointer[word];
                    delete testResults[word].lines;
                    delete testResults[word].dictPointer;
                }

                const latestTestFilePath = `data/test/${langPair}-latest.json`;

                writeFileSync(latestTestFilePath, JSON.stringify(testResults));
            }
        }
    }

    console.log('2-tidy-up.js finished.');
});
