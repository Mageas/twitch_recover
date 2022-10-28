use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::TwitchRecoverError;

/// Get a random header
pub fn get_random_header() -> Result<&'static str, TwitchRecoverError> {
    match crate::USER_AGENTS.choose(&mut thread_rng()) {
        Some(user_agent) => Ok(user_agent),
        None => Err(TwitchRecoverError::UserAgent),
    }
}
