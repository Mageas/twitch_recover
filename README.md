# Twitch Recover

This crate allows you to recover a twitch vod.

<br>

## Details

- Recover from a twitchtracker url

``` rust
let options = VodRecoverOptions {
    ..Default::default()
};

let url = "https://twitchtracker.com/streamer_id/streams/twitch_tracker_vod_id";
let vod = VodRecover::from_twitchtracker(url).await.unwrap();

let url = vod.get_url(&options).await.unwrap();

println!("{}", url);
```

- Manual recover

``` rust
let date = "2022-10-29 13:06";
let timestamp = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M")
    .unwrap()
    .timestamp();

let options = VodRecoverOptions {
    ..Default::default()
};

let vod = VodRecover::from_manual("streamer_name", "vod_id", timestamp);

let url = vod.get_url(&options).await.unwrap();

println!("{}", url);
```

<br>

#### License

<sup>
Licensed under <a href="LICENSE">GPL-3.0
</sup>
