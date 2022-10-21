use reqwest::header::USER_AGENT;

use crate::error::{TwitchRecoverError, TwitchRecoverErrorKind};
use crate::utils;

use futures_util::StreamExt;

#[derive(Debug)]
pub struct VodUnmute<'a> {
    link: &'a str,
}

#[derive(Debug)]
pub enum VodUnmuteResult {
    VodAlreadyUnmuted,
    Vod(String),
}

impl<'a> VodUnmute<'a> {
    pub fn new(link: &'a str) -> Self {
        Self { link }
    }

    pub async fn unmute(&self) -> Result<VodUnmuteResult, TwitchRecoverError> {
        if !self.is_muted().await? {
            return Ok(VodUnmuteResult::VodAlreadyUnmuted);
        }

        let url = self.link.replace("index-dvr.m3u8", "");

        let client = reqwest::Client::new();
        let response = client.get(self.link).send().await?;
        let response_size = response.content_length();

        let mut stream = response.bytes_stream();
        let mut buffer = vec![];

        let mut counter = 0;

        while let Some(stream) = stream.next().await {
            let chunk = match stream {
                Ok(chunck) => chunck,
                Err(_) => todo!(),
            };

            for byte in chunk {
                if byte == b'\n' {
                    let line = std::str::from_utf8(buffer.as_slice()).unwrap();

                    if line.contains("-unmuted") && !line.starts_with('#') {
                        let r = format!("{}{}-muted.ts", url, counter);
                        println!("{}", &r);
                        counter += 1;
                    } else if !line.contains("-unmuted") && !line.starts_with('#') {
                        let r = format!("{}{}.ts", url, counter);
                        println!("{}", &r);
                        counter += 1;
                    } else {
                        println!("{}", &line);
                    }
                    buffer.clear();
                } else {
                    buffer.push(byte);
                }
            }
        }

        dbg!(&response_size, &url);

        return Err(TwitchRecoverError::new(
            TwitchRecoverErrorKind::VodNotFound,
            "TODO".to_string(),
        ));
    }

    async fn is_muted(&self) -> Result<bool, TwitchRecoverError> {
        let client = reqwest::Client::new();
        let header = utils::get_random_header()?;
        let response = client
            .get(self.link)
            .header(USER_AGENT, header)
            .send()
            .await?;

        if response.status() != 200 {
            return Err(TwitchRecoverError::new(
                TwitchRecoverErrorKind::VodNotFound,
                "TODO".to_string(),
            ));
        }

        Ok(response.text().await?.contains("unmuted"))
    }
}
