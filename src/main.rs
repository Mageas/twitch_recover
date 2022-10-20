mod error;
use crate::error::{TwitchRecoverError, TwitchRecoverErrorKind, TwitchRecoverResult};

mod vod_recover;
use crate::vod_recover::VodRecover;

mod constants;
use crate::constants::*;

mod utils;

#[tokio::main]
async fn main() -> TwitchRecoverResult {
    let mut vod = VodRecover::from_url("https://twitchtracker.com/skyyart/streams/39967004696")?;
    let mut vod = VodRecover::from_url("https://twitchtracker.com/zerator/streams/46124092796")?;

    vod.generate_timestamp_from_url().await?;
    let link = vod.get_link().await?;

    dbg!(&vod);
    dbg!(&link);

    Ok(())
}
