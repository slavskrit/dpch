use log::{info, log, warn};
use pyroscope::pyroscope::{PyroscopeAgentReady, PyroscopeAgentRunning};
use pyroscope::{PyroscopeAgent, Result};
use pyroscope_pprofrs::{pprof_backend, PprofConfig};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use teloxide::types::InputFile;
use teloxide::{dispatching::dialogue::InMemStorage, net::Download, prelude::*};
use tokio::fs;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
}

static APPLICATION_NAME: &str = "dpch.tags";
static INSTAGRAM_MAIN_URL: &str = "https://www.instagram.com/";

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let url = std::env::var("PYROSCOPE_URL").expect("PYROSCOPE_URL not set");
    info!("Starting DPCH bot...");
    info!("PYROSCOPE_URL: {url}");

    let agent = PyroscopeAgent::builder(url, APPLICATION_NAME.to_string())
        .backend(pprof_backend(PprofConfig::new().sample_rate(100)))
        .build()?;
    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    println!("Start Time: {}", start);

    let bot = Bot::from_env();

    // let (add_tag, remov_tag) = agent_running.tag_wrapper();

    // add_tag("series_a".to_string(), "Number 1".to_string());
    let agent_running = agent.start();
    let (add_tag, remove_tag) = agent_running.unwrap().tag_wrapper();

    let add_tag = Arc::new(add_tag);
    let remove_tag = Arc::new(remove_tag);

    remove_tag("series_a".to_string(), "a".to_string());
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let add_tag = add_tag.clone();
        let remove_tag = remove_tag.clone();
        async move {
            match msg.text().map(|text| text.contains(INSTAGRAM_MAIN_URL)) {
                Some(true) => {
                    let link = msg.text().unwrap();
                    info!("Found Instagram link in the text: {}", link);
                    bot.send_message(msg.chat.id, format!("Found Instagram link: {}", link))
                        .await?;
                    add_tag("action".to_string(), "long_task".to_string());
                    let url = extract_link(link).unwrap();
                    let video_id = extract_video_id(url).unwrap();
                    let video_path = download(url, video_id.clone()).await;
                    info!("File with id {video_id} saved to {video_path}");
                    bot.send_video(msg.chat.id, InputFile::file(video_path))
                        .await?;
                    remove_tag("action".to_string(), "lonk_task".to_string());
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
        return Some(link);
    }
    None
}

fn extract_video_id(url: &str) -> Option<String> {
    let instagram_url_start = "https://www.instagram.com/p/";
    if let Some(start) = url.find(instagram_url_start) {
        let t = &url[start + instagram_url_start.len()..];
        let end = t.find('/').unwrap_or_else(|| t.len());
        let video_id = &t[..end];
        return Some(video_id.to_string());
    }
    None
}

async fn download(url: &str, video_id: String) -> String {
    let output = Command::new("yt-dlp")
        .args(["-v", "-f", "mp4", "-o", &video_id, url])
        .current_dir("/tmp")
        .status();
    match output {
        Ok(r) => {
            return format!("/tmp/{video_id}");
        }
        Err(_) => {
            log::error!("Could not parse an audio by given path: {url}");
            return String::new();
        }
    }
    // match output {
    //     Ok(_output) => match fs::read_to_string(format!("{url}.txt")).await {
    //         Ok(result) => {
    //             log::info!("Result for {url}: {result}");
    //             return result;
    //         }
    //         Err(_) => {
    //             log::error!("Could not parse an audio by given path: {url}");
    //             return String::new();
    //         }
    //     },
    //     Err(_) => {
    //         log::error!("Could not parse an audio by given path: {url}");
    //         return String::new();
    //     }
    // }
}

async fn long_task() {
    print!("Long");
    thread::sleep(Duration::from_millis(4000));
}
