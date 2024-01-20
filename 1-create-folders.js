const {mkdirSync} = require('fs');

const folders = [
    'kaikki',
    'sentences',
    'tidy',
    'language',
    'temp',
    'temp/dict',
    'temp/ipa',
    'test'
];

for (const folder of folders) {
    mkdirSync(`data/${folder}`, {recursive: true});
}