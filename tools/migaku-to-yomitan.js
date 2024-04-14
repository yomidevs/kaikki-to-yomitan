/*
  Written mostly by ChatGPT4
*/

const { execSync } = require('child_process');
const { readdirSync, writeFileSync, unlinkSync, readFile } = require('fs');
const { writeInBatches } = require('../util/util');
const path = require('path');

function convertToYomitan(sourceJson) {
  // Parse the source JSON
  const data = JSON.parse(sourceJson);

  let count = 1; // Initialize a counter starting from 1

  const convertedData = data.map(item => {
    const definitionWithNewLines = item.definition.replace(/<br>/g, "\n");

    const convertedItem = [
      item.term, // "term" line
      "", //Empty string for "pronunciation" line
      "", // Empty string for "altterm" line
      "", // Empty string for "pos" line
      0, // Default value for the 5th element
      [definitionWithNewLines], // Array with "definition" line as an element
      count, // Assign the current count to the 7th element
      "", // Empty string for "audio" line
    ];

    count++; // Increment count for the next iteration
    return convertedItem;
  });

  return convertedData;
}

function generateIndexJson(fileName) {
    // Remove '_Dictionary' (case insensitive) from the fileName and replace '_' with spaces
    const title = fileName.replace(/_dictionary/gi, '').replace(/_/g, ' ');
  
    // Get the revision by getting the current date and time
    const revision = new Date().toLocaleString();
  
    // Create the index JSON object
    const indexJson = {
      title,
      revision,
      sequenced: true,
      format: 3,
      author: "",
      url: "",
      description: "Converted from a migaku dictionary.",
    };
  
    // Stringify the index JSON object
    return indexJson;
}

const file = process.argv[2];

if(!file){
  console.log("no file name provided, run with: node migaku-to-yomitan.js <file>.json");
  process.exit();
}

const temp_folder = "../data/temp/dict"; // empty folder, will DELETE all files in it

for (const file of readdirSync(temp_folder)) {
    unlinkSync(`${temp_folder}/${file}`);
}

// Read the source JSON file
readFile(file, 'utf8', (err, sourceJson) => {
  if (err) {
    console.error(`Error reading the file ${file}:`, err);
    return;
  }

  // Convert the source JSON to the new format
  const converted = convertToYomitan(sourceJson);

  // Generate the index JSON file
  const index = generateIndexJson(path.basename(file, '.json'));

  writeFileSync(`${temp_folder}/index.json`, JSON.stringify(index, null, 4));
  writeInBatches(temp_folder, converted, 'term_bank_', 50000);

  execSync(`zip -j ${index['title'].split(' ').join('_')}.zip ${temp_folder}/*`);
})

