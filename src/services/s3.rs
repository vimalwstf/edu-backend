use crate::error::AppError;

use aws_sdk_s3::Client;

pub struct S3Service;

impl S3Service {
    pub async fn get_download_url(s3_client: &Client, s3_key: &str) -> Result<String, AppError> {
        let bucket_name = std::env::var("AWS_BUCKET_NAME").expect("AWS_BUCKET_NAME must be set");
        let presigned_req = s3_client
            .get_object()
            .bucket(&bucket_name)
            .key(s3_key)
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(
                    std::time::Duration::from_secs(3600),
                )
                .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
            )
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

        Ok(presigned_req.uri().to_string())
    }
}
