extern crate dotenv;
extern crate s3;

use advisory_lock::{AdvisoryFileLock, FileLockMode};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use chrono::Utc;
use env_logger::Env;
use hmac::{Hmac, Mac};
use log::{error, info, warn};
use reqwest::header::{
    AUTHORIZATION, CONTENT_LENGTH, HeaderMap, HeaderValue,
};
use reqwest::Client;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use sha2::Sha256;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::time::Duration;
use thiserror::Error;
use tokio_cron_scheduler::{Job, JobScheduler};
use tokio_util::io::ReaderStream;

#[cfg(unix)]
use inotify::{EventMask, Inotify, WatchMask};

type HmacSha256 = Hmac<Sha256>;

enum StorageBackend {
    S3(Bucket),
    Azure(AzureBlobClient),
}

struct UploadResult {
    provider: &'static str,
    status_code: u16,
    uploaded_bytes: u64,
}

struct AzureBlobClient {
    client: Client,
    auth: AzureAuth,
    container_name: String,
    blob_endpoint: String,
    blob_prefix: String,
}

enum AzureAuth {
    SharedKey {
        account_name: String,
        account_key: String,
    },
    ManagedIdentity {
        client_id: String,
        tenant_id: Option<String>,
        federated_token_file: Option<String>,
        authority_host: String,
    },
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to set (name {name:?}, value {value:?})")]
    InvalidOverWrite { name: String, value: String },
}

const BIN_PATH: &str = "/bin:/sbin:/usr/bin:/usr/sbin:/usr/local/bin";
const CDC_NAME: &str = "cdc";
static DEFAULT_BASE_DIR: &str = "/var/mnt/core-dump-handler";
static DEFAULT_CORE_DIR: &str = "/var/mnt/core-dump-handler/cores";
const AZURE_BLOB_API_VERSION: &str = "2023-11-03";

