mod error_kind;
mod error_type;

pub use self::error_kind::TwitchRecoverErrorKind;
pub use self::error_type::TwitchRecoverError;

pub type TwitchRecoverResult<T = ()> = Result<T, TwitchRecoverError>;
