mod error;
use crate::error::{TwitchRecoverError, TwitchRecoverResult};

mod vod_recover;
use crate::vod_recover::VodRecover;

// mod vod_unmute;
// use crate::vod_unmute::VodUnmute;

mod constants;
use crate::constants::*;

mod args;
use args::Commands;
use clap::Parser;

mod utils;

use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::Cli::parse();

    match args.commands {
        Commands::Link(c) => {
            let mut vod = VodRecover::from_url(&c.link)?;

            println!("Searching for {}", &c.link);

            vod.generate_timestamp_from_url().await?;
            let link = vod.get_link().await?;

            println!("\nLink found:\n{}\n", &link);
        }
        Commands::Bulk(c) => {
            let mut links = vec![];
            for link in c.links {
                let mut vod = VodRecover::from_url(&link)?;

                println!("Searching for {}", &link);

                vod.generate_timestamp_from_url().await?;
                let link = vod.get_link().await?;

                links.push(link);
            }
            println!("\nLinks found:");
            for link in links {
                println!("{}", &link);
            }
        }
    }

    Ok(())
}
