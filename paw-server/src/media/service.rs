use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use std::time::Duration;

use aws_sdk_s3::Client;

#[async_trait]
trait StorageProvider {
    async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, anyhow::Error>;

    async fn presigned_url(&self, key: &str, expires_in: Duration)
        -> Result<String, anyhow::Error>;

    async fn presigned_put_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, anyhow::Error>;

    async fn delete_object(&self, key: &str) -> Result<(), anyhow::Error>;
}

struct AwsS3Storage {
    client: Client,
    bucket: String,
}

struct GcpStorage {
    client: Client,
    bucket: String,
}

impl AwsS3Storage {
    async fn from_env() -> Self {
        let endpoint = read_env("S3_ENDPOINT", Some("http://localhost:39080"));
        let bucket = read_env("S3_BUCKET", Some("paw-media"));
        let region = read_env("S3_REGION", Some("us-east-1"));
        let access_key = read_credential("AWS_ACCESS_KEY_ID", "S3_ACCESS_KEY");
        let secret_key = read_credential("AWS_SECRET_ACCESS_KEY", "S3_SECRET_KEY");

        let mut cfg_builder = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region))
            .endpoint_url(endpoint);

        if let (Some(access), Some(secret)) = (access_key, secret_key) {
            if !access.is_empty() && !secret.is_empty() {
                cfg_builder = cfg_builder
                    .credentials_provider(Credentials::new(access, secret, None, None, "env"));
            }
        } else {
            cfg_builder = cfg_builder.credentials_provider(Credentials::new(
                "paw_minio".to_string(),
                "paw_minio_password".to_string(),
                None,
                None,
                "env",
            ));
        }

        let force_path_style = std::env::var("S3_FORCE_PATH_STYLE")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "True"))
            .unwrap_or(true);

        let shared_config = cfg_builder.load().await;
        let s3_config = S3ConfigBuilder::from(&shared_config)
            .force_path_style(force_path_style)
            .build();

        Self {
            client: Client::from_conf(s3_config),
            bucket,
        }
    }
}

impl GcpStorage {
    async fn from_env() -> Self {
        let endpoint = read_env("S3_ENDPOINT", Some("https://storage.googleapis.com"));
        let bucket = read_env("S3_BUCKET", Some("paw-media"));
        let region = read_env("S3_REGION", Some("us-east-1"));
        let mut cfg_builder = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region))
            .endpoint_url(&endpoint);

        if let (Some(access), Some(secret)) = (
            std::env::var("GOOGLE_STORAGE_ACCESS_KEY").ok(),
            std::env::var("GOOGLE_STORAGE_SECRET_KEY").ok(),
        ) {
            if !access.is_empty() && !secret.is_empty() {
                cfg_builder = cfg_builder
                    .credentials_provider(Credentials::new(access, secret, None, None, "gcp"));
            }
        }

        let shared_config = cfg_builder.load().await;
        let s3_config = S3ConfigBuilder::from(&shared_config)
            .force_path_style(false)
            .build();

        Self {
            client: Client::from_conf(s3_config),
            bucket,
        }
    }
}

#[async_trait]
impl StorageProvider for AwsS3Storage {
    async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, anyhow::Error> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await?;

        Ok(key.to_string())
    }

    async fn presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, anyhow::Error> {
        let config = PresigningConfig::expires_in(expires_in)?;
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(config)
            .await?;
        Ok(presigned.uri().to_string())
    }

    async fn presigned_put_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, anyhow::Error> {
        let config = PresigningConfig::expires_in(expires_in)?;
        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(config)
            .await?;
        Ok(presigned.uri().to_string())
    }

    async fn delete_object(&self, key: &str) -> Result<(), anyhow::Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}

#[async_trait]
impl StorageProvider for GcpStorage {
    async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, anyhow::Error> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await?;
        Ok(key.to_string())
    }

    async fn presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, anyhow::Error> {
        let config = PresigningConfig::expires_in(expires_in)?;
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(config)
            .await?;
        Ok(presigned.uri().to_string())
    }

    async fn presigned_put_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, anyhow::Error> {
        let config = PresigningConfig::expires_in(expires_in)?;
        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(config)
            .await?;
        Ok(presigned.uri().to_string())
    }

    async fn delete_object(&self, key: &str) -> Result<(), anyhow::Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}

pub struct MediaService {
    provider: Box<dyn StorageProvider + Send + Sync>,
}

impl MediaService {
    pub async fn new_from_env() -> Self {
        let backend = std::env::var("STORAGE_BACKEND")
            .unwrap_or_else(|_| "aws".to_string())
            .to_lowercase();

        let provider: Box<dyn StorageProvider + Send + Sync> = match backend.as_str() {
            "gcp" | "gcs" => Box::new(GcpStorage::from_env().await),
            _ => Box::new(AwsS3Storage::from_env().await),
        };

        Self { provider }
    }

    pub async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, anyhow::Error> {
        self.provider.upload(key, data, content_type).await
    }

    pub async fn presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, anyhow::Error> {
        self.provider.presigned_url(key, expires_in).await
    }

    pub async fn presigned_put_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, anyhow::Error> {
        self.provider.presigned_put_url(key, expires_in).await
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), anyhow::Error> {
        self.provider.delete_object(key).await
    }
}

fn read_env(name: &str, default: Option<&str>) -> String {
    std::env::var(name)
        .or_else(|_| read_storage_alias(name))
        .unwrap_or_else(|_| default.unwrap_or_default().to_string())
}

fn read_storage_alias(name: &str) -> Result<String, std::env::VarError> {
    match name {
        "S3_ENDPOINT" => std::env::var("STORAGE_ENDPOINT"),
        "S3_BUCKET" => std::env::var("STORAGE_BUCKET"),
        "S3_REGION" => std::env::var("STORAGE_REGION"),
        _ => Err(std::env::VarError::NotPresent),
    }
}

fn read_credential(primary: &str, fallback: &str) -> Option<String> {
    std::env::var(primary)
        .or_else(|_| std::env::var(fallback))
        .ok()
}
