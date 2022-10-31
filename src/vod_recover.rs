use std::sync::{Arc, Mutex};
use std::thread;

use crypto::digest::Digest;
use crypto::sha1::Sha1;

use chrono::naive::NaiveDateTime;

use regex::Regex;

use crate::utils::request;
use crate::{TwitchRecoverError, TwitchRecoverResult};

/// # Represent a parsed twitchtracker url
///
/// The first field is the *streamer* and the second is the *vod_id*
struct ParsedTwitchTrackerUrl<'a>(&'a str, &'a str);

/// # Struct representing a VOD
///
/// It is composed of a *streamer*, a *video_id*, and a *timestamp*
#[derive(Debug)]
pub struct VodRecover<'a> {
    streamer: &'a str,
    vod_id: &'a str,
    timestamp: i64,
}

/// twitchtracker related section
impl VodRecover<'_> {
    /// # Create a VodRecover from a twitchtracker url
    ///
    /// ## Examples
    ///
    /// ```
    /// # use twitch_recover::VodRecover;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let url = "https://twitchtracker.com/streamer_name/streams/stream_id";
    /// let vod = VodRecover::from_twitchtracker(url).await;
    /// # assert!(vod.is_err());
    /// # }
    /// ```
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
    /// # Manually recover a vod
    ///
    /// Recover with a streamer name, vod is and a timestamp
    ///
    /// ## Examples
    ///
    /// ```
    /// # use twitch_recover::VodRecover;
    /// use chrono::naive::NaiveDateTime;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let date = "2022-10-29 13:06";
    /// let timestamp = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M")
    ///     .unwrap()
    ///     .timestamp();
    ///     
    /// let vod = VodRecover::from_manual("streamer_name", "stream_id", timestamp);
    /// # }
    /// ```
    pub fn from_manual(streamer: &'a str, vod_id: &'a str, timestamp: i64) -> VodRecover<'a> {
        Self {
            streamer,
            vod_id,
            timestamp,
        }
    }
}

/// # Options for a vod
///
/// ## Examples
///
/// ```
/// # use twitch_recover::VodRecoverOptions;
/// let options = VodRecoverOptions {
///     ..Default::default()
/// };
/// # assert_eq!(options.chunck, 17);
/// ```
///
/// ```
/// # use twitch_recover::VodRecoverOptions;
/// let options = VodRecoverOptions::new(100);
/// # assert_eq!(options.chunck, 100);
/// ```
#[derive(Debug)]
pub struct VodRecoverOptions {
    /// How many concurrent requests
    pub chunck: usize,
}

impl VodRecoverOptions {
    /// New VodRecoverOptions
    pub fn new(chunck: usize) -> Self {
        Self { chunck }
    }
}

impl Default for VodRecoverOptions {
    /// Default value for VodRecoverOptions
    fn default() -> Self {
        Self {
            chunck: crate::DOMAINS.len(),
        }
    }
}

/// Urls related section
impl VodRecover<'_> {
    /// Get the vod url
    pub async fn get_url(&self, options: &VodRecoverOptions) -> TwitchRecoverResult<String> {
        let urls = self.generate_all_urls();
        let urls = Self::split_urls_in_chunks(urls, options.chunck);
        Self::find_valid_url(urls).await
    }

    /// Split urls into chunks
    fn split_urls_in_chunks(urls: Vec<String>, chunck: usize) -> Vec<Vec<String>> {
        let mut output = vec![];
        let mut output_chunck = vec![];
        for url in urls {
            output_chunck.push(url);
            if output_chunck.len() == chunck {
                output.push(output_chunck);
                output_chunck = vec![];
            }
        }
        if !output_chunck.is_empty() {
            output.push(output_chunck);
        }
        output
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
    async fn find_valid_url(urls: Vec<Vec<String>>) -> TwitchRecoverResult<String> {
        let valid_url: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        for chunck in urls {
            let mut joinhandles = Vec::new();
            let shared_chuck = Arc::new(chunck);

            for offset in 0..shared_chuck.len() {
                let child_chunck = shared_chuck.clone();
                let child_valid_url = valid_url.clone();
                joinhandles.push(thread::spawn(move || -> TwitchRecoverResult {
                    let url = &child_chunck[offset];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_twitchtracker_url() {
        let url = "https://twitchtracker.com/streamer_name/streams/10000000";
        let ParsedTwitchTrackerUrl(streamer, vod_id) =
            VodRecover::parse_twitchtracker_url(url).unwrap();

        assert_eq!(streamer, "streamer_name");
        assert_eq!(vod_id, "10000000");
    }

    #[test]
    #[should_panic]
    fn fail_streamer_name_parse_twitchtracker_url() {
        let url = "https://twitchtracker.cstreamer_name/streams/10000000";
        VodRecover::parse_twitchtracker_url(url).unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_vod_id_parse_twitchtracker_url() {
        let url = "https://twitchtracker.cstreamer_name/stre10000000";
        VodRecover::parse_twitchtracker_url(url).unwrap();
    }

    #[test]
    fn test_parse_twitchtracker_timestamp() {
        let page =
            r#"...<div class=\"stream-timestamp-dt to-dowdatetime\">2022-10-30 14:57:02</div>..."#
                .to_string();
        let timestamp = VodRecover::parse_twitchtracker_timestamp(page, "").unwrap();

        assert_eq!(timestamp, 1667141822);
    }

    #[test]
    fn test_split_urls_in_chunks() {
        let urls = vec!["".to_string(); 7];
        let chunks = VodRecover::split_urls_in_chunks(urls, 3);

        assert_eq!(chunks[0].len(), 3);
        assert_eq!(chunks[1].len(), 3);
        assert_eq!(chunks[2].len(), 1);
    }

    #[test]
    fn test_generate_all_urls() {
        let vod = VodRecover {
            streamer: "streamer_name",
            vod_id: "vod_id",
            timestamp: 100000,
        };
        let urls = vod.generate_all_urls();

        assert_eq!(urls.first().unwrap(), "https://vod-secure.twitch.tv/6fc3cda1d80cf7bf6b72_streamer_name_vod_id_100000/chunked/index-dvr.m3u8");
        assert_eq!(urls.last().unwrap(), "https://d3aqoihi2n8ty8.cloudfront.net/8a03f89d58f01a4b5539_streamer_name_vod_id_100059/chunked/index-dvr.m3u8");
    }
}
