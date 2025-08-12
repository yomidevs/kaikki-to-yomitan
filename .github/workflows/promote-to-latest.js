const AWS = require('aws-sdk');

// Configure AWS SDK for Cloudflare R2
const s3 = new AWS.S3({
  accessKeyId: process.env.R2_ACCESS_KEY_ID,
  secretAccessKey: process.env.R2_SECRET_ACCESS_KEY,
  region: 'auto',
  endpoint: process.env.R2_ENDPOINT,
  s3ForcePathStyle: true
});

async function getAllObjectsWithPrefix(bucketName, prefix) {
  const allObjects = [];
  let continuationToken = undefined;
  
  do {
    const params = {
      Bucket: bucketName,
      Prefix: prefix,
      MaxKeys: 1000, // Maximum allowed
      ...(continuationToken && { ContinuationToken: continuationToken })
    };
    
    const response = await s3.listObjectsV2(params).promise();
    
    if (response.Contents) {
      allObjects.push(...response.Contents);
    }
    
    continuationToken = response.NextContinuationToken;
    console.log(`Fetched ${allObjects.length} objects so far...`);
    
  } while (continuationToken);
  
  console.log(`Total objects found: ${allObjects.length}`);
  return allObjects;
}

async function promoteReleaseToLatest(releaseVersion) {
  const bucketName = process.env.R2_BUCKET_NAME;
  
  try {
    console.log(`Checking if release ${releaseVersion} exists...`);
    
    // Get ALL objects with pagination
    const releaseObjects = await getAllObjectsWithPrefix(bucketName, `releases/${releaseVersion}/`);
    
    if (releaseObjects.length === 0) {
      throw new Error(`Release ${releaseVersion} not found in R2 bucket`);
    }
    
    console.log(`Release ${releaseVersion} found with ${releaseObjects.length} objects. Promoting to latest...`);
    
    if(releaseVersion !== 'backup') {
        console.log('Deleting current backup folder...');
        try {
        const backupObjects = await getAllObjectsWithPrefix(bucketName, 'releases/backup/');
        
        if (backupObjects.length > 0) {
            const deleteParams = {
            Bucket: bucketName,
            Delete: {
                Objects: backupObjects.map(obj => ({ Key: obj.Key }))
            }
            };
            await s3.deleteObjects(deleteParams).promise();
            console.log('Current backup folder deleted');
        } else {
            console.log('No current backup folder found, skipping deletion');
        }
        } catch (error) {
        console.log('Error deleting backup folder:', error.message);
        // Continue with the process even if backup deletion fails
        }
        
        // Rename current latest folder to backup (if it exists)
        console.log('Renaming current latest folder to backup...');
        try {
        const latestObjects = await getAllObjectsWithPrefix(bucketName, 'releases/latest/');
        
        if (latestObjects.length > 0) {
            // Copy objects from latest to backup concurrently in batches
            const batchSize = 100; // Process 100 files at a time
            for (let i = 0; i < latestObjects.length; i += batchSize) {
                const batch = latestObjects.slice(i, i + batchSize);
                const copyPromises = batch.map(obj => {
                    const newKey = obj.Key.replace('releases/latest/', 'releases/backup/');
                    return s3.copyObject({
                        Bucket: bucketName,
                        CopySource: `${bucketName}/${obj.Key}`,
                        Key: newKey
                    }).promise();
                });
                
                await Promise.all(copyPromises);
                console.log(`Copied batch ${Math.floor(i/batchSize) + 1}/${Math.ceil(latestObjects.length/batchSize)}`);
            }
            
            // Delete objects from latest
            const deleteParams = {
                Bucket: bucketName,
                Delete: {
                    Objects: latestObjects.map(obj => ({ Key: obj.Key }))
                }
            };
            await s3.deleteObjects(deleteParams).promise();
            console.log('Current latest folder renamed to backup');
        } else {
            console.log('No current latest folder found, skipping rename to backup');
        }
        } catch (error) {
        console.log('Error renaming latest to backup:', error.message);
        // Continue with the process even if backup rename fails
        }
    }
    
    // Now copy the target release to latest
    console.log(`Copying release ${releaseVersion} to latest...`);
    
    // Copy objects from release to latest in batches
    const batchSize = 100;
    for (let i = 0; i < releaseObjects.length; i += batchSize) {
      const batch = releaseObjects.slice(i, i + batchSize);
      const copyPromises = batch.map(obj => {
        const newKey = obj.Key.replace(`releases/${releaseVersion}/`, 'releases/latest/');
        return s3.copyObject({
          Bucket: bucketName,
          CopySource: `${bucketName}/${obj.Key}`,
          Key: newKey
        }).promise();
      });
      
      await Promise.all(copyPromises);
      console.log(`Copied batch ${Math.floor(i/batchSize) + 1}/${Math.ceil(releaseObjects.length/batchSize)}`);
    }
    
    console.log(`Successfully promoted release ${releaseVersion} to latest folder`);
    
    // List what's now in the latest folder
    console.log('Contents of latest folder:');
    const newLatestObjects = await getAllObjectsWithPrefix(bucketName, 'releases/latest/');
    
    if (newLatestObjects.length > 0) {
      newLatestObjects.forEach(obj => {
        console.log(`  ${obj.Key}`);
      });
    }
    
  } catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
  }
}

// Main execution
if (require.main === module) {
  const releaseVersion = process.argv[2];
  if (!releaseVersion) {
    console.error('Usage: node promote-to-latest.js <release_version>');
    process.exit(1);
  }
  
  promoteReleaseToLatest(releaseVersion);
}

module.exports = { promoteReleaseToLatest };
