const LineByLineReader = require('line-by-line');
const { consoleOverwrite, clearConsoleLine, logProgress, findPartOfSpeech, loadJsonArray, writeInBatches, currentDate } = require('./util/util');
const { readdirSync, unlinkSync, writeFileSync } = require('fs');

const { 
    source_iso: sourceIso,
    target_iso: targetIso,
    kaikki_file: kaikkiFile,
    temp_folder: writeFolder,
} = process.env;

function processTranslations(translations, glosses, senses, sense){
    if (!translations) return;

    for (const translation of translations) {
        const translationIso = translation.code || translation.lang_code;
        if (translationIso !== targetIso) continue;
        const translated = translation.word || translation.note;
        if(!translated) continue;
        if(!translation.sense){
            if(!translation.sense_id){
                if(sense){
                    translation.sense = sense?.glosses?.[0];
                }
            } else {
                translation.sense = senses?.[translation.sense_id - 1]?.glosses?.[0]
            }
        }
        if(!translation.sense){
            translation.sense = "_none";
        } 
        const senseTranslations = glosses.get(translation.sense);
        if(!senseTranslations){
            glosses.set(translation.sense, [translated]);
        } else {
            senseTranslations.push(translated);
        }
    }
}

const partsOfSpeech = loadJsonArray(`data/language/target-language-tags/en/parts_of_speech.json`);
const skippedPartsOfSpeech = {};

const indexJson = {
    title: `kty-${sourceIso}-${targetIso}-gloss`,
    format: 3,
    revision: currentDate,
    sequenced: true,
    author: 'Kaikki-to-Yomitan contributors',
    url: 'https://github.com/themoeway/kaikki-to-yomitan',
    description: 'Dictionaries for various language pairs generated from Wiktionary data, via Kaikki and Kaikki-to-Yomitan.',
    attribution: 'https://kaikki.org/',
    sourceLanguage: sourceIso,
    targetLanguage: targetIso,
};

const ymtLemmas = [];

let lineCount = 0;
consoleOverwrite(`make-glossary.js started...`);

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
    const { pos, senses, translations } = parsedLine;
    const word = getCanonicalForm(parsedLine);
    const reading = getReading(word, parsedLine);

    if(!(word && pos && senses)) return;

    const glosses = new Map();

    processTranslations(translations, glosses, senses, null);
    for (const sense of senses) {
        const { translations } = sense;
        processTranslations(translations, glosses, senses, sense);
    }

    if (glosses.length === 0) return;
    
    const processedPoS = findPartOfSpeech(pos, partsOfSpeech, skippedPartsOfSpeech);
    const definitions = [];
    for (const [sense, translations] of glosses.entries()) {
        if(sense === "_none") {
            definitions.push(...translations)
            continue;
        } 
        definitions.push({
            type: "structured-content", 
            content: {
                tag: 'div',
                content: [
                    {
                        tag: 'span',
                        content: sense
                    },
                    {
                        tag: 'ul',
                        content: translations.map(translation => ({
                            tag: 'li',
                            content: translation
                        }))
                    }
                ]
            }
        })
    }
    if(definitions.length === 0) return;
    ymtLemmas.push([
        word,
        reading,
        processedPoS,
        processedPoS,
        0, // frequency
        definitions, // glosses
        0, // sequence
        '', // term_tags
    ]);
}

function getCanonicalForm({word, forms}) {
    if(!forms) return word;

    const canonicalForm = forms.find(form => 
        form.tags &&
        form.tags.includes('canonical')
    );
    if (canonicalForm) {
        word = canonicalForm.form;
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


lr.on('end', () => {
    clearConsoleLine();
    process.stdout.write(`Processed ${lineCount} lines...\n`);

    if(ymtLemmas.length === 0){
        console.log("No translations found. Exiting...");
        process.exit(0);
    }

    for (const file of readdirSync(`${writeFolder}/dict`)) {
        unlinkSync(`${writeFolder}/dict/${file}`);
    }

    writeFileSync(`${writeFolder}/dict/index.json`, JSON.stringify(indexJson, null, 2));

    return writeInBatches(writeFolder, ymtLemmas, `dict/term_bank_`, 25000);

});
