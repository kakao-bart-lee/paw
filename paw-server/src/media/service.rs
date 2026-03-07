use aws_credential_types::Credentials;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use std::time::Duration;

enum StorageBackend {
    AwsCompatible,
    Gcp,
}

impl StorageBackend {
    fn from_env() -> Self {
        match std::env::var("STORAGE_BACKEND")
            .unwrap_or_else(|_| "aws".to_string())
            .to_lowercase()
            .as_str()
        {
            "gcp" | "gcs" => Self::Gcp,
            _ => Self::AwsCompatible,
        }
    }

    fn endpoint(&self) -> String {
        std::env::var("S3_ENDPOINT")
            .or_else(|_| std::env::var("STORAGE_ENDPOINT"))
            .unwrap_or_else(|_| match self {
                Self::AwsCompatible => "http://localhost:9000".to_string(),
                Self::Gcp => "https://storage.googleapis.com".to_string(),
            })
    }

    fn bucket(&self) -> String {
        std::env::var("S3_BUCKET")
            .or_else(|_| std::env::var("STORAGE_BUCKET"))
            .unwrap_or_else(|_| "paw-media".to_string())
    }

    fn region(&self) -> String {
        std::env::var("S3_REGION")
            .or_else(|_| std::env::var("STORAGE_REGION"))
            .unwrap_or_else(|_| "us-east-1".to_string())
    }

    fn force_path_style(&self) -> bool {
        match self {
            Self::Gcp => false,
            Self::AwsCompatible => std::env::var("S3_FORCE_PATH_STYLE")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "True"))
                .unwrap_or(true),
        }
    }

    fn credentials(self) -> Option<Credentials> {
        let access_key = std::env::var("AWS_ACCESS_KEY_ID")
            .or_else(|_| std::env::var("S3_ACCESS_KEY"))
            .ok();
        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY")
            .or_else(|_| std::env::var("S3_SECRET_KEY"))
            .ok();

        match (access_key, secret_key) {
            (Some(access), Some(secret)) if !access.is_empty() && !secret.is_empty() => {
                Some(Credentials::new(access, secret, None, None, "env"))
            }
            _ => match self {
                Self::AwsCompatible => Some(Credentials::new(
                    "paw_minio".to_string(),
                    "paw_minio_password".to_string(),
                    None,
                    None,
                    "env",
                )),
                Self::Gcp => None,
            },
        }
    }
}

pub struct MediaService {
    s3_client: aws_sdk_s3::Client,
    bucket: String,
}

impl MediaService {
    pub async fn new_from_env() -> Self {
        let backend = StorageBackend::from_env();
        let endpoint = backend.endpoint();
        let bucket = backend.bucket();
        let region = backend.region();
        let mut config_builder = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region))
            .endpoint_url(&endpoint);

        if let Some(credentials) = backend.credentials() {
            config_builder = config_builder.credentials_provider(credentials);
        }

        let shared_config = config_builder.load().await;
        let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
            .force_path_style(backend.force_path_style())
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
