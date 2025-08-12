const AWS = require('aws-sdk');
const fs = require('fs');

// Configure AWS SDK for Cloudflare R2
const s3 = new AWS.S3({
  accessKeyId: process.env.R2_ACCESS_KEY_ID,
  secretAccessKey: process.env.R2_SECRET_ACCESS_KEY,
  region: 'auto',
  endpoint: process.env.R2_ENDPOINT,
  s3ForcePathStyle: true
});

async function downloadIPAFiles(calver, isos, editionLanguages) {
  const bucketName = process.env.R2_BUCKET_NAME;
  
  try {
    console.log('Downloading IPA dictionaries from R2...');
    console.log(`Calver: ${calver}`);
    console.log(`ISOs: ${isos.join(', ')}`);
    console.log(`Edition languages: ${editionLanguages.join(', ')}`);
    
    let downloadedCount = 0;
    let skippedCount = 0;
    
    // Download IPA files for each source-target combination
    for (const sourceIso of isos) {
      for (const targetIso of isos) {
        const filename = `kty-${sourceIso}-${targetIso}-ipa.zip`;
        
        // Skip if file already exists locally
        if (fs.existsSync(filename)) {
          console.log(`Skipping ${filename} - already exists locally`);
          skippedCount++;
          continue;
        }
        
        // Skip if target language doesn't have an edition
        if (!editionLanguages.includes(targetIso)) {
          console.log(`Skipping ${filename} - ${targetIso} is not an edition language`);
          skippedCount++;
          continue;
        }
        
        const key = `releases/${calver}/${filename}`;
        
        try {
          console.log(`Downloading ${filename}...`);
          
          const response = await s3.getObject({
            Bucket: bucketName,
            Key: key
          }).promise();
          
          fs.writeFileSync(filename, response.Body);
          console.log(`✓ Successfully downloaded ${filename}`);
          downloadedCount++;
          
        } catch (error) {
          if (error.code === 'NoSuchKey' || error.statusCode === 404) {
            console.log(`Skipping ${filename} - not found in R2`);
            skippedCount++;
          } else {
            console.log(`Error downloading ${filename}: ${error.message}`);
            skippedCount++;
          }
        }
      }
    }
    
    console.log(`\nDownload summary:`);
    console.log(`  Downloaded: ${downloadedCount} files`);
    console.log(`  Skipped: ${skippedCount} files`);
    console.log(`  Total processed: ${downloadedCount + skippedCount} files`);
    
    if (downloadedCount > 0) {
      console.log('\nDownloaded files:');
      const downloadedFiles = fs.readdirSync('.').filter(file => 
        file.endsWith('.zip') && file.includes('-ipa.zip')
      );
      downloadedFiles.forEach(file => console.log(`  ${file}`));
    }
    
  } catch (error) {
    console.error('Error downloading IPA files:', error.message);
    throw error;
  }
}

// Helper function to parse JSON arrays from environment variables
function parseJsonArray(jsonString) {
  try {
    return JSON.parse(jsonString);
  } catch (error) {
    console.error('Error parsing JSON array:', error.message);
    return [];
  }
}

// Main execution
if (require.main === module) {
  const calver = process.argv[2];
  const isosJson = process.argv[3];
  const editionLanguagesJson = process.argv[4];
  
  if (!calver || !isosJson || !editionLanguagesJson) {
    console.error('Usage: node download-ipa.js <calver> <isos_json> <edition_languages_json>');
    console.error('Example: node download-ipa.js 25.08.11.18 \'["en","de","ja"]\' \'["en","de"]\'');
    process.exit(1);
  }
  
  try {
    const isos = parseJsonArray(isosJson);
    const editionLanguages = parseJsonArray(editionLanguagesJson);
    
    if (isos.length === 0 || editionLanguages.length === 0) {
      console.error('Error: Invalid ISO or edition languages arrays');
      process.exit(1);
    }
    
    downloadIPAFiles(calver, isos, editionLanguages);
  } catch (error) {
    console.error('Download failed:', error.message);
    process.exit(1);
  }
}

module.exports = { downloadIPAFiles }; 