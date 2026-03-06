use aws_credential_types::Credentials;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use std::time::Duration;

pub struct MediaService {
    s3_client: aws_sdk_s3::Client,
    bucket: String,
}

impl MediaService {
    pub async fn new_from_env() -> Self {
        let endpoint = std::env::var("S3_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:9000".to_string());
        let bucket = std::env::var("S3_BUCKET")
            .unwrap_or_else(|_| "paw-media".to_string());
        let access_key = std::env::var("S3_ACCESS_KEY")
            .unwrap_or_else(|_| "paw_minio".to_string());
        let secret_key = std::env::var("S3_SECRET_KEY")
            .unwrap_or_else(|_| "paw_minio_password".to_string());
        let region = std::env::var("S3_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string());

        let credentials = Credentials::new(access_key, secret_key, None, None, "env");

        let shared_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region))
            .credentials_provider(credentials)
            .endpoint_url(&endpoint)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
            .force_path_style(true)
            .build();

        Self {
            s3_client: aws_sdk_s3::Client::from_conf(s3_config),
            bucket,
        }
    }

    pub async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, anyhow::Error> {
        self.s3_client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await?;

        Ok(key.to_string())
    }

    pub async fn presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, anyhow::Error> {
        let config = PresigningConfig::expires_in(expires_in)?;

        let presigned = self
            .s3_client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(config)
            .await?;

        Ok(presigned.uri().to_string())
    }

    pub async fn presigned_put_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, anyhow::Error> {
        let config = PresigningConfig::expires_in(expires_in)?;

        let presigned = self
            .s3_client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(config)
            .await?;

        Ok(presigned.uri().to_string())
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), anyhow::Error> {
        self.s3_client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}
