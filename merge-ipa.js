const StreamZip = require('node-stream-zip');
const { execSync } = require('child_process');
const { readdirSync, existsSync, readFileSync, writeFileSync, unlinkSync, mkdirSync } = require('fs');
const { writeInBatches } = require('./util/util');
const date = require('date-and-time');
const now = new Date();

const tempFolder = 'data/temp/ipa';
mkdirSync(tempFolder, { recursive: true });

async function main(){
    const languages = JSON.parse(readFileSync('languages.json', 'utf8'));
    
    for (const {iso: sourceIso} of languages){
        const globalIpa = {};
        let globalTags = [];

        for (const {iso: targetIso} of languages){
            let localIpa = [];
            let localTags = [];

            const file = `kty-${sourceIso}-${targetIso}-ipa.zip`;
            if (existsSync(file)) {
                const zip = new StreamZip.async({ file });
                const files = Object.keys(await zip.entries());
                for (const file of files) {
                    if(file.startsWith("term_meta_bank_")){
                        const data = await zip.entryData(file);
                        const json = JSON.parse(data.toString());
                        localIpa = localIpa.concat(json);
                    }
                    if(file.startsWith("tag_bank_")){
                        const data = await zip.entryData(file);
                        const json = JSON.parse(data.toString());
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
                            const existingIpa = existingIpas.find(({ipa}) => ipa === newIpa.ipa);
                            if(!existingIpa){
                                existingIpas.push(newIpa);
                                const newTags = newIpa.tags.map(tag => localTags.find(([tagId]) => tagId === tag));
                                for (const newTag of newTags) {
                                    if(newTag && !globalTags.find(([tagId]) => tagId === newTag[0])){
                                        globalTags.push(newTag);
                                    }
                                }
                            } else {
                                const newTags = newIpa.tags.filter(tag => !existingIpa.tags.includes(tag));
                                for (const newTag of newTags) {
                                    existingIpa.tags.push(newTag);
                                    const fullTag = localTags.find(([tagId]) => tagId === newTag);
                                    if(fullTag && !globalTags.find(([tagId]) => tagId === fullTag[0])){
                                        globalTags.push(fullTag);
                                    }
                                }
                            }   
                        }
                    }
                }
            } 
        }

        const globalIpaLength = Object.keys(globalIpa).length;
        if(globalIpaLength) console.log("globalIpa", globalIpaLength);
        const globalTagsLength = globalTags.length;
        if(globalTagsLength) console.log("globalTags", globalTagsLength);
        
        const url = 'https://github.com/yomidevs/kaikki-to-yomitan';
        const title = `kty-${sourceIso}-ipa`;
        const latestReleaseUrl = `${url}/releases/latest/download/${title}`;
        const globalIndex = {
            "format": 3,
            "revision": date.format(now, 'YYYY.MM.DD'),
            "sequenced": true,
            title,
            url,
            "isUpdatable": true,
            "indexUrl": `${latestReleaseUrl}-index.json`,
            "downloadUrl": `${latestReleaseUrl}.zip`,
        }

        if(globalIpaLength){

            for (const file of readdirSync(tempFolder)) {
                unlinkSync(`${tempFolder}/${file}`);
            }


            writeFileSync(`${tempFolder}/index.json`, JSON.stringify(globalIndex, null, 4));
            writeInBatches(tempFolder, Object.values(globalIpa), 'term_meta_bank_', 500000);
            writeInBatches(tempFolder, globalTags, 'tag_bank_', 50000);
            
            const outputFolder = `data/language/${sourceIso}/`;
            mkdirSync(outputFolder, { recursive: true });
            execSync(`zip -j ${outputFolder}/${title}.zip ${tempFolder}/*`);
            writeFileSync(`${outputFolder}/${title}-index.json`, JSON.stringify(globalIndex, null, 4));
        }
    }
}

main()
