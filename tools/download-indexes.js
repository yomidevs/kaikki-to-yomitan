const fs = require('fs');
const path = require('path');
const https = require('https');

// Base URL for downloads
const BASE_URL = 'https://pub-c3d38cca4dc2403b88934c56748f5144.r2.dev/releases/latest/';

// Ensure the indexes directory exists
const indexesDir = path.join(__dirname, 'indexes');
if (!fs.existsSync(indexesDir)) {
    fs.mkdirSync(indexesDir, { recursive: true });
}

// Read languages.json
const languagesPath = path.join(__dirname, '..', 'languages.json');
const languages = JSON.parse(fs.readFileSync(languagesPath, 'utf8'));

// Sleep function for backoff delays
function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

// Function to download a file with exponential backoff
async function downloadFile(url, filepath, maxRetries = 4, baseDelay = 10000) {
    for (let attempt = 0; attempt <= maxRetries; attempt++) {
        try {
            return await downloadFileAttempt(url, filepath);
        } catch (error) {
            if (attempt === maxRetries) {
                console.log(`❌ Failed after ${maxRetries + 1} attempts: ${path.basename(filepath)}`);
                throw error;
            }
            
            const delay = baseDelay * Math.pow(2, attempt) *  (1 + Math.random());
            console.log(`⚠️  Attempt ${attempt + 1} failed for ${path.basename(filepath)}, retrying in ${delay}ms...`);
            await sleep(delay);
        }
    }
}

// Single download attempt
function downloadFileAttempt(url, filepath) {
    return new Promise((resolve, reject) => {
        const file = fs.createWriteStream(filepath);
        
        const request = https.get(url, (response) => {
            if (response.statusCode === 200) {
                response.pipe(file);
                file.on('finish', () => {
                    file.close();
                    console.log(`✅ Downloaded: ${path.basename(filepath)}`);
                    resolve();
                });
            } else if (response.statusCode === 404) {
                console.log(`❌ Not found: ${path.basename(filepath)}`);
                file.close();
                // Safely remove file if it exists
                try {
                    if (fs.existsSync(filepath)) {
                        fs.unlinkSync(filepath);
                    }
                } catch (err) {
                    // Ignore unlink errors
                }
                resolve(); // Resolve as this is expected for some combinations
            } else if (response.statusCode === 429 || response.statusCode === 503) {
                // Rate limited or service unavailable - should retry
                file.close();
                // Safely remove file if it exists
                try {
                    if (fs.existsSync(filepath)) {
                        fs.unlinkSync(filepath);
                    }
                } catch (err) {
                    // Ignore unlink errors
                }
                reject(new Error(`HTTP ${response.statusCode} - Rate limited`));
            } else {
                console.log(`⚠️  HTTP ${response.statusCode}: ${path.basename(filepath)}`);
                file.close();
                // Safely remove file if it exists
                try {
                    if (fs.existsSync(filepath)) {
                        fs.unlinkSync(filepath);
                    }
                } catch (err) {
                    // Ignore unlink errors
                }
                reject(new Error(`HTTP ${response.statusCode}`));
            }
        }).on('error', (err) => {
            file.close();
            // Safely remove file if it exists
            try {
                if (fs.existsSync(filepath)) {
                    fs.unlinkSync(filepath);
                }
            } catch (unlinkErr) {
                // Ignore unlink errors
            }
            reject(err);
        });
        
        // Set timeout for the request
        request.setTimeout(30000, () => {
            request.destroy();
            file.close();
            // Safely remove file if it exists
            try {
                if (fs.existsSync(filepath)) {
                    fs.unlinkSync(filepath);
                }
            } catch (err) {
                // Ignore unlink errors
            }
            reject(new Error('Request timeout'));
        });
    });
}

