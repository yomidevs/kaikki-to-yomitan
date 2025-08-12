const AWS = require('aws-sdk');

// Configure AWS SDK for Cloudflare R2
const s3 = new AWS.S3({
  accessKeyId: process.env.R2_ACCESS_KEY_ID,
  secretAccessKey: process.env.R2_SECRET_ACCESS_KEY,
  region: 'auto',
  endpoint: process.env.R2_ENDPOINT,
  s3ForcePathStyle: true
});

async function promoteReleaseToLatest(releaseVersion) {
  const bucketName = process.env.R2_BUCKET_NAME;
  
  try {
    console.log(`Checking if release ${releaseVersion} exists...`);
    
    // Verify the release exists
    try {
      await s3.headObject({
        Bucket: bucketName,
        Key: `releases/${releaseVersion}/`
      }).promise();
      console.log(`Release ${releaseVersion} found. Promoting to latest...`);
    } catch (error) {
      if (error.code === 'NotFound' || error.statusCode === 404) {
        throw new Error(`Release ${releaseVersion} not found in R2 bucket`);
      }
      throw error;
    }
    
    // Delete current backup folder if it exists
    console.log('Deleting current backup folder...');
    try {
      const backupObjects = await s3.listObjectsV2({
        Bucket: bucketName,
        Prefix: 'releases/backup/'
      }).promise();
      
      if (backupObjects.Contents && backupObjects.Contents.length > 0) {
        const deleteParams = {
          Bucket: bucketName,
          Delete: {
            Objects: backupObjects.Contents.map(obj => ({ Key: obj.Key }))
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
      const latestObjects = await s3.listObjectsV2({
        Bucket: bucketName,
        Prefix: 'releases/latest/'
      }).promise();
      
      if (latestObjects.Contents && latestObjects.Contents.length > 0) {
        // Copy objects from latest to backup
        for (const obj of latestObjects.Contents) {
          const newKey = obj.Key.replace('releases/latest/', 'releases/backup/');
          await s3.copyObject({
            Bucket: bucketName,
            CopySource: `${bucketName}/${obj.Key}`,
            Key: newKey
          }).promise();
        }
        
        // Delete objects from latest
        const deleteParams = {
          Bucket: bucketName,
          Delete: {
            Objects: latestObjects.Contents.map(obj => ({ Key: obj.Key }))
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
    
    // Now rename the target release to latest
    console.log(`Renaming release ${releaseVersion} to latest...`);
    const releaseObjects = await s3.listObjectsV2({
      Bucket: bucketName,
      Prefix: `releases/${releaseVersion}/`
    }).promise();
    
    if (releaseObjects.Contents && releaseObjects.Contents.length > 0) {
      // Copy objects from release to latest
      for (const obj of releaseObjects.Contents) {
        const newKey = obj.Key.replace(`releases/${releaseVersion}/`, 'releases/latest/');
        await s3.copyObject({
          Bucket: bucketName,
          CopySource: `${bucketName}/${obj.Key}`,
          Key: newKey
        }).promise();
      }
      
      // Delete objects from original release folder
      const deleteParams = {
        Bucket: bucketName,
        Delete: {
          Objects: releaseObjects.Contents.map(obj => ({ Key: obj.Key }))
        }
      };
      await s3.deleteObjects(deleteParams).promise();
      
      console.log(`Successfully promoted release ${releaseVersion} to latest folder`);
    } else {
      throw new Error(`No objects found in release ${releaseVersion}`);
    }
    
    // List what's now in the latest folder
    console.log('Contents of latest folder:');
    const newLatestObjects = await s3.listObjectsV2({
      Bucket: bucketName,
      Prefix: 'releases/latest/'
    }).promise();
    
    if (newLatestObjects.Contents) {
      newLatestObjects.Contents.forEach(obj => {
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
