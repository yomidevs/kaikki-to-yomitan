const StreamZip = require('node-stream-zip');
const { execSync } = require('child_process');
const { readdirSync, existsSync, readFileSync, writeFileSync, unlinkSync } = require('fs');
const { writeInBatches } = require('../util/util');

const temp_folder = "../data/temp/dict"; // empty folder, will DELETE all files in it

if(!process.argv[2]){
    console.log("no file name provided, run with: node v4-to-v3.js <file>.zip");
    process.exit();
}

const file = process.argv[2];

if(!file.endsWith(".zip")){
    console.log("file must be a zip file");
    process.exit();
}

let terms = [];
let metaTerms = [];
let tags = [];
let index = {};

main(file);

async function main(file){
    if (existsSync(file)) {
        const zip = new StreamZip.async({ file });
        console.log("found", file);
        const files = Object.keys(await zip.entries());
        
        for (const file of files) {
            if(file.startsWith("term_meta_bank_")){
                metaTerms = metaTerms.concat(await getZipFileData(file, zip));
            }
            if(file.startsWith("term_bank_")){
                terms = terms.concat(await getZipFileData(file, zip));
            }
            if(file.startsWith("tag_bank_")){
                tags = tags.concat(await getZipFileData(file, zip));
            }
            if(file === "index.json"){
                index = await getZipFileData(file, zip);
            }
        }

        console.log("index", index)
        console.log("terms", terms.length);
        console.log("meta terms", metaTerms.length);
        console.log("tags", tags.length);

        index['format'] = 3;
        index['title'] += '-v3';

        await zip.close();

        for (const term of terms) {
            if(typeof term[8] === "string" && Array.isArray(term[9])){
                for(const inflectionChain of term[9]){
                    term[5].push([
                        term[8],
                        inflectionChain
                    ]);
                }
            }
            term.splice(8, 2);
        }

        for (const file of readdirSync(temp_folder)) {
            unlinkSync(`${temp_folder}/${file}`);
        }

        writeFileSync(`${temp_folder}/index.json`, JSON.stringify(index, null, 4));
        writeInBatches(temp_folder, Object.values(terms), 'term_bank_', 50000);
        writeInBatches(temp_folder, metaTerms, 'term_meta_bank_', 50000);
        writeInBatches(temp_folder, tags, 'tag_bank_', 50000);

        execSync(`zip -j ${index.title.split(' ').join('_')}.zip ${temp_folder}/*`);

        console.log(`\nDone.`);
    } else {
        console.log("file not found");
    }
}

async function getZipFileData(file, zip){
    const data = await zip.entryData(file);
    const json = JSON.parse(data);
    return json;
}
