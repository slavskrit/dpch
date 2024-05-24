use chrono::NaiveDateTime;
use db::Database;
use log::{error, info};
use rand::distributions::{Alphanumeric, DistString};
use std::env;
use std::io::Result;
use std::process::Command;
use std::sync::Arc;
use teloxide::payloads::SendVideoSetters;
use teloxide::prelude::*;
use teloxide::types::InputFile;

use std::fs::metadata;
use std::io;
use std::path::Path;

fn get_file_size<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    let metadata = metadata(path)?;
    Ok(metadata.len())
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
}

static INSTAGRAM_MAIN_URL: &str = "https://www.instagram.com/";

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();
    let postgres_user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    let postgres_password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let postgres_db = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
    let database_url = format!("postgres://{postgres_user}:{postgres_password}@postgres:5432/");

    info!("Starting DPCH bot...");
    info!("DB at: {database_url}");

    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    info!("Start Time: {start}");

    let db = Arc::new(Database::new(&database_url, &postgres_db).await.unwrap());
    db.create_table().await.unwrap();

    let bot = Bot::from_env();
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let db = db.clone();
        async move {
            match msg.text().map(|text| text.contains(INSTAGRAM_MAIN_URL)) {
                Some(true) => {
                    let link = msg.text().unwrap();
                    info!("Found Instagram link in the text: {}", link);
                    // db.add("download", "", true).await.unwrap();
                    let url = extract_link(link).unwrap();
                    let video_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
                    let video_path = download(url, video_id.clone()).await;
                    info!("File with url {url} saved to {video_path}");
                    // db.add("download", "", false).await.unwrap();
                    // db.add("upload", "", true).await.unwrap();
                    match get_file_size(&video_path) {
                        Ok(size) => db.filesize(size as i64).await.unwrap(),
                        Err(e) => error!("Error getting file size: {}", e),
                    }
                    bot.send_video(msg.chat.id, InputFile::file(video_path))
                        .reply_to_message_id(msg.id)
                        .await?;
                    // db.add("upload", "", false).await.unwrap();
                }
                Some(false) => {
                    info!("Text does not contain an Instagram link.");
                }
                None => {
                    info!("No text found in the message.");
                }
            }
            respond(())
        }
    })
    .await;

    Ok(())
}

fn extract_link(message: &str) -> Option<&str> {
    if let Some(start) = message.find(INSTAGRAM_MAIN_URL) {
        let t = &message[start..];
        let end = t.find(' ').unwrap_or_else(|| t.len());
        let link = &t[..end];
        info!("Link extracted {link}");
        return Some(link);
    }
    None
}

async fn download(url: &str, video_id: String) -> String {
    let path = format!("/tmp/{video_id}");
    let output = Command::new("yt-dlp")
        .args(["-v", "-f", "mp4", "-o", &path, url])
        .current_dir("/tmp")
        .status();
    match output {
        Ok(_) => {
            return path;
        }
        Err(_) => {
            log::error!("Could not download a video for a given path: {url}");
            return String::new();
        }
    }
}