static DEFAULT_SUID_DUMPABLE: &str = "2";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut env_path = env::current_exe()?;
    env_path.pop();
    env_path.push(".env");

    let mut envloadmsg = String::from("Loading .env");
    match dotenv::from_path(env_path) {
        Ok(v) => v,
        Err(_) => {
            envloadmsg = "no .env file found \n That's ok if running in kubernetes".to_string()
        }
    }

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let host_dir = env::var("HOST_DIR").unwrap_or_else(|_| DEFAULT_BASE_DIR.to_string());
    let core_dir_command = env::var("CORE_DIR").unwrap_or_else(|_| DEFAULT_CORE_DIR.to_string());
    let suid = env::var("SUID_DUMPABLE").unwrap_or_else(|_| DEFAULT_SUID_DUMPABLE.to_string());
    let deploy_crio_config = env::var("DEPLOY_CRIO_CONFIG")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();

    let deploy_crio_exe = env::var("DEPLOY_CRIO_EXE")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();

    let host_location = host_dir.as_str();
    let pattern: String = std::env::args().nth(1).unwrap_or_default();

    info!("{}", envloadmsg);
    // Catch remove command for uninstall options
    if pattern == "remove" {
        info!("Removing {}", host_location);
        remove()?;
        process::exit(0);
    }

    if pattern == "sweep" {
        let file = std::env::args().nth(2).unwrap_or_default();
        if !file.is_empty() {
            let storage = match get_storage_backend() {
                Ok(v) => v,
                Err(e) => {
                    error!("Storage client creation failed in sweep: {}", e);
                    process::exit(1);
                }
            };
            let p = Path::new(&file);
            info!("Uploading {}", file);
            process_file(p, &storage).await;
        } else {
            info!("Uploading all content in {}", core_dir_command);
            run_polling_agent().await;
        }
        process::exit(0);
    }
    info!("Setting host location to: {}", host_location);
    info!(
        "Current Directory for setup is {}",
        env::current_dir().unwrap().display()
    );

    if deploy_crio_config == "true" {
        generate_crio_config(host_location)?;
    }

    if deploy_crio_exe == "true" {
        copy_crictl_to_hostdir(host_location)?;
    }
    copy_core_dump_composer_to_hostdir(host_location)?;
    apply_sysctl(
        "kernel.core_pattern",
        format!("{host_location}/core_pattern.bak").as_str(),
        format!(
            "|{host_location}/{CDC_NAME} -c=%c -e=%e -p=%p -s=%s -t=%t -d={core_dir_command} -h=%h -E=%E")
        .as_str(),
    )?;
    apply_sysctl(
        "kernel.core_pipe_limit",
        format!("{host_location}/core_pipe_limit.bak").as_str(),
        "128",
    )?;

    apply_sysctl(
        "fs.suid_dumpable",
        format!("{host_location}/suid_dumpable.bak").as_str(),
        &suid,
    )?;

    create_env_file(host_location)?;
    // Run polling agent on startup to clean up files.

    let interval = env::var("INTERVAL").unwrap_or_else(|_| String::from(""));
    let mut schedule = env::var("SCHEDULE").unwrap_or_else(|_| String::from(""));

    let use_inotify = env::var("USE_INOTIFY")
        .unwrap_or_else(|_| String::from("false"))
        .to_lowercase();

    // Allow no uploads when none of the upload configs have been supplied
    // then we are leaving the cores on the box
    if interval.is_empty() && schedule.is_empty() && use_inotify == "false" {
        // let it spin
        loop {
            std::thread::sleep(Duration::from_millis(1000));
        }
    } else {
        run_polling_agent().await;
    }

    if !interval.is_empty() && !schedule.is_empty() {
        warn!(
            "Interval set to: {}, Schedule set to:{}\n Using interval:{}",
            interval, schedule, interval
        );
    }

    if !interval.is_empty() {
        let mut i_interval = match interval.parse::<u64>() {
            Ok(v) => v,
            Err(e) => {
                error!("Error parsing interval : {interval} Error: {e}");
                panic!("Error parsing interval {e}");
            }
        };
        i_interval /= 1000;
        schedule = format!("1/{i_interval} * * * * *");
        if use_inotify == "true" {
            warn!("Both schedule and INotify set. Running schedule")
        }
    }

    if !schedule.is_empty() {
        info!("Schedule Initialising with: {}", schedule);
        let sched = match JobScheduler::new().await {
            Ok(v) => v,
            Err(e) => {
                error!("Schedule Creation Failed with {e}");
                panic!("Schedule Creation Failed with {e}")
            }
        };

        let s_job = match Job::new_async(schedule.as_str(), move |uuid, mut l| {
            Box::pin(async move {
                let next_tick = l.next_tick_for_job(uuid).await;
                match next_tick {
                    Ok(Some(ts)) => {
                        info!("Next scheduled run {:?}", ts);
                        run_polling_agent().await;
                    }
                    _ => warn!("Could not get next tick for job"),
                }
            })
        }) {
            Ok(v) => v,
            Err(e) => {
                error!("Schedule Job Creation with {schedule} failed, {e}");
                panic!("Schedule Job Creation with {schedule} failed, {e}")
            }
        };
        info!("Created Schedule job: {:?}", s_job.guid());
        match sched.add(s_job).await {
            Ok(v) => v,
            Err(e) => {
                error!("Job Add failed {e:#?}");
                panic!("Job Scheduing failed, {e:#?}");
            }
        };
        match sched.start().await {
            Ok(v) => v,
            Err(e) => {
                error!("Schedule Start failed {e:#?}");
                panic!("Schedule Start failed, {e:#?}");
            }
        };
        loop {
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    #[cfg(unix)]
    if use_inotify == "true" {
        info!("INotify Starting...");
        let inotify_task = tokio::spawn(async move {
            let mut inotify = match Inotify::init() {
                Ok(v) => v,
                Err(e) => {
                    error!("Inotify init failed: {e}");
                    panic!("Inotify init failed: {e}")
                }
            };
            info!("INotify Initialised...");
            match inotify.watches().add(&core_dir_command, WatchMask::CLOSE) {
                Ok(_) => {}
                Err(e) => {
                    error!("Add watch failed: {e}");
                    panic!("Add watch failed: {e}")
                }
            };
            info!("INotify watching : {}", core_dir_command);
            let mut buffer = [0; 4096];
            loop {
                let events = match inotify.read_events_blocking(&mut buffer) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("read events failed: {}", e);
                        continue;
                    }
                };
                for event in events {
                    if event.mask.contains(EventMask::CLOSE_WRITE) {
                        if event.mask.contains(EventMask::ISDIR) {
                            warn!("Unknown Directory created: {:?}", event.name);
                        } else {
                            let storage = match get_storage_backend() {
                                Ok(v) => v,
                                Err(e) => {
                                    error!("Storage client creation failed in event: {}", e);
                                    continue;
                                }
                            };
                            match event.name {
                                Some(s) => {
                                    let file = format!(
                                        "{}/{}",
                                        core_dir_command,
                                        s.to_str().unwrap_or_default()
                                    );
                                    let p = Path::new(&file);
                                    process_file(p, &storage).await
                                }
                                None => {
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        });
        inotify_task.await?;
    }

    #[cfg(not(unix))]
    if use_inotify == "true" {
        warn!("USE_INOTIFY is not supported on this platform; using polling instead");
    }

    Ok(())
}

async fn process_file(zip_path: &Path, storage: &StorageBackend) {
    info!("Uploading: {}", zip_path.display());

    let f = File::open(zip_path).expect("no file found");

    match AdvisoryFileLock::try_lock(&f, FileLockMode::Shared) {
        Ok(_) => { /* If we can lock then we are ok */ }
        Err(e) => {
            let l_inotify = env::var("USE_INOTIFY")
                .unwrap_or_else(|_| String::from("false"))
                .to_lowercase();
            if l_inotify == "false" {
                info!("File locked so we are ignoring it for this iteration {}", e);
            } else {
                error!("File locked on INotify shouldn't happen as we are waiting for file close events.\nPlease recycling pod to perform sweep\n{}", e);
            }
            return;
        }
    }

    let metadata = fs::metadata(zip_path).expect("unable to read metadata");
    info!("zip size is {}", metadata.len());
    let path_str = match zip_path.to_str() {
        Some(v) => v,
        None => {
            error!("Failed to extract path");
            return;
        }
    };
    let upload_file_name: &str = match zip_path.file_name().unwrap().to_str() {
        Some(v) => v,
        None => {
            error!("Failed to get file name for upload");
            return;
        }
    };

    let fasync = tokio::fs::File::open(zip_path)
        .await
        .expect("file was removed");

    let upload_result = match storage
        .upload(upload_file_name, metadata.len(), fasync)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("Upload Failed {}", e);
            return;
        }
    };
    match fs::remove_file(path_str) {
        Ok(v) => v,
        Err(e) => {
            error!("File delete failed: {}", e);
            return;
        }
    };
    info!(
        "{} returned: status_code: {} uploaded_bytes: {}",
        upload_result.provider,
        upload_result.status_code,
        upload_result.uploaded_bytes
    );
}

impl StorageBackend {
    async fn upload(
        &self,
        upload_file_name: &str,
        content_length: u64,
        file: tokio::fs::File,
    ) -> Result<UploadResult, anyhow::Error> {
        match self {
            StorageBackend::S3(bucket) => {
                let mut file = file;
                let response = bucket.put_object_stream(&mut file, upload_file_name).await?;
                Ok(UploadResult {
                    provider: "S3",
                    status_code: response.status_code(),
                    uploaded_bytes: response.uploaded_bytes() as u64,
                })
            }
            StorageBackend::Azure(client) => client.upload(upload_file_name, content_length, file).await,
        }
    }
}

impl AzureBlobClient {
    async fn upload(
        &self,
        upload_file_name: &str,
        content_length: u64,
        file: tokio::fs::File,
    ) -> Result<UploadResult, anyhow::Error> {
        let blob_name = self.blob_name(upload_file_name);
        let request_url = self.blob_url(&blob_name);
        let request_date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        let authorization = self.authorization_header(&blob_name, content_length, &request_date).await?;

        let mut headers = HeaderMap::new();
        headers.insert("x-ms-blob-type", HeaderValue::from_static("BlockBlob"));
        headers.insert(
            "x-ms-date",
            HeaderValue::from_str(&request_date).map_err(|error| anyhow::Error::msg(error.to_string()))?,
        );
        headers.insert("x-ms-version", HeaderValue::from_static(AZURE_BLOB_API_VERSION));
        headers.insert(CONTENT_LENGTH, HeaderValue::from(content_length));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&authorization).map_err(|error| anyhow::Error::msg(error.to_string()))?,
        );

        let response = self
            .client
            .put(request_url)
            .headers(headers)
            .body(reqwest::Body::wrap_stream(ReaderStream::new(file)))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Azure Blob upload failed with {}: {}", status, body);
        }

        Ok(UploadResult {
            provider: "Azure Blob",
            status_code: status.as_u16(),
            uploaded_bytes: content_length,
        })
    }

    fn blob_name(&self, upload_file_name: &str) -> String {
        if self.blob_prefix.is_empty() {
            upload_file_name.to_string()
        } else {
            format!("{}/{}", self.blob_prefix, upload_file_name)
        }
    }

    fn blob_url(&self, blob_name: &str) -> String {
        let encoded_blob_name = blob_name
            .split('/')
            .map(urlencoding::encode)
            .collect::<Vec<_>>()
            .join("/");
        format!(
            "{}/{}/{}",
            self.blob_endpoint.trim_end_matches('/'),
            self.container_name,
            encoded_blob_name
        )
    }

    async fn authorization_header(
        &self,
        blob_name: &str,
        content_length: u64,
        request_date: &str,
    ) -> Result<String, anyhow::Error> {
        match &self.auth {
            AzureAuth::SharedKey {
                account_name,
                account_key,
            } => {
                let canonicalized_headers = format!(
                    "x-ms-blob-type:BlockBlob\nx-ms-date:{}\nx-ms-version:{}",
                    request_date, AZURE_BLOB_API_VERSION
                );
                let canonicalized_resource = format!(
                    "/{}/{}/{}",
                    account_name, self.container_name, blob_name
                );
                let string_to_sign = format!(
                    "PUT\n\n\n{}\n\n\n\n\n\n\n\n\n{}\n{}",
                    content_length, canonicalized_headers, canonicalized_resource
                );

                let decoded_key = BASE64_STANDARD.decode(account_key.as_bytes())?;
                let mut mac = HmacSha256::new_from_slice(&decoded_key)?;
                mac.update(string_to_sign.as_bytes());
                let signature = BASE64_STANDARD.encode(mac.finalize().into_bytes());
                Ok(format!("SharedKey {}:{}", account_name, signature))
            }
            AzureAuth::ManagedIdentity {
                client_id,
                tenant_id,
                federated_token_file,
                authority_host,
            } => {
                let token = if let Some(token_file) = federated_token_file {
                    let tenant_id = tenant_id.as_ref().ok_or_else(|| {
                        anyhow::anyhow!(
                            "AZURE_TENANT_ID is required when AZURE_FEDERATED_TOKEN_FILE is set"
                        )
                    })?;
                    self.exchange_federated_token(
                        authority_host,
                        tenant_id,
                        client_id,
                        token_file,
                    )
                    .await?
                } else {
                    self.get_imds_token(client_id).await?
                };

                Ok(format!("Bearer {}", token))
            }
        }
    }

    async fn exchange_federated_token(
        &self,
        authority_host: &str,
        tenant_id: &str,
        client_id: &str,
        token_file: &str,
    ) -> Result<String, anyhow::Error> {
        let assertion = fs::read_to_string(token_file)?;
        let url = format!(
            "{}/{}/oauth2/v2.0/token",
            authority_host.trim_end_matches('/'),
            tenant_id
        );

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "client_id={}&scope={}&grant_type=client_credentials&client_assertion_type=urn:ietf:params:oauth:client-assertion-type:jwt-bearer&client_assertion={}",
                urlencoding::encode(client_id),
                urlencoding::encode("https://storage.azure.com/.default"),
                urlencoding::encode(assertion.trim())
            ))
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            anyhow::bail!("Azure federated token exchange failed with {}: {}", status, body);
        }

        let value: serde_json::Value = serde_json::from_str(&body)?;
        value
            .get("access_token")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .ok_or_else(|| anyhow::anyhow!("Azure federated token response did not contain access_token"))
    }

    async fn get_imds_token(&self, client_id: &str) -> Result<String, anyhow::Error> {
        let mut url = reqwest::Url::parse(
            "http://169.254.169.254/metadata/identity/oauth2/token",
        )?;
        url.query_pairs_mut()
            .append_pair("api-version", "2018-02-01")
            .append_pair("resource", "https://storage.azure.com/")
            .append_pair("client_id", client_id);

        let response = self
            .client
            .get(url)
            .header("Metadata", "true")
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            anyhow::bail!("Azure IMDS token request failed with {}: {}", status, body);
        }

        let value: serde_json::Value = serde_json::from_str(&body)?;
        value
            .get("access_token")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .ok_or_else(|| anyhow::anyhow!("Azure IMDS token response did not contain access_token"))
    }
}

