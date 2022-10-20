#[derive(Debug)]
pub enum TwitchRecoverErrorKind {
    UrlParseStreamer,
    UrlParseVodId,

    Regex,

    UserAgent,

    BadRequest(reqwest::Error),
    BadResponse,

    VodNotFound,
}

impl From<reqwest::Error> for TwitchRecoverErrorKind {
    fn from(error: reqwest::Error) -> Self {
        Self::BadRequest(error)
    }
}
