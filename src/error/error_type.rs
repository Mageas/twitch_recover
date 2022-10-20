use super::TwitchRecoverErrorKind;

#[derive(Debug)]
pub struct TwitchRecoverError {
    _kind: TwitchRecoverErrorKind,
    _cause: String,
}

#[allow(dead_code)]
impl TwitchRecoverError {
    pub fn new(_kind: TwitchRecoverErrorKind, _cause: String) -> Self {
        Self { _kind, _cause }
    }

    pub fn kind(&self) -> &TwitchRecoverErrorKind {
        &self._kind
    }
}

impl std::fmt::Display for TwitchRecoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self._cause)
    }
}

impl From<reqwest::Error> for TwitchRecoverError {
    fn from(err: reqwest::Error) -> Self {
        let cause = err.to_string();
        Self {
            _kind: TwitchRecoverErrorKind::from(err),
            _cause: cause,
        }
    }
}
