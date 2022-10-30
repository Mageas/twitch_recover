use rand::seq::SliceRandom;
use rand::thread_rng;

use reqwest::header::USER_AGENT;

use crate::{TwitchRecoverError, TwitchRecoverResult};

/// Get a random header
pub fn get_random_header() -> Result<&'static str, TwitchRecoverError> {
    match crate::USER_AGENTS.choose(&mut thread_rng()) {
        Some(user_agent) => Ok(user_agent),
        None => Err(TwitchRecoverError::UserAgent),
    }
}

/// Request that returns a TwitchRecoverResult
pub async fn request(url: &str) -> TwitchRecoverResult<String> {
    let header = get_random_header()?;
    let client = reqwest::Client::new();
    let response = client.get(url).header(USER_AGENT, header).send().await?;

    if response.status() != 200 {
        return Err(TwitchRecoverError::BadResponseCode(
            response.status().to_string(),
            url.to_owned(),
        ));
    }

    Ok(response.text().await?)
}
