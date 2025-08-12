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

async function deleteObjectsInBatches(s3, bucketName, objectsToDelete, operationName = 'objects') {
  if (objectsToDelete.length === 0) {
    console.log(`No ${operationName} to delete`);
    return;
  }
  
  // Delete objects in batches of 1000 (AWS S3 limit)
  const batchSize = 1000;
  for (let i = 0; i < objectsToDelete.length; i += batchSize) {
    const batch = objectsToDelete.slice(i, i + batchSize);
    const deleteParams = {
      Bucket: bucketName,
      Delete: {
        Objects: batch.map(obj => ({ Key: obj.Key }))
      }
    };
    await s3.deleteObjects(deleteParams).promise();
    console.log(`Deleted ${operationName} batch ${Math.floor(i/batchSize) + 1}/${Math.ceil(objectsToDelete.length/batchSize)} (${batch.length} files)`);
  }
  console.log(`Deleted all ${operationName} (${objectsToDelete.length} total)`);
}

async function copyObjectsInBatches(s3, bucketName, sourceObjects, sourcePrefix, targetPrefix, operationName = 'objects') {
  if (sourceObjects.length === 0) {
    console.log(`No ${operationName} to copy`);
    return;
  }
  
  // Copy objects in batches
  const batchSize = 200;
  for (let i = 0; i < sourceObjects.length; i += batchSize) {
    const batch = sourceObjects.slice(i, i + batchSize);
    const copyPromises = batch.map(obj => {
      const newKey = obj.Key.replace(sourcePrefix, targetPrefix);
      return s3.copyObject({
        Bucket: bucketName,
        CopySource: `${bucketName}/${obj.Key}`,
        Key: newKey
      }).promise();
    });
    
    // Add generous timeout to prevent hanging
    const timeoutMs = 5 * 60 * 1000; // 5 minutes per batch
    await Promise.race([
      Promise.all(copyPromises),
      new Promise((_, reject) => 
        setTimeout(() => reject(new Error(`Copy batch ${Math.floor(i/batchSize) + 1} timed out after ${timeoutMs/1000}s`)), timeoutMs)
      )
    ]);
    console.log(`Copied ${operationName} batch ${Math.floor(i/batchSize) + 1}/${Math.ceil(sourceObjects.length/batchSize)} (${batch.length} files)`);
  }
  console.log(`Copied all ${operationName} (${sourceObjects.length} total)`);
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
            await deleteObjectsInBatches(s3, bucketName, backupObjects, 'backup folder files');
            await deleteObjectsInBatches(s3, bucketName, backupObjects, 'backup folder files');
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
            await copyObjectsInBatches(s3, bucketName, latestObjects, 'releases/latest/', 'releases/backup/', 'latest to backup files');
            
            // Delete objects from latest
            await deleteObjectsInBatches(s3, bucketName, latestObjects, 'latest folder files');
            await deleteObjectsInBatches(s3, bucketName, latestObjects, 'latest folder files');
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
    await copyObjectsInBatches(s3, bucketName, releaseObjects, `releases/${releaseVersion}/`, 'releases/latest/', 'release to latest files');
    
    console.log(`Successfully promoted release ${releaseVersion} to latest folder`);
    
    // List what's now in the latest folder
    console.log('Contents of latest folder:');
    const newLatestObjects = await getAllObjectsWithPrefix(bucketName, 'releases/latest/');
    console.log(`Latest folder now has ${newLatestObjects.length} objects`);

    // get all files, delete those not in latest or backup
    const allFiles = await getAllObjectsWithPrefix(bucketName, '');
    const filesToDelete = allFiles.filter(file =>
      file.Key.startsWith('releases/') &&
      !file.Key.startsWith('releases/latest/') && 
      !file.Key.startsWith('releases/backup/'));

    console.log(`Found ${filesToDelete.length} files to delete`);
    if(filesToDelete.length > 0) {
        await deleteObjectsInBatches(s3, bucketName, filesToDelete, 'files not in latest or backup');
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
