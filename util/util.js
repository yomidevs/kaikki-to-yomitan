//@ts-nocheck
const path = require('path');
const { readFileSync, writeFileSync, existsSync } = require('fs');
const date = require('date-and-time');

const tagOrder = JSON.parse(readFileSync(path.resolve(__dirname, '../data/language/tag_order.json'), 'utf-8'));

const tagOrderAll = [];

for (const [, tags] of Object.entries(tagOrder)) {
    tagOrderAll.push(...tags);
}

// sorts tags to follow `tag_order.json`
// tags not in tag_order are simply added to end of array

function sortTags(targetIso, tags) {
    if (targetIso !== 'en') return tags;

    return tags.sort((a, b) => {
        const indexA = tagOrderAll.indexOf(a);
        const indexB = tagOrderAll.indexOf(b);

        // Check if the tags are in tagOrder
        const isInOrderA = indexA !== -1;
        const isInOrderB = indexB !== -1;

        // Handle cases where both tags are in tagOrder or both are not
        if ((isInOrderA && isInOrderB) || (!isInOrderA && !isInOrderB)) {
            return indexA - indexB;
        }

        // Place the tag that is in tagOrder before the one that is not
        return isInOrderA ? -1 : 1;
    });
}

// sorts inflection entries to be nearby similar inflections
/**
 * @param {string[]} tags 
 * @returns {string[]}
 */
function similarSort(tags) {
    return tags.sort((a, b) => {
        const aWords = a.split(' ');
        const bWords = b.split(' ');

        // Check if the second word exists before comparing
        const mainComparison = (aWords[1] || '').localeCompare(bWords[1] || '');

        if (mainComparison !== 0) {
            return mainComparison;
        }

        for (let i = 0; i < Math.min(aWords.length, bWords.length); i++) {
            if (aWords[i] !== bWords[i]) {
                return aWords[i].localeCompare(bWords[i]);
            }
        }

        return aWords.length - bWords.length;
    });
}

// merge similar tags if the only difference is the persons
// input: ['first-person singular present', 'third-person singular present']
// output: ['first/third-person singular present']

/**
 * @param {string} targetIso 
 * @param {string[]} tags 
 * @returns {string[]}
 */
function mergePersonTags(targetIso, tags) {
    const persons = ["first-person", "second-person", "third-person"];

    if (tags.length > 1 && persons.some(item => JSON.stringify(tags).includes(item)) && targetIso === 'en') {
        function personSort(items) {
            return items.sort((a, b) => persons.indexOf(a) - persons.indexOf(b));
        }

        /** @type {string[]} */
        const result = [];
        /** @type {Object<string, string[]>} */
        const mergeObj = {};

        for (const item of tags) {
            const allTags = item.split(' ');
            const personTags = allTags.filter(tag => persons.includes(tag));

            if (personTags.length === 1) {
                const [person] = personTags;
                const otherTags = allTags.filter(tag => !persons.includes(tag));
                const tagKey = otherTags.join('_');

                mergeObj[tagKey] ??= [];
                mergeObj[tagKey].push(person);
            } else {
                result.push(item);
            }
        }

        for (const [tagKey, personMatches] of Object.entries(mergeObj)) {
            const tags = tagKey.split('_');
            const mergedTag = personSort(personMatches).join('/').replace(/-person/g, '') + '-person';

            result.push(sortTags(targetIso, [...tags, mergedTag]).join(' '));
        }

        return result;
    } else return tags;
}

function writeInBatches(tempPath, inputArray, filenamePrefix, batchSize = 100000, bankIndex = 0) {
    consoleOverwrite(`Writing ${inputArray.length.toLocaleString()} entries of ${filenamePrefix}...`);

    while (inputArray.length > 0) {
        const batch = inputArray.splice(0, batchSize);
        bankIndex += 1;
        const filename = `${tempPath}/${filenamePrefix}${bankIndex}.json`;
        const content = JSON.stringify(batch, null, 2);

        writeFileSync(filename, content);
    }

    return bankIndex;
}

function clearConsoleLine() {
    process.stdout.write('\r\x1b[K'); // \r moves the cursor to the beginning of the line, \x1b[K clears the line
}

function consoleOverwrite(text) {
    clearConsoleLine();
    process.stdout.write(text);
}

function logProgress(msg, current, total, interval = 1000) {
    if (current % interval === 0) {
        let progress = `${msg} ${current.toLocaleString()}`;
        if (total) {
            const percent = Math.floor(current / total * 100);
            progress += ` / ${total.toLocaleString()} (${percent}%)`;
        }
        progress += '...';
        consoleOverwrite(progress);
    }
}

function mapJsonReplacer (key, value) {
    if (value instanceof Map) {
        return {
            _type: "map",
            map: [...value],
        }
    } else return value;
}

function mapJsonReviver (key, value) {
    if (value._type == "map") return new Map(value.map);
    else return value;
}

function loadJsonArray(file) {
    return existsSync(file) ? JSON.parse(readFileSync(file)) : [];
}

function incrementCounter(key, counter) {
    counter[key] = (counter[key] || 0) + 1;
}

function findPartOfSpeech(pos, partsOfSpeech, skippedPartsOfSpeech) {
    for(const posAliases of partsOfSpeech){
        if (posAliases.includes(pos)){
            return posAliases[0];
        }
    }
    if(skippedPartsOfSpeech) incrementCounter(pos, skippedPartsOfSpeech);
    return pos;
}

const now = new Date();
const currentDate = date.format(now, 'YYYY.MM.DD');

module.exports = { 
    sortTags, 
    similarSort,
    mergePersonTags,
    writeInBatches,
    consoleOverwrite,
    clearConsoleLine,
    logProgress,
    mapJsonReplacer,
    mapJsonReviver,
    loadJsonArray,
    findPartOfSpeech,
    incrementCounter,
    currentDate
};