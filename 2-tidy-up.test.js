const { readFileSync, readdirSync } = require('fs');
const path = require('path');

const filePairs = {};

const testFolder = 'data/test';

for (const file of readdirSync(testFolder)) {
    const filePath = path.join(testFolder, file);
    const fileName = path.basename(filePath, path.extname(filePath));
    const languagePair = fileName.replace('-latest', '').replace('-valid', '');

    filePairs[languagePair] ??= {};

    if (fileName.includes('valid')) {
        filePairs[languagePair].validFile = filePath;
    } else if (fileName.includes('latest')) {
        filePairs[languagePair].latestFile = filePath;
    }
}

for (const [pairName, {validFile, latestFile}] of Object.entries(filePairs)) {
    it(`checks ${pairName} dictionary results`, () => {
        const valid = JSON.parse(readFileSync(validFile));
        const latest = JSON.parse(readFileSync(latestFile));

        for (const [word, { result }] of Object.entries(valid)) {
            expect(latest[word].result).toStrictEqual(result);
        }
    });
}
