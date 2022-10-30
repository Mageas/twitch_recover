use thiserror::Error;

#[derive(Error, Debug)]
pub enum TwitchRecoverError {
    #[error("Unable to parse the streamer ({0})")]
    UrlParseStreamer(String),
    #[error("Unable to parse the vod id ({0})")]
    UrlParseVodId(String),

    #[error("Unable to parse the timestamp ({0})")]
    PageParseTimestamp(String),

    #[error("Unable to select a user agent for the request")]
    UserAgent,

    #[error("{0}")]
    BadRequest(#[from] reqwest::Error),

    #[error("{0} from ({1})")]
    BadResponseCode(String, String),

    #[error("Vod not found")]
    VodNotFound,
}

pub type TwitchRecoverResult<T = ()> = Result<T, TwitchRecoverError>;
