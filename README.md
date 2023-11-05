# hourlybot-rs
A Mastodon bot that can periodically post images

## Current functionality
* Expects you to already have a bot token and put it into `mastodon-data.toml`
* ~~Takes no arguments and defaults to reading files from `{$PWD}/images/`~~
* ~~Executes posting function at the top of every hour~~
* Command-line arguments via Clap:
    * Specify the base directory where images are located
    * Specify posting frequency based on 6 available presets [top-of-hour, bottom-of-hour, once-daily, twice-daily, four-times-daily, six-times-daily]
* Maintains a statefile to keep duplicates to a minimum
* Outputs some not particularly useful status info after each post

## ~~Planned~~ Aspiratonal functionality
* Register bot and get token automatically when supplied with `client_id` and `client_secret` (then write this out to the config toml for later use)
* Maybe multiple directories/schedules????
* More useful status info after posting

## Toml file
* Here's a template.  Fill in `base` and `token`.
```
base = ""
client_id = ""
client_secret = ""
redirect = ""
token = ""
```

hourlybot-rs v0.2.11 4/Nov/2023
