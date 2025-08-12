import AWS from 'aws-sdk';
import fs from 'fs';
import path from 'path';

// Configure AWS SDK for Cloudflare R2
const s3 = new AWS.S3({
  accessKeyId: process.env.R2_ACCESS_KEY_ID,
  secretAccessKey: process.env.R2_SECRET_ACCESS_KEY,
  region: 'auto',
  endpoint: process.env.R2_ENDPOINT,
  s3ForcePathStyle: true
});

export async function uploadFiles(calver, filePattern, description) {
  const bucketName = process.env.R2_BUCKET_NAME;
  
  try {
    console.log(`Finding ${description}...`);
    
    // Find files matching the pattern
    const files = findFiles('data/language', filePattern);
    
    if (files.length === 0) {
      console.log(`No ${description} found.`);
      return;
    }
    
    console.log(`Found ${files.length} ${description}:`);
    files.forEach(file => console.log(`  ${file}`));
    
    // Upload each file to R2
    for (const file of files) {
      const fileName = path.basename(file);
      const key = `releases/${calver}/${fileName}`;
      
      console.log(`Uploading ${fileName} to ${key}...`);
      
      const fileContent = fs.readFileSync(file);
      await s3.upload({
        Bucket: bucketName,
        Key: key,
        Body: fileContent,
        ACL: 'public-read',
        ContentType: getContentType(fileName)
      }).promise();
      
      console.log(`✓ Successfully uploaded ${fileName}`);
    }
    
    console.log(`All ${description} uploaded successfully!`);
    
  } catch (error) {
    console.error(`Error uploading ${description}:`, error.message);
    throw error;
  }
}

function findFiles(dir, pattern) {
  const files = [];
  
  function scanDirectory(currentDir) {
    try {
      const items = fs.readdirSync(currentDir);
      
      for (const item of items) {
        const fullPath = path.join(currentDir, item);
        const stat = fs.statSync(fullPath);
        
        if (stat.isDirectory()) {
          scanDirectory(fullPath);
        } else if (stat.isFile() && pattern.test(item)) {
          files.push(fullPath);
        }
      }
    } catch (error) {
      // Skip directories we can't read
      console.log(`Warning: Could not read directory ${currentDir}: ${error.message}`);
    }
  }
  
  scanDirectory(dir);
  return files;
}

function getContentType(fileName) {
  if (fileName.endsWith('.zip')) {
    return 'application/zip';
  } else if (fileName.endsWith('.json')) {
    return 'application/json';
  }
  return 'application/octet-stream';
}

export async function uploadDictionaryFiles(calver) {
  await uploadFiles(calver, /.*kty.*\.zip$/, 'dictionary files');
}

export async function uploadIndexFiles(calver) {
  await uploadFiles(calver, /.*kty.*-index\.json$/, 'index.json files');
}

export async function uploadMergedFiles(calver) {
  await uploadFiles(calver, /.*\.zip$/, 'merged dictionary files');
}

// Main execution
if (import.meta.url === `file://${process.argv[1]}`) {
  const command = process.argv[2];
  const calver = process.argv[3];
  
  if (!command || !calver) {
    console.error('Usage: node upload-files.js <command> <calver>');
    console.error('Commands: dict, index, merged');
    console.error('Example: node upload-files.js dict 25.08.11.18');
    process.exit(1);
  }
  
  try {
    switch (command) {
      case 'dict':
        await uploadDictionaryFiles(calver);
        break;
      case 'index':
        await uploadIndexFiles(calver);
        break;
      case 'merged':
        await uploadMergedFiles(calver);
        break;
      default:
        console.error(`Unknown command: ${command}`);
        process.exit(1);
    }
  } catch (error) {
    console.error('Upload failed:', error.message);
    process.exit(1);
  }
} 