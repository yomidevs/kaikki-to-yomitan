const { existsSync, readFileSync, mkdirSync } = require('fs');
const path = require('path');
const { execSync } = require('child_process');


for (const dir of ["./data/test/dict", "./data/test/tidy", "./data/test/temp"]){
    if(!existsSync(dir)){
        mkdirSync(dir, {recursive: true});
    }
}

const languages = JSON.parse(readFileSync('languages.json', 'utf8'));


for (const {iso: sourceIso} of languages){
    for (const {iso: targetIso} of languages){
        const tidyLemmas = `data/test/tidy/${sourceIso}-${targetIso}-lemmas.json`;
        const tidyForms = `data/test/tidy/${sourceIso}-${targetIso}-forms.json`;
        
        if(!existsSync(tidyLemmas) || !existsSync(tidyForms)){
            continue;
        }

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
        
        const testTermTags = JSON.parse(readFileSync(`data/test/temp/dict/tag_bank_1.json`, 'utf8'));
        const testTerms = JSON.parse(readFileSync(`data/test/temp/dict/term_bank_1.json`, 'utf8'));
        const testIpaTags = JSON.parse(readFileSync(`data/test/temp/ipa/tag_bank_1.json`, 'utf8'));
        const testIpa = JSON.parse(readFileSync(`data/test/temp/ipa/term_meta_bank_1.json`, 'utf8'));

        const validTermTags = JSON.parse(readFileSync(`data/test/dict/${sourceIso}/${targetIso}/tag_bank_1.json`, 'utf8'));
        const validTerms = JSON.parse(readFileSync(`data/test/dict/${sourceIso}/${targetIso}/term_bank_1.json`, 'utf8'));
        const validIpaTags = JSON.parse(readFileSync(`data/test/ipa/${sourceIso}/${targetIso}/tag_bank_1.json`, 'utf8'));
        const validIpa = JSON.parse(readFileSync(`data/test/ipa/${sourceIso}/${targetIso}/term_meta_bank_1.json`, 'utf8'));

        describe(`Converting tidy ${sourceIso}-${targetIso} to yomitan format`, () => {
            test('should have valid term tags', () => {
                expect(testTermTags).toEqual(validTermTags);
            });

            test('should have valid terms', () => {
                expect(testTerms).toEqual(validTerms);
            });

            test('should have valid ipa tags', () => {
                expect(testIpaTags).toEqual(validIpaTags);
            });

            test('should have valid ipa', () => {
                expect(testIpa).toEqual(validIpa);
            });
        });
    }
}