fn get_storage_backend() -> Result<StorageBackend, anyhow::Error> {
    let provider = env::var("STORAGE_PROVIDER")
        .unwrap_or_else(|_| String::new())
        .to_lowercase();

    if provider == "azure" || (provider.is_empty() && has_azure_config()) {
        return Ok(StorageBackend::Azure(get_azure_blob_client()?));
    }

    Ok(StorageBackend::S3(get_bucket()?))
}

fn has_azure_config() -> bool {
    env::var("AZURE_CLIENT_ID").is_ok()
        || env::var("AZURE_STORAGE_CONNECTION_STRING").is_ok()
        || (env::var("AZURE_STORAGE_ACCOUNT_NAME").is_ok()
            && env::var("AZURE_STORAGE_CONTAINER_NAME").is_ok())
}

fn get_bucket() -> Result<Bucket, anyhow::Error> {
    let s3_access_key = env::var("S3_ACCESS_KEY").unwrap_or_default();
    let s3_secret = env::var("S3_SECRET").unwrap_or_default();
    let s3_bucket_name = env::var("S3_BUCKET_NAME").unwrap_or_default();
    let s3_region = env::var("S3_REGION").unwrap_or_default();

    let custom_endpoint = env::var("S3_ENDPOINT").unwrap_or_default();

    let region = if custom_endpoint.is_empty() {
        s3_region.parse().unwrap()
    } else {
        info!("Setting s3 endpoint location to: {}", custom_endpoint);

        Region::Custom {
            region: s3_region,
            endpoint: custom_endpoint,
        }
    };

    let credentials = if env::var("AWS_WEB_IDENTITY_TOKEN_FILE").is_ok() {
        Credentials::from_sts_env(std::env!("CARGO_PKG_NAME"))
    } else if s3_access_key.is_empty() || s3_secret.is_empty() {
        Credentials::new(None, None, None, None, None)
    } else {
        Credentials::new(
            Some(s3_access_key.as_str()),
            Some(s3_secret.as_str()),
            None,
            None,
            None,
        )
    }?;

    Ok(Bucket::new(&s3_bucket_name, region, credentials)?.with_path_style())
}

fn get_azure_blob_client() -> Result<AzureBlobClient, anyhow::Error> {
    let connection_string = env::var("AZURE_STORAGE_CONNECTION_STRING").unwrap_or_default();
    let mut blob_endpoint = env::var("AZURE_STORAGE_BLOB_ENDPOINT").unwrap_or_default();
    let mut account_name = env::var("AZURE_STORAGE_ACCOUNT_NAME").unwrap_or_default();
    let mut account_key = env::var("AZURE_STORAGE_ACCOUNT_KEY").unwrap_or_default();
    let client_id = env::var("AZURE_CLIENT_ID").unwrap_or_default();
    let tenant_id = env::var("AZURE_TENANT_ID").ok();
    let federated_token_file = env::var("AZURE_FEDERATED_TOKEN_FILE").ok();
    let authority_host = env::var("AZURE_AUTHORITY_HOST")
        .unwrap_or_else(|_| "https://login.microsoftonline.com".to_string());

    if !connection_string.is_empty() {
        let values = parse_connection_string(&connection_string);
        if account_name.is_empty() {
            account_name = values.get("AccountName").cloned().unwrap_or_default();
        }
        if account_key.is_empty() {
            account_key = values.get("AccountKey").cloned().unwrap_or_default();
        }
        if blob_endpoint.is_empty() {
            blob_endpoint = values.get("BlobEndpoint").cloned().unwrap_or_default();
            if blob_endpoint.is_empty() {
                let endpoint_suffix = values
                    .get("EndpointSuffix")
                    .cloned()
                    .unwrap_or_else(|| "core.windows.net".to_string());
                let default_protocol = values
                    .get("DefaultEndpointsProtocol")
                    .cloned()
                    .unwrap_or_else(|| "https".to_string());
                if !account_name.is_empty() {
                    blob_endpoint = format!(
                        "{}://{}.blob.{}",
                        default_protocol, account_name, endpoint_suffix
                    );
                }
            }
        }
    }

    let container_name = env::var("AZURE_STORAGE_CONTAINER_NAME")?;
    let blob_prefix = env::var("AZURE_STORAGE_BLOB_PREFIX")
        .unwrap_or_default()
        .trim_matches('/')
        .to_string();

    let auth = if !client_id.is_empty() {
        if blob_endpoint.is_empty() {
            anyhow::bail!(
                "Azure managed identity requires AZURE_STORAGE_BLOB_ENDPOINT or AZURE_STORAGE_CONNECTION_STRING to derive it"
            );
        }

        AzureAuth::ManagedIdentity {
            client_id,
            tenant_id,
            federated_token_file,
            authority_host,
        }
    } else {
        if account_name.is_empty() || account_key.is_empty() || blob_endpoint.is_empty() {
            anyhow::bail!(
                "Azure Storage requires AZURE_STORAGE_CONTAINER_NAME and either AZURE_CLIENT_ID or AZURE_STORAGE_CONNECTION_STRING / AZURE_STORAGE_ACCOUNT_NAME / AZURE_STORAGE_ACCOUNT_KEY / AZURE_STORAGE_BLOB_ENDPOINT"
            );
        }

        AzureAuth::SharedKey {
            account_name,
            account_key,
        }
    };

    if blob_endpoint.is_empty() {
        anyhow::bail!(
            "Azure Storage requires AZURE_STORAGE_BLOB_ENDPOINT or AZURE_STORAGE_CONNECTION_STRING to derive it"
        );
    }

    Ok(AzureBlobClient {
        client: Client::builder().build()?,
        auth,
        container_name,
        blob_endpoint,
        blob_prefix,
    })
}

async fn run_polling_agent() {
    let core_location = env::var("CORE_DIR").unwrap_or_else(|_| DEFAULT_CORE_DIR.to_string());
    info!("Executing Agent with location : {}", core_location);

    let storage = match get_storage_backend() {
        Ok(v) => v,
        Err(e) => {
            error!("Storage client creation failed: {}", e);
            return;
        }
    };

    let dir = fs::read_dir(core_location).unwrap();

    let paths: Vec<PathBuf> = dir
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| !r.is_dir())
        .collect();

    info!("Dir Content {:?}", paths);
    for zip_path in paths {
        process_file(&zip_path, &storage).await;
    }
}

fn parse_connection_string(connection_string: &str) -> HashMap<String, String> {
    connection_string
        .split(';')
        .filter_map(|entry| {
            let (key, value) = entry.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

#[cfg(test)]
mod unit_tests {
    use super::parse_connection_string;

    #[test]
    fn parses_azure_connection_string() {
        let values = parse_connection_string(
            "DefaultEndpointsProtocol=https;AccountName=testacct;AccountKey=dGVzdA==;EndpointSuffix=core.windows.net",
        );

        assert_eq!(values.get("AccountName"), Some(&"testacct".to_string()));
        assert_eq!(values.get("AccountKey"), Some(&"dGVzdA==".to_string()));
        assert_eq!(
            values.get("EndpointSuffix"),
            Some(&"core.windows.net".to_string())
        );
    }
}

fn generate_crio_config(host_location: &str) -> Result<(), std::io::Error> {
    info!("Generating crio file");
    let endpoint = env::var("CRIO_ENDPOINT")
        .unwrap_or_else(|_| "unix:///run/containerd/containerd.sock".to_string());
    let destination = format!("{}/{}", host_location, "crictl.yaml");
    let mut crictl_file = File::create(destination)?;
    let text = format!("runtime-endpoint: {endpoint}\nimage-endpoint: {endpoint}\ntimeout: 2\ndebug: false\npull-image-on-create: false");
    crictl_file.write_all(text.as_bytes())?;
    crictl_file.flush()?;
    Ok(())
}

fn copy_crictl_to_hostdir(host_location: &str) -> Result<(), std::io::Error> {
    let location = "./crictl".to_string();
    let destination = format!("{}/{}", host_location, "crictl");
    info!("Copying the crictl from {} to {}", location, destination);
    fs::copy(location, destination)?;
    Ok(())
}

fn copy_core_dump_composer_to_hostdir(host_location: &str) -> Result<(), std::io::Error> {
    let version = env::var("VENDOR").unwrap_or_else(|_| "default".to_string());
    match version.to_lowercase().as_str() {
        "default" => {
            let location = format!("./vendor/default/{CDC_NAME}");
            let destination = format!("{host_location}/{CDC_NAME}");
            info!("Copying the composer from {} to {}", location, destination);
            fs::copy(location, destination)?;
        }
        "rhel7" => {
            let location = format!("./vendor/rhel7/{CDC_NAME}");
            let destination = format!("{host_location}/{CDC_NAME}");
            info!("Copying the composer from {} to {}", location, destination);
            fs::copy(location, destination)?;
        }
        _ => {
            error!("Unknown vendor: {}", version);
            process::exit(1);
        }
    }
    Ok(())
}

fn create_env_file(host_location: &str) -> Result<(), std::io::Error> {
    let loglevel = env::var("COMP_LOG_LEVEL").unwrap_or_else(|_| "debug".to_string());
    let ignore_crio = env::var("COMP_IGNORE_CRIO")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();
    let include_proc_info = env::var("COMP_INCLUDE_PROC_INFO")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();
    let crio_image = env::var("COMP_CRIO_IMAGE_CMD").unwrap_or_else(|_| "img".to_string());
    let destination = format!("{}/{}", host_location, ".env");
    let use_crio_config = env::var("DEPLOY_CRIO_CONFIG")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();
    let filename_template = env::var("COMP_FILENAME_TEMPLATE").unwrap_or_else(|_| {
        "{uuid}-dump-{timestamp}-{hostname}-{exe_name}-{pid}-{signal}".to_string()
    });
    let log_length = env::var("COMP_LOG_LENGTH").unwrap_or_else(|_| "500".to_string());
    let pod_selector_label = env::var("COMP_POD_SELECTOR_LABEL").unwrap_or_default();
    let timeout = env::var("COMP_TIMEOUT").unwrap_or_else(|_| "600".to_string());

    let compression = env::var("COMP_COMPRESSION")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase();

    let core_events = env::var("COMP_CORE_EVENTS")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();

    let event_directory = env::var("COMP_CORE_EVENT_DIR")
        .unwrap_or_else(|_| format!("{}/{}", host_location, "events"))
        .to_lowercase();
    info!("Creating {} file with LOG_LEVEL={}", destination, loglevel);
    let mut env_file = File::create(destination)?;
    let text = format!(
        "LOG_LEVEL={loglevel}\nIGNORE_CRIO={ignore_crio}\nINCLUDE_PROC_INFO={include_proc_info}\nCRIO_IMAGE_CMD={crio_image}\nUSE_CRIO_CONF={use_crio_config}\nFILENAME_TEMPLATE={filename_template}\nLOG_LENGTH={log_length}\nPOD_SELECTOR_LABEL={pod_selector_label}\nTIMEOUT={timeout}\nCOMPRESSION={compression}\nCORE_EVENTS={core_events}\nEVENT_DIRECTORY={event_directory}\n");
    info!("Writing composer .env \n{}", text);
    env_file.write_all(text.as_bytes())?;
    env_file.flush()?;
    Ok(())
}

fn get_path() -> String {
    let mut local_bin = env::var("LOCAL_BIN").unwrap_or_else(|_| "".to_string());
    local_bin.push(':');
    local_bin.push_str(BIN_PATH);
    local_bin
}

fn get_sysctl(name: &str) -> Result<String, anyhow::Error> {
    info!("Getting sysctl for {}", name);
    let output = Command::new("sysctl")
        .env("PATH", get_path())
        .args(["-n", name])
        .output()?;
    let lines = String::from_utf8(output.stdout)?;
    let line = lines.lines().take(1).next().unwrap_or("");
    Ok(line.to_string())
}
fn apply_sysctl(name: &str, location: &str, value: &str) -> Result<(), anyhow::Error> {
    info!("Starting sysctl for {} {} with {}", name, location, value);
    let ctl = get_sysctl(name)?;
    // The values are different so let's back up and apply
    if ctl.as_str() != value {
        let mut file = File::create(location)?;
        file.write_all(ctl.as_bytes())?;
        file.flush()?;
        info!("Created Backup of {}", location);
        overwrite_sysctl(name, value)?
    } else {
        info!("{} with value {} is already applied", name, ctl);
    }
    Ok(())
}

fn overwrite_sysctl(name: &str, value: &str) -> Result<(), anyhow::Error> {
    let s = format!("{name}={value}");
    let output = Command::new("sysctl")
        .env("PATH", get_path())
        .args(["-w", s.as_str()])
        .status()?;
    if !output.success() {
        let e = Error::InvalidOverWrite {
            name: name.to_string(),
            value: value.to_string(),
        };
        return Err(anyhow::Error::new(e));
    }
    Ok(())
}

fn remove() -> Result<(), anyhow::Error> {
    restore_sysctl("kernel", "core_pattern")?;
    restore_sysctl("kernel", "core_pipe_limit")?;
    restore_sysctl("fs", "suid_dumpable")?;
    let host_dir = env::var("HOST_DIR").unwrap_or_else(|_| DEFAULT_BASE_DIR.to_string());
    let exe = format!("{host_dir}/{CDC_NAME}");
    let env_file = format!("{host_dir}/.env");
    let crictl_file = format!("{host_dir}/crictl.yaml");
    let composer_file = format!("{host_dir}/composer.log");
    let crictl_exe = format!("{host_dir}/crictl");

    fs::remove_file(exe)?;
    fs::remove_file(env_file)?;

    if !Path::new(&crictl_exe).exists() {
        fs::remove_file(crictl_exe)?;
    }
    if !Path::new(&crictl_file).exists() {
        fs::remove_file(crictl_file)?;
    }
    if !Path::new(&composer_file).exists() {
        fs::remove_file(composer_file)?;
    }

    Ok(())
}
fn restore_sysctl(prefix: &str, name: &str) -> Result<(), anyhow::Error> {
    info!("Restoring Backup of {}", name);
    let host_dir = env::var("HOST_DIR").unwrap_or_else(|_| DEFAULT_BASE_DIR.to_string());
    let file_name = format!("{host_dir}/{name}.bak");
    let sysctl_name = format!("{prefix}.{name}");
    let line = fs::read_to_string(&file_name)?;
    overwrite_sysctl(sysctl_name.as_str(), line.as_str())?;
    fs::remove_file(file_name)?;

    Ok(())
}
