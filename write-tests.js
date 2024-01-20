const { execSync } = require('child_process');
const { readdirSync, existsSync, readFileSync, writeFileSync, mkdirSync } = require('fs');

const languages = JSON.parse(readFileSync('languages.json', 'utf8'));

for( const dir of ["./data/test/kaikki", "./data/test/tidy"]){
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

        const validForms = JSON.parse(readFileSync(`data/test/tidy/${sourceIso}-${targetIso}-forms.json`, 'utf8'));
        const validLemmas = JSON.parse(readFileSync(`data/test/tidy/${sourceIso}-${targetIso}-lemmas.json`, 'utf8'));

        writeFileSync(`data/test/tidy/${sourceIso}-${targetIso}-forms.json`, JSON.stringify(validForms, null, 2));
        writeFileSync(`data/test/tidy/${sourceIso}-${targetIso}-lemmas.json`, JSON.stringify(validLemmas, null, 2));
    }
}