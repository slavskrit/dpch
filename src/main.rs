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
use tokio::fs::remove_file;

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
static X_MAIN_URL: &str = "https://x.com/";

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();
    // let postgres_user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    // let postgres_password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    // let postgres_db = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
    // let database_url = format!("postgres://{postgres_user}:{postgres_password}@postgres:5432/");

    info!("Starting DPCH bot...");
    // info!("DB at: {database_url}");

    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    info!("Start Time: {start}");

    // let db = Arc::new(Database::new(&database_url, &postgres_db).await.unwrap());
    // db.create_table().await.unwrap();

    let bot = Bot::from_env();
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        // let db = db.clone();
        async move {
            if let Some(text) = msg.text() {
                if text.contains(X_MAIN_URL) {
                    process_message(&bot, &msg, text, X_MAIN_URL).await;
                } else if text.contains(INSTAGRAM_MAIN_URL) {
                    process_message(&bot, &msg, text, INSTAGRAM_MAIN_URL).await;
                } else {
                    info!("Text does not contain an Instagram or X link.");
                }
            } else {
                info!("No text found in the message.");
            }
            respond(())
        }
    })
    .await;

    Ok(())
}

async fn process_message(bot: &Bot, msg: &Message, text: &str, main_url: &str) {
    info!("Found {} link in the text: {}", main_url, text);
    if let Some(url) = extract_link(text, main_url) {
        let video_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        let video_path = download(url, video_id.clone()).await;
        info!("File with url {} saved to {}", url, video_path);

        match get_file_size(&video_path) {
            Ok(size) => info!("Size of the file: {}", size),
            Err(e) => error!("Error getting file size: {}", e),
        }

        if let Err(e) = bot
            .send_video(msg.chat.id, InputFile::file(video_path.clone()))
            .reply_to_message_id(msg.id)
            .await
        {
            error!("Error sending video: {}", e);
        }

        if let Err(e) = remove_file(video_path).await {
            error!("Error removing file: {}", e);
        }
    }
}

fn extract_link<'a>(message: &'a str, main_url: &'a str) -> Option<&'a str> {
    info!("Extracting link {} for the {}", message, main_url);
    if let Some(start) = message.find(main_url) {
        let t = &message[start..];
        let end = t.find(' ').unwrap_or_else(|| t.len());
        let link = &t[..end];
        info!("Link extracted {}", link);
        return Some(link);
    }
    None
}

async fn download(url: &str, video_id: String) -> String {
    let path = format!("/tmp/{}", video_id);
    let output = Command::new("yt-dlp")
        .args(["-v", "-f", "mp4", "-o", &path, url])
        .current_dir("/tmp")
        .status();

    match output {
        Ok(status) if status.success() => path,
        Ok(status) => {
            error!("yt-dlp exited with status: {}", status);
            String::new()
        }
        Err(e) => {
            error!("Error running yt-dlp: {}", e);
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_link() {
        let message = "Check this link https://www.instagram.com/p/xyz123/ for more details.";
        let main_url = "https://www.instagram.com/";
        let result = extract_link(message, main_url);
        assert_eq!(result, Some("https://www.instagram.com/p/xyz123/"));
    }

    #[test]
    fn test_extract_link_with_multiple_links() {
        let message =
            "Here are two links: https://www.instagram.com/p/xyz123/ and https://x.com/abc456/";
        let main_url = "https://www.instagram.com/";
        let result = extract_link(message, main_url);
        assert_eq!(result, Some("https://www.instagram.com/p/xyz123/"));
    }

    #[test]
    fn test_extract_link_no_link() {
        let message = "No links here.";
        let main_url = "https://www.instagram.com/";
        let result = extract_link(message, main_url);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_file_size() {
        use std::fs::File;
        use std::io::Write;

        let path = "/tmp/test_file_size.txt";
        let mut file = File::create(path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let size = get_file_size(path).unwrap();
        assert_eq!(size, 14); // "Hello, world!\n" is 14 bytes

        std::fs::remove_file(path).unwrap();
    }
}
