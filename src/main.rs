mod error;
use error::TwitchRecoverResult;

use crate::error::{TwitchRecoverError, TwitchRecoverErrorKind};

use rand::seq::SliceRandom;
use rand::thread_rng;

use chrono::naive::NaiveDateTime;

use crypto::digest::Digest;
use crypto::sha1::Sha1;

use reqwest;
use reqwest::header::USER_AGENT;
use tokio;

use regex::Regex;

const DOMAINS: &'static [&'static str] = &[
    "https://vod-secure.twitch.tv/",
    "https://vod-metro.twitch.tv/",
    "https://vod-pop-secure.twitch.tv/",
    "https://d2e2de1etea730.cloudfront.net/",
    "https://dqrpb9wgowsf5.cloudfront.net/",
    "https://ds0h3roq6wcgc.cloudfront.net/",
    "https://d2nvs31859zcd8.cloudfront.net/",
    "https://d2aba1wr3818hz.cloudfront.net/",
    "https://d3c27h4odz752x.cloudfront.net/",
    "https://dgeft87wbj63p.cloudfront.net/",
    "https://d1m7jfoe9zdc1j.cloudfront.net/",
    "https://d3vd9lfkzbru3h.cloudfront.net/",
    "https://d2vjef5jvl6bfs.cloudfront.net/",
    "https://d1ymi26ma8va5x.cloudfront.net/",
    "https://d1mhjrowxxagfy.cloudfront.net/",
    "https://ddacn6pr5v0tl.cloudfront.net/",
    "https://d3aqoihi2n8ty8.cloudfront.net/",
];

const USER_AGENTS: &'static [&'static str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 12_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:103.0) Gecko/20100101 Firefox/103.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 12.5; rv:103.0) Gecko/20100101 Firefox/103.0",
    "Mozilla/5.0 (X11; Linux i686; rv:103.0) Gecko/20100101 Firefox/103.0",
    "Mozilla/5.0 (Linux x86_64; rv:103.0) Gecko/20100101 Firefox/103.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux i686; rv:103.0) Gecko/20100101 Firefox/103.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:103.0) Gecko/20100101 Firefox/103.0",
    "Mozilla/5.0 (X11; Fedora; Linux x86_64; rv:103.0) Gecko/20100101 Firefox/103.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:102.0) Gecko/20100101 Firefox/102.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 12.5; rv:102.0) Gecko/20100101 Firefox/102.0",
    "Mozilla/5.0 (X11; Linux i686; rv:102.0) Gecko/20100101 Firefox/102.0",
    "Mozilla/5.0 (Linux x86_64; rv:102.0) Gecko/20100101 Firefox/102.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux i686; rv:102.0) Gecko/20100101 Firefox/102.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:102.0) Gecko/20100101 Firefox/102.0",
    "Mozilla/5.0 (X11; Fedora; Linux x86_64; rv:102.0) Gecko/20100101 Firefox/102.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 12_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36 Edg/103.0.1264.77",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 12_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36 Edg/103.0.1264.77",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.99 Safari/537.36"
];

#[derive(Debug)]
struct VodInfos<'a> {
    id: &'a str,
    streamer: &'a str,
    timestamp: Option<i64>,
}

impl<'a> VodInfos<'a> {
    pub fn new(id: &'a str, streamer: &'a str) -> Self {
        Self {
            id,
            streamer,
            timestamp: None,
        }
    }

    pub async fn generate_timestamp(&mut self) -> TwitchRecoverResult {
        let header = get_random_header()?;
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "https://twitchtracker.com/{}/streams/{}",
                self.streamer, self.id
            ))
            .header(USER_AGENT, header)
            .send()
            .await?;

        if response.status() != 200 {
            return Err(TwitchRecoverError::new(
                TwitchRecoverErrorKind::BadResponse,
                "Bad response while fetching twitchtracker".to_string(),
            ));
        }

        let page = response.text().await?;

        let capture = match Regex::new(r"(stream-timestamp-dt.+>(?P<timestamp>.+)<)")
            .unwrap()
            .captures(&page)
        {
            Some(c) => c,
            None => {
                return Err(TwitchRecoverError::new(
                    TwitchRecoverErrorKind::Regex,
                    "Unable to parse the timestamp".to_string(),
                ))
            }
        };
        let date = match capture.name("timestamp") {
            Some(g) => g,
            None => {
                return Err(TwitchRecoverError::new(
                    TwitchRecoverErrorKind::Regex,
                    "Unable to parse the timestamp".to_string(),
                ))
            }
        };

        let date = page[date.start()..date.end()].to_string();
        let timestamp = NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .timestamp();

        self.timestamp = Some(timestamp);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> TwitchRecoverResult {
    let mut vod_infos = vod_from_url("https://twitchtracker.com/skyyart/streams/39967004696")?;
    println!("{:?}", &vod_infos);

    vod_infos.generate_timestamp().await?;

    println!("{:?}", &vod_infos);
    let urls = get_all_url(&vod_infos);

    let url = get_valid_url(&urls).await?;

    dbg!(&url);

    Ok(())
}

async fn get_valid_url(urls: &Vec<String>) -> Result<&str, TwitchRecoverError> {
    let client = reqwest::Client::new();
    for url in urls {
        let header = get_random_header()?;
        let response = client.get(url).header(USER_AGENT, header).send().await?;

        if response.status() == 200 {
            return Ok(url);
        }
    }
    Err(TwitchRecoverError::new(
        TwitchRecoverErrorKind::NoValidUrlFound,
        "Unable to find a valid url".to_string(),
    ))
}

fn get_all_url(vod_infos: &VodInfos) -> Vec<String> {
    let mut urls = vec![];
    for sec in 0..60 {
        let timestamp = vod_infos.timestamp.unwrap() + sec;
        let base_url = format!("{}_{}_{}", vod_infos.streamer, vod_infos.id, timestamp);

        let mut hasher = Sha1::new();
        hasher.input_str(&base_url);
        let hashed_base_url = hasher.result_str();

        for domain in DOMAINS {
            urls.push(format!(
                "{}{}_{}/chunked/index-dvr.m3u8",
                domain,
                hashed_base_url[..20].to_owned(),
                base_url
            ));
        }
    }
    urls
}

fn vod_from_url(url: &str) -> Result<VodInfos, TwitchRecoverError> {
    let streamer = match url.split("com/").nth(1) {
        None => {
            return Err(TwitchRecoverError::new(
                TwitchRecoverErrorKind::UrlParseStreamer,
                "Unable to parse streamer from Url".to_string(),
            ))
        }
        Some(res) => match res.split('/').next() {
            None => {
                return Err(TwitchRecoverError::new(
                    TwitchRecoverErrorKind::UrlParseStreamer,
                    "Unable to parse streamer from Url".to_string(),
                ))
            }
            Some(streamer) => streamer,
        },
    };

    let vod_id = match url.split("streams/").nth(1) {
        None => {
            return Err(TwitchRecoverError::new(
                TwitchRecoverErrorKind::UrlParseVideoId,
                "Unable to parse video_id from Url".to_string(),
            ))
        }
        Some(vod_id) => vod_id,
    };

    Ok(VodInfos::new(vod_id, streamer))
}

fn get_random_header() -> Result<&'static str, TwitchRecoverError> {
    match USER_AGENTS.choose(&mut thread_rng()) {
        Some(user_agent) => Ok(user_agent),
        None => Err(TwitchRecoverError::new(
            TwitchRecoverErrorKind::UserAgent,
            "Unable to randomly select a user agent".to_string(),
        )),
    }
}
