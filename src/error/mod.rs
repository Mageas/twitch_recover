use thiserror::Error;

#[derive(Error, Debug)]
pub enum TwitchRecoverError {
    #[error("streamer not found for {0}")]
    UrlParseStreamer(String),
    #[error("vod id not found for {0}")]
    UrlParseVodId(String),

    #[error("regex")]
    Regex,

    #[error("unable to select a user agent")]
    UserAgent,

    #[error("{0}")]
    BadRequest(#[from] reqwest::Error),

    #[error("stream not found")]
    StreamNotFound,

    #[error("vod not found")]
    VodNotFound,
}

pub type TwitchRecoverResult<T = ()> = Result<T, TwitchRecoverError>;
