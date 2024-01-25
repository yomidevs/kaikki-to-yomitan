const { execSync } = require('child_process');
const { readdirSync, existsSync, readFileSync, writeFileSync, mkdirSync } = require('fs');

const languages = JSON.parse(readFileSync('languages.json', 'utf8'));

for( const dir of ["./data/test/kaikki", "./data/test/tidy", "./data/test/temp/dict", "./data/test/temp/ipa"]){
    if(!existsSync(dir)){
        mkdirSync(dir, {recursive: true});
    }
}

for (const {iso: sourceIso} of languages){
    for (const {iso: targetIso} of languages){
        const kaikkiFile = `data/test/kaikki/${sourceIso}-${targetIso}.json`;
        
        if(!existsSync(kaikkiFile)){
            continue;
        }

        for (const dir of [`data/test/dict/${sourceIso}/${targetIso}`, `data/test/ipa/${sourceIso}/${targetIso}`]){
            if(!existsSync(dir)){
                mkdirSync(dir, {recursive: true});
            }
        }
        
        execSync(
            "node 3-tidy-up.js", 
            {
                env:{
                    ...process.env, 
                    kaikki_file: kaikkiFile,
                    source_iso: sourceIso,
                    target_iso: targetIso,
                    tidy_folder: `./data/test/tidy`
                }
            }
        );

        console.log(`Making yomitan for ${sourceIso}-${targetIso}`);
        execSync(
            "node 4-make-yomitan.js",
            {
                env:{
                    ...process.env, 
                    source_iso: sourceIso,
                    target_iso: targetIso,
                    DICT_NAME: 'test',
                    tidy_folder: `./data/test/tidy`,
                    temp_folder: `./data/test/temp`
                }
            }
        );

        prettifyFile(`data/test/tidy/${sourceIso}-${targetIso}-forms.json`);
        prettifyFile(`data/test/tidy/${sourceIso}-${targetIso}-lemmas.json`);

        const dictFiles = readdirSync(`data/temp/dict`);
        for(const file of dictFiles){
            if(file === `tag_bank_1.json` || file === 'term_bank_1.json'){
                outputFile = `data/test/dict/${sourceIso}/${targetIso}/${file}`;
                execSync(`mv data/test/temp/dict/${file} ${outputFile}`);
                prettifyFile(outputFile);
            }
        }

        const ipaFiles = readdirSync(`data/temp/ipa`);
        for(const file of ipaFiles){
            if(file === `tag_bank_1.json` || file === 'term_meta_bank_1.json'){
                outputFile = `data/test/ipa/${sourceIso}/${targetIso}/${file}`;
                execSync(`mv data/test/temp/ipa/${file} ${outputFile}`);
                prettifyFile(outputFile);
            }
        }
    }
}

function prettifyFile(file){
    const data = JSON.parse(readFileSync(file, 'utf8'));
    writeFileSync(file, JSON.stringify(data, null, 2));
}