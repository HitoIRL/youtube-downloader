use custom_error::custom_error;
use serde::Deserialize;
use serde_json::{json, Value};
use regex::Regex;

const YOUTUBE_API_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";

#[derive(Clone, Copy, Deserialize, Debug)]
#[allow(non_camel_case_types)]
enum AudioQuality {
    NONE,
    AUDIO_QUALITY_LOW,
    AUDIO_QUALITY_MEDIUM,
    AUDIO_QUALITY_HIGH
}

struct VideoDetails {
    title: String,
    extension: String,
    download_url: String,
}

pub struct File {
    pub name: String,
    pub url: String,
}

custom_error!{pub Error
    InvalidVideo = "Invalid YouTube video URL provided",
}

fn get_video_extension(mime: &str) -> String {
    let mut extension = "m4a";

    if mime.contains("/webm") {
        extension = "webm";
    }

    extension.to_owned()
}

async fn get_video_details(id: &str) -> Result<VideoDetails, Error> {
    let api_url = format!("https://youtubei.googleapis.com/youtubei/v1/player?key={YOUTUBE_API_KEY}");
    let json = json!({
        "videoId": id,
        "context": {
            "client": {
                "clientName": "ANDROID",
                "clientVersion": "16.02"
            }
        }
    });

    let client = reqwest::Client::new();
    let res = client.post(api_url).json(&json).send().await.unwrap();
    let video_info: Value = res.json().await.unwrap();

    // check if video exists
    let is_playable = video_info["playabilityStatus"]["status"] == json!("OK");
    let has_details = video_info["videoDetails"] != json!(null);

    if !is_playable && !has_details {
        return Err(Error::InvalidVideo);
    }

    let video_formats = video_info["streamingData"]["formats"].as_array().unwrap();
    let video_adaptive_formats = video_info["streamingData"]["adaptiveFormats"].as_array().unwrap();

    let mut last_audio_quality = AudioQuality::NONE;
    let mut download_url = "";

    for format in video_formats.iter().chain(video_adaptive_formats) {
        let audio_quality: AudioQuality = serde_json::from_value(format["audioQuality"].clone()).unwrap_or(AudioQuality::NONE);

        if matches!(audio_quality, AudioQuality::NONE) {
            continue;
        }

        let is_better = audio_quality as u8 > last_audio_quality as u8;

        if format["mimeType"].to_string().contains("audio/") && is_better {
            download_url = format["url"].as_str().unwrap();
            last_audio_quality = audio_quality;
        }
    }

    let title = video_info["videoDetails"]["title"].as_str().unwrap_or("unknown").to_owned();
    let extension = get_video_extension(id);

    Ok(VideoDetails {
        title,
        extension,
        download_url: String::from(download_url),
    })
}

pub async fn get_download_url(url: &str) -> Result<File, Error> { // todo: video
    let id_regex = Regex::new(r"(?:v=|.be/)(.*$)").unwrap();
    let id = match id_regex.captures(url) {
        Some(captures) => captures.get(1).unwrap().as_str(),
        None => url // assume we have already provided id instead of url
    };

    let details = get_video_details(id).await;

    match details {
        Ok(details) => {
            Ok(File {
                name: format!("{}.{}", details.title, details.extension),
                url: details.download_url,
            })
        },
        Err(error) => Err(error)
    }
}
