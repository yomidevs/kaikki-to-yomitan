const StreamZip = require('node-stream-zip');
const { execSync } = require('child_process');
const { readdirSync, existsSync, readFileSync, writeFileSync, mkdirSync } = require('fs');
const date = require('date-and-time');
const now = new Date();

async function main(){
    const languages = JSON.parse(readFileSync('languages.json', 'utf8'));
    
    for (const {iso: sourceIso} of languages){
        const globalIpa = {};
        const globalTags = [];

        for (const {iso: targetIso} of languages){
            let localIpa = [];
            let localTags = [];

            const file = `data/language/${sourceIso}/${targetIso}/kty-${sourceIso}-${targetIso}-ipa.zip`;
            if (existsSync(file)) {
                console.log("found", file);
                const zip = new StreamZip.async({ file });
                const files = Object.keys(await zip.entries());
                for (const file of files) {
                    if(file.startsWith("term_meta_bank_")){
                        const data = await zip.entryData(file);
                        const json = JSON.parse(data);
                        localIpa = localIpa.concat(json);
                    }
                    if(file.startsWith("tag_bank_")){
                        const data = await zip.entryData(file);
                        const json = JSON.parse(data);
                        localTags = localTags.concat(json);
                    }
                }

                console.log("localIpa", localIpa.length);
                console.log("localTags", localTags.length);

                await zip.close();

                for (const local of localIpa) {
                    const [term] = local
                    if(!globalIpa[term]){
                        globalIpa[term] = local;
                    } else {
                        const existingIpas = globalIpa[term][2]['transcriptions']
                        const newIpas = local[2]['transcriptions']

                        for (const newIpa of newIpas) {
                            if(!existingIpas.find(({ipa}) => ipa === newIpa.ipa)){
                                existingIpas.push(newIpa);
                                // [["🇺🇸","dialect",0,"US",0]] tag bank
                                const newTags = newIpa.tags.map(tag => localTags.find(([tagId]) => tagId === tag));
                                globalTags = globalTags.concat(newTags);
                            }
                        }
                    }
                }

                // const index = {"format":3,"revision":"ymt-2024.01.23","sequenced":true,"title":"kty-en-de-ipa"}
                const globalIndex = {
                    "format": 3,
                    "revision": date.format(now, 'YYYY.MM.DD'),
                    "sequenced": true,
                    "title": `kty-${sourceIso}-ipa`
                }
                // write index.json, term
            }
        }
    }
}

main()
