use clap::{Args, Parser, Subcommand};

/// Recover the paid vod of your favorites twitch streamers
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Recover a vod from TwitchTracker
    Link(Link),
    /// Recover vods from TwitchTracker
    Bulk(Bulk),
}

#[derive(Args, Debug)]
pub struct Link {
    pub link: String,
}

#[derive(Args, Debug)]
pub struct Bulk {
    pub links: Vec<String>,
}
