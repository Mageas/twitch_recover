# Twitch Recover

Inspired by [twitch_recover](https://github.com/pravindoesstuff/twitch_recover).

Twitch Recover is a free tool that allows you to recover direct m3u8 links (Working with sub only VODs).

## **How to use**

``` text
Recover the paid vod of your favorites twitch streamers

Usage: twitch_recover <COMMAND>

Commands:
  link  Recover a vod from TwitchTracker
  bulk  Recover vods from TwitchTracker
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

## **Build instructions**

Clone the repository:
```
git clone https://gitea.heartnerds.org/Mageas/twitch_recover
```

Move into the project directory:
```
cd twitch_recover
```

Build the project with cargo:
```
cargo build --release
```

The binary is located in `./target/release/twitch_recover`

