use std::sync::{Arc, Mutex};
use std::thread;

use crypto::digest::Digest;
use crypto::sha1::Sha1;

use chrono::naive::NaiveDateTime;

use regex::Regex;

use crate::utils::request;
use crate::{TwitchRecoverError, TwitchRecoverResult};

/// Represent a parsed twitchtracker url
struct ParsedTwitchTrackerUrl<'a>(&'a str, &'a str);

/// Struct representing a VOD
#[derive(Debug)]
pub struct VodRecover<'a> {
    streamer: &'a str,
    vod_id: &'a str,
    timestamp: i64,
}

/// twitchtracker related section
impl VodRecover<'_> {
    /// Create a VodRecover from a twitchtracker url
    pub async fn from_twitchtracker(url: &str) -> TwitchRecoverResult<VodRecover> {
        let ParsedTwitchTrackerUrl(streamer, vod_id) = Self::parse_twitchtracker_url(url)?;

        let url = format!("https://twitchtracker.com/{}/streams/{}", streamer, vod_id);

        let page = request(&url).await?;
        let timestamp = Self::parse_twitchtracker_timestamp(page, &url)?;

        Ok(VodRecover {
            streamer,
            vod_id,
            timestamp,
        })
    }

    /// Parse the twitchtracker url
    fn parse_twitchtracker_url(url: &str) -> TwitchRecoverResult<ParsedTwitchTrackerUrl> {
        let streamer = match url.split("com/").nth(1) {
            None => return Err(TwitchRecoverError::UrlParseStreamer(url.to_owned())),
            Some(res) => match res.split('/').next() {
                None => return Err(TwitchRecoverError::UrlParseStreamer(url.to_owned())),
                Some(streamer) => streamer,
            },
        };

        let vod_id = match url.split("streams/").nth(1) {
            None => return Err(TwitchRecoverError::UrlParseVodId(url.to_owned())),
            Some(vod_id) => vod_id,
        };

        Ok(ParsedTwitchTrackerUrl(streamer, vod_id))
    }

    /// Parse the timestamp from the twitchtracker page
    fn parse_twitchtracker_timestamp(page: String, url: &str) -> TwitchRecoverResult<i64> {
        let capture = match Regex::new(r"(stream-timestamp-dt.+>(?P<timestamp>.+)<)")
            .unwrap()
            .captures(&page)
        {
            Some(c) => c,
            None => return Err(TwitchRecoverError::PageParseTimestamp(url.to_owned())),
        };
        let date = match capture.name("timestamp") {
            Some(g) => g,
            None => return Err(TwitchRecoverError::PageParseTimestamp(url.to_owned())),
        };

        let date = page[date.start()..date.end()].to_string();
        let timestamp = NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .timestamp();

        Ok(timestamp)
    }
}

/// Manual related section
impl<'a> VodRecover<'a> {
    /// Manually recover a vod with a streamer name, vod is and a timestamp
    pub fn from_manual(streamer: &'a str, vod_id: &'a str, timestamp: i64) -> VodRecover {
        Self {
            streamer,
            vod_id,
            timestamp,
        }
    }
}

/// Urls related section
impl VodRecover<'_> {
    /// Get the vod url
    pub async fn get_url(&self) -> TwitchRecoverResult<String> {
        let urls = self.generate_all_urls();
        self.find_valid_url(urls).await
    }

    /// Generate all the possible urls
    fn generate_all_urls(&self) -> Vec<String> {
        let mut urls = vec![];
        for sec in 0..60 {
            let timestamp = self.timestamp + sec;
            let base_url = format!("{}_{}_{}", self.streamer, self.vod_id, timestamp);

            let mut hasher = Sha1::new();
            hasher.input_str(&base_url);
            let hashed_base_url = hasher.result_str();

            for domain in crate::DOMAINS {
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

    /// Find a valid url
    async fn find_valid_url(&self, urls: Vec<String>) -> TwitchRecoverResult<String> {
        let url_len = urls.len();
        let domains_len = crate::DOMAINS.len();
        let shared_urls = Arc::new(urls);

        let valid_url: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        for url_offset in 0..(url_len / domains_len) {
            let mut joinhandles = Vec::new();
            for domain_offset in 0..domains_len {
                let child_urls = shared_urls.clone();
                let child_valid_url = valid_url.clone();
                joinhandles.push(thread::spawn(move || -> TwitchRecoverResult {
                    let url = &child_urls[domain_offset * url_offset];
                    let response = reqwest::blocking::get(url)?;

                    if response.status() == 200 {
                        let mut guard = child_valid_url.lock().unwrap();
                        *guard = Some(url.to_owned());
                    }
                    Ok(())
                }));
            }

            for handle in joinhandles.into_iter() {
                handle.join().unwrap()?;

                let guard = valid_url.lock().unwrap();
                if let Some(ref v) = *guard {
                    return Ok(v.to_owned());
                }
            }
        }

        Err(TwitchRecoverError::VodNotFound)
    }
}
