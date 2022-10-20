use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::{TwitchRecoverError, TwitchRecoverErrorKind};

/// Get a random header
pub fn get_random_header() -> Result<&'static str, TwitchRecoverError> {
    match crate::USER_AGENTS.choose(&mut thread_rng()) {
        Some(user_agent) => Ok(user_agent),
        None => Err(TwitchRecoverError::new(
            TwitchRecoverErrorKind::UserAgent,
            "Unable to randomly select a user agent".to_string(),
        )),
    }
}
