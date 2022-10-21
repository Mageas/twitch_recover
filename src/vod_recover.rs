use regex::Regex;

use reqwest::header::USER_AGENT;

use chrono::naive::NaiveDateTime;

use crypto::digest::Digest;
use crypto::sha1::Sha1;

use std::sync::{Arc, Mutex};
use std::thread;

use crate::utils;
use crate::{TwitchRecoverError, TwitchRecoverErrorKind, TwitchRecoverResult};

#[derive(Debug)]
pub struct VodRecover<'a> {
    vod_id: &'a str,
    streamer: &'a str,
    timestamp: Option<i64>,
}

impl<'a> VodRecover<'a> {
    /// Generate the VodRecover struct without a timestamp
    pub fn from_url(url: &'a str) -> Result<Self, TwitchRecoverError> {
        let streamer = match url.split("com/").nth(1) {
            None => {
                return Err(TwitchRecoverError::new(
                    TwitchRecoverErrorKind::UrlParseStreamer,
                    "Unable to parse the streamer from the Url".to_string(),
                ))
            }
            Some(res) => match res.split('/').next() {
                None => {
                    return Err(TwitchRecoverError::new(
                        TwitchRecoverErrorKind::UrlParseStreamer,
                        "Unable to parse the streamer from the Url".to_string(),
                    ))
                }
                Some(streamer) => streamer,
            },
        };

        let vod_id = match url.split("streams/").nth(1) {
            None => {
                return Err(TwitchRecoverError::new(
                    TwitchRecoverErrorKind::UrlParseVodId,
                    "Unable to parse the vod id from the Url".to_string(),
                ))
            }
            Some(vod_id) => vod_id,
        };

        Ok(Self {
            vod_id,
            streamer,
            timestamp: None,
        })
    }

    /// Generate the timestamp
    pub async fn generate_timestamp_from_url(&mut self) -> TwitchRecoverResult {
        let header = utils::get_random_header()?;
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "https://twitchtracker.com/{}/streams/{}",
                self.streamer, self.vod_id
            ))
            .header(USER_AGENT, header)
            .send()
            .await?;

        if response.status() != 200 {
            return Err(TwitchRecoverError::new(
                TwitchRecoverErrorKind::BadResponse,
                "Unable to find the stream".to_string(),
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
                    "Unable to parse the timestamp of the stream".to_string(),
                ))
            }
        };
        let date = match capture.name("timestamp") {
            Some(g) => g,
            None => {
                return Err(TwitchRecoverError::new(
                    TwitchRecoverErrorKind::Regex,
                    "Unable to parse the timestamp of the stream".to_string(),
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

    /// Get the link of the vod
    pub async fn get_link(&self) -> Result<String, TwitchRecoverError> {
        let links = self.generate_links();
        let link = Self::find_valid_link(links).await?;
        Ok(link)
    }

    /// Generate all the possible urls for a given vod
    fn generate_links(&self) -> Vec<String> {
        let mut links = vec![];
        for sec in 0..60 {
            let timestamp = self.timestamp.unwrap() + sec;
            let base_link = format!("{}_{}_{}", self.streamer, self.vod_id, timestamp);

            let mut hasher = Sha1::new();
            hasher.input_str(&base_link);
            let hashed_base_link = hasher.result_str();

            for domain in crate::DOMAINS {
                links.push(format!(
                    "{}{}_{}/chunked/index-dvr.m3u8",
                    domain,
                    hashed_base_link[..20].to_owned(),
                    base_link
                ));
            }
        }
        links
    }

    /// Find a valid link in multi thread
    async fn find_valid_link(links: Vec<String>) -> Result<String, TwitchRecoverError> {
        let link_len = links.len();
        let domains_len = crate::DOMAINS.len();
        let shared_links = Arc::new(links);

        let valid_link: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        for link_offset in 0..(link_len / domains_len) {
            let mut joinhandles = Vec::new();
            for domain_offset in 0..domains_len {
                let child_links = shared_links.clone();
                let child_valid_link = valid_link.clone();
                joinhandles.push(thread::spawn(move || -> Result<(), TwitchRecoverError> {
                    let link = &child_links[domain_offset * link_offset];
                    let response = reqwest::blocking::get(link)?;

                    if response.status() == 200 {
                        let mut guard = child_valid_link.lock().unwrap();
                        *guard = Some(link.to_owned());
                    }
                    Ok(())
                }));
            }

            for handle in joinhandles.into_iter() {
                handle.join().unwrap()?;

                let guard = valid_link.lock().unwrap();
                if let Some(ref v) = *guard {
                    return Ok(v.to_owned());
                }
            }
        }

        Err(TwitchRecoverError::new(
            TwitchRecoverErrorKind::VodNotFound,
            "Unable to find the vod for the current link (if the vod is older than 60 days, it may be deleted)".to_string(),
        ))
    }

    /// Find a valid link in single thread
    async fn find_valid_link_sg(links: Vec<String>) -> Result<String, TwitchRecoverError> {
        let client = reqwest::Client::new();
        for link in links {
            let header = utils::get_random_header()?;
            let response = client.get(&link).header(USER_AGENT, header).send().await?;

            if response.status() == 200 {
                return Ok(link);
            }
        }
        Err(TwitchRecoverError::new(
            TwitchRecoverErrorKind::VodNotFound,
            "Unable to find the vod for the current link (if the vod is older than 60 days, it may be deleted)".to_string(),
        ))
    }
}
