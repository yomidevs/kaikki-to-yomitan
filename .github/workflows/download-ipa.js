import AWS from 'aws-sdk';
import fs from 'fs';

// Configure AWS SDK for Cloudflare R2
const s3 = new AWS.S3({
  accessKeyId: process.env.R2_ACCESS_KEY_ID,
  secretAccessKey: process.env.R2_SECRET_ACCESS_KEY,
  region: 'auto',
  endpoint: process.env.R2_ENDPOINT,
  s3ForcePathStyle: true
});

// Load languages.json to get edition languages and all ISOs
function loadLanguages() {
  try {
    const languagesData = fs.readFileSync('languages.json', 'utf8');
    return JSON.parse(languagesData);
  } catch (error) {
    console.error('Error loading languages.json:', error.message);
    return [];
  }
}

export async function downloadIPAFiles(calver) {
  const bucketName = process.env.R2_BUCKET_NAME;
  
  try {
    console.log('Downloading IPA dictionaries from R2...');
    console.log(`Calver: ${calver}`);

    const languages = loadLanguages();
        
    let downloadedCount = 0;
    let skippedCount = 0;
    
    // Download IPA files for each source-target combination
    for (const sourceLanguage of languages) {
      const sourceIso = sourceLanguage.iso;
      for (const targetLanguage of languages) {
        if(!targetLanguage.hasEdition) {
          continue;
        }
        const targetIso = targetLanguage.iso;
        const filename = `kty-${sourceIso}-${targetIso}-ipa.zip`;
        
        // Skip if file already exists locally
        if (fs.existsSync(filename)) {
          console.log(`Skipping ${filename} - already exists locally`);
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

// Main execution
if (import.meta.url === `file://${process.argv[1]}`) {
  const calver = process.argv[2];
  
  if (!calver) {
    console.error('Usage: node download-ipa.js <calver>');
    console.error('Example: node download-ipa.js 25.08.11.18');
    process.exit(1);
  }
  
  try {
    downloadIPAFiles(calver);
  } catch (error) {
    console.error('Download failed:', error.message);
    process.exit(1);
  }
} 