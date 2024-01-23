const { writeFileSync } = require('fs');


function writeInBatches(inputArray, filenamePrefix, batchSize = 100000) {
    consoleOverwrite(`Writing ${inputArray.length.toLocaleString()} entries of ${filenamePrefix}...`);

    let bankIndex = 0;

    while (inputArray.length > 0) {
        const batch = inputArray.splice(0, batchSize);
        bankIndex += 1;
        const filename = `${tempPath}/${filenamePrefix}${bankIndex}.json`;
        const content = JSON.stringify(batch, null, 2);

        writeFileSync(filename, content);
    }
}

module.exports = {writeInBatches}