//! Tiny CLI program to dump information about kartlytics videos

use anyhow::Context;
use rusoto_core::Region;
use rusoto_s3::GetObjectRequest;
use rusoto_s3::S3Client;
use rusoto_s3::S3;
use serde::Deserialize;
use structopt::StructOpt;
use tokio::io::AsyncReadExt;

#[macro_use]
extern crate anyhow;

/// Describes the command-line arguments for this program.
#[derive(Debug, StructOpt)]
#[structopt(about = "Print list of kartlytics videos")]
struct Demo {
    /// AWS region for S3 bucket
    #[structopt(long, default_value = "us-east-1", parse(try_from_str))]
    region: Region,
    /// AWS bucket containing the object to fetch
    bucket: String,
    /// AWS key identifying the object to fetch
    key: String,
    /// Maximum number of videos to print
    #[structopt(long, default_value = "10")]
    max_videos: usize,
}

/// Describes the videos in the Kartlytics summary.
#[derive(Deserialize)]
struct KartlyticsVideo {
    id: String,
    uploaded: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Setup.
    let _ = env_logger::try_init();
    let app = Demo::from_args();
    let s3 = S3Client::new(app.region.clone());

    // Make the HTTP request to S3.
    let object_output = s3
        .get_object(GetObjectRequest {
            bucket: app.bucket.clone(),
            key: app.key.clone(),
            ..Default::default()
        })
        .await
        .with_context(|| {
            format!("fetching bucket {:?} key {:?}", app.bucket, app.key)
        })?;

    // Read the body.
    let mut body = object_output
        .body
        .ok_or_else(|| anyhow!("object missing body"))?
        .into_async_read();
    let mut contents = String::new();
    body.read_to_string(&mut contents).await.context("reading object")?;

    // Parse the body as JSON.
    let summary: Vec<KartlyticsVideo> =
        serde_json::from_str(&contents).context("parsing object")?;

    // Print a summary.
    println!("printing up to {} videos", app.max_videos);
    println!("{:32} {}", "ID", "UPLOADED");
    for vid in summary.iter().take(app.max_videos) {
        println!("{:32} {}", vid.id, vid.uploaded);
    }

    Ok(())
}
