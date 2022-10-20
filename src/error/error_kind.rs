#[derive(Debug)]
pub enum TwitchRecoverErrorKind {
    UrlParseStreamer,
    UrlParseVideoId,

    Regex,

    UserAgent,

    BadRequest(reqwest::Error),
    BadResponse,

    NoValidUrlFound,
}

impl From<reqwest::Error> for TwitchRecoverErrorKind {
    fn from(error: reqwest::Error) -> Self {
        Self::BadRequest(error)
    }
}
