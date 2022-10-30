# Twitch Recover

This crate allows you to recover a twitch vod.

``` toml
[dependencies]
anyhow = "0.1.0"
```
<br>

## Details

- Recover from a twitchtracker url

``` rust
let vod = VodRecover::from_twitchtracker("https://twitchtracker.com/streamer_name/streams/stream_id").await.unwrap();
let url = vod.get_url().await.unwrap();
println!("{}", url);
```

- Manual recover

``` rust
let date = "2022-10-29 13:06";
let timestamp = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M")
    .unwrap()
    .timestamp();
     
let vod = VodRecover::from_manual("streamer_name", "stream_id", timestamp);
let url = vod.get_url().await.unwrap();
println!("{}", url);
```

<br>

#### License

<sup>
Licensed under <a href="LICENSE">GPL-3.0
</sup>