// Function to download all possible index files
async function downloadAllIndexes() {
    console.log('🚀 Starting download of all possible index files...\n');
    
    const downloadPromises = [];
    let totalFiles = 0;
    let downloadedFiles = 0;
    let notFoundFiles = 0;
    let errorFiles = 0;
    let retryCount = 0;
    
    // Generate all possible combinations
    for (const sourceLang of languages) {
        for (const targetLang of languages) {
            const sourceIso = sourceLang.iso;
            const targetIso = targetLang.iso;
            
            // Create base filename
            const baseFilename = `kty-${sourceIso}-${targetIso}`;
            
            // Download main index file
            if (targetLang.hasEdition) {
                const mainUrl = `${BASE_URL}${baseFilename}-index.json`;
                const mainFilepath = path.join(indexesDir, `${baseFilename}-index.json`);
                totalFiles++;
                
                downloadPromises.push(
                    downloadFile(mainUrl, mainFilepath)
                        .then(() => downloadedFiles++)
                        .catch((error) => {
                            if (error.message.includes('Rate limited')) {
                                retryCount++;
                            }
                            errorFiles++;
                        })
                );
            }
            
            if (sourceIso !== targetIso && sourceLang.hasEdition) {
                const glossUrl = `${BASE_URL}${baseFilename}-gloss-index.json`;
                const glossFilepath = path.join(indexesDir, `${baseFilename}-gloss-index.json`);
                totalFiles++;
                
                downloadPromises.push(
                    downloadFile(glossUrl, glossFilepath)
                        .then(() => downloadedFiles++)
                        .catch((error) => {
                            if (error.message.includes('Rate limited')) {
                                retryCount++;
                            }
                            errorFiles++;
                        })
                );
            }   
            
            if (targetLang.hasEdition) {
                // Download -ipa variant
                const ipaUrl = `${BASE_URL}${baseFilename}-ipa-index.json`;
                const ipaFilepath = path.join(indexesDir, `${baseFilename}-ipa-index.json`);
                totalFiles++;
                
                downloadPromises.push(
                    downloadFile(ipaUrl, ipaFilepath)
                        .then(() => downloadedFiles++)
                        .catch((error) => {
                            if (error.message.includes('Rate limited')) {
                                retryCount++;
                            }
                            errorFiles++;
                        })
                );
            }
        }
        // try kty-en-ipa-index.json and the like
        const filename = `kty-${sourceLang.iso}-ipa-index.json`;
        const ipaUrl = `${BASE_URL}${filename}`;
        const ipaFilepath = path.join(indexesDir, filename);
        totalFiles++;
        
        downloadPromises.push(
            downloadFile(ipaUrl, ipaFilepath)
                .then(() => downloadedFiles++)
                .catch((error) => {
                    if (error.message.includes('Rate limited')) {
                        retryCount++;
                    }
                    errorFiles++;
                })
        );
    }
    
    console.log(`📊 Total files to attempt: ${totalFiles}`);
    console.log(`📁 Language combinations: ${languages.length * (languages.length - 1)}`);
    console.log(`🔄 Variants per combination: 3 (main, -gloss, -ipa)\n`);
    
    try {
        await Promise.allSettled(downloadPromises);
        
        console.log('\n📈 Download Summary:');
        console.log(`✅ Successfully downloaded: ${downloadedFiles}`);
        console.log(`❌ Not found (404): ${notFoundFiles}`);
        console.log(`⚠️  Errors: ${errorFiles}`);
        console.log(`🔄 Total retries: ${retryCount}`);
        console.log(`📁 Files saved to: ${indexesDir}`);
        
    } catch (error) {
        console.error('❌ Error during download process:', error.message);
    }
}

// Function to show progress
function showProgress() {
    const files = fs.readdirSync(indexesDir);
    console.log(`\n📁 Current files in indexes folder: ${files.length}`);
    if (files.length > 0) {
        console.log('Files:');
        files.slice(0, 10).forEach(file => console.log(`  - ${file}`));
        if (files.length > 10) {
            console.log(`  ... and ${files.length - 10} more`);
        }
    }
}

// Main execution
if (require.main === module) {
    downloadAllIndexes()
        .then(() => {
            showProgress();
            console.log('\n🎉 Download process completed!');
        })
        .catch((error) => {
            console.error('💥 Fatal error:', error);
            process.exit(1);
        });
}

module.exports = { downloadAllIndexes, downloadFile };
