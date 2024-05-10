const { existsSync, readFileSync, mkdirSync } = require('fs');
const path = require('path');
const { execSync } = require('child_process');


for (const dir of ["./data/test/kaikki", "./data/test/tidy", "./data/test/temp"]){
    if(!existsSync(dir)){
        mkdirSync(dir, {recursive: true});
    }
}

const languages = JSON.parse(readFileSync('languages.json', 'utf8'));


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
                    tidy_folder: `./data/test/temp`
                }
            }
        );
        
        const testForms = JSON.parse(readFileSync(`data/test/temp/${sourceIso}-${targetIso}-forms-0.json`, 'utf8'));
        const testLemmas = JSON.parse(readFileSync(`data/test/temp/${sourceIso}-${targetIso}-lemmas.json`, 'utf8'));
        
        const validForms = JSON.parse(readFileSync(`data/test/tidy/${sourceIso}-${targetIso}-forms-0.json`, 'utf8'));
        const validLemmas = JSON.parse(readFileSync(`data/test/tidy/${sourceIso}-${targetIso}-lemmas.json`, 'utf8'));

        describe(`Tidying up ${sourceIso}-${targetIso}`, () => {
            test('should have valid forms', () => {
                expect(testForms).toEqual(validForms);
            });

            test('should have valid lemmas', () => {
                expect(testLemmas).toEqual(validLemmas);
            });
        });
    }
}