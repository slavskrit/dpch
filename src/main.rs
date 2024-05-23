use log::info;
use rand::distributions::{Alphanumeric, DistString};
use std::io::Result;
use std::process::Command;
use std::sync::Arc;
use teloxide::payloads::SendVideoSetters;
use teloxide::prelude::*;
use teloxide::types::InputFile;

// mod db;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
}

static APPLICATION_NAME: &str = "dpch";
static INSTAGRAM_MAIN_URL: &str = "https://www.instagram.com/";

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();
    info!("Starting DPCH bot...");

    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    info!("Start Time: {start}");

    // let db = Database::new(&database_url).await.unwrap();
    // //
    //     db.set("foo", "bar").await.unwrap();
    //     let value = db.get("foo").await.unwrap();
    //
    let bot = Bot::from_env();
    // let agent_running = agent.start();
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        // let (add_tag, remove_tag) = (add_tag.clone(), remove_tag.clone());
        async move {
            match msg.text().map(|text| text.contains(INSTAGRAM_MAIN_URL)) {
                Some(true) => {
                    let link = msg.text().unwrap();
                    info!("Found Instagram link in the text: {}", link);
                    // add_tag("action".to_string(), "download_video".to_string()).unwrap();
                    let url = extract_link(link).unwrap();
                    let video_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
                    let video_path = download(url, video_id.clone()).await;
                    info!("File with url {url} saved to {video_path}");
                    // remove_tag("action".to_string(), "download_video".to_string()).unwrap();
                    // add_tag("action".to_string(), "upload_video".to_string()).unwrap();
                    bot.send_video(msg.chat.id, InputFile::file(video_path))
                        .reply_to_message_id(msg.id)
                        .await?;
                    // remove_tag("action".to_string(), "upload_video".to_string()).unwrap();
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
