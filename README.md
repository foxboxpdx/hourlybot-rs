# hourlybot-rs
A Mastodon bot that can periodically post images

## Current functionality
* Expects you to already have a bot token and put it into a TOML file matching the supplied template
* Command-line arguments via Clap:
    * Base dir for images
    * Posting frequency - 6 available presets!
    * Location of Mastodon config file
* Maintains a statefile to keep image reposts to a minimum
    * Statefile resets after everything's gotten posted once
    * Stored in /tmp, might make that an option later
* Outputs media location and toot ID after posting
* Wakes up once an hour to let you know it's still alive
* Calls tokio-scheduler a 'punk bitch' because it totally is one
* Really crappy deduper!
    * Uses a basic hashing method to compare files
    * Prints out a list of suspected duplicates
    * Can add a '.DUP' suffix to suspected duplicates

## ~~Planned~~ Aspiratonal functionality
* Register bot and get token automatically when supplied with `client_id` and `client_secret` (then write this out to the config toml for later use)
    * This is literally part of the example code for the Mastodon crate but pfft whatever
* Maybe multiple directories/schedules????

## Toml config file template
* Here's a template.  Fill in `base` and `token`.
```
base = ""
client_id = ""
client_secret = ""
redirect = ""
token = ""
```

hourlybot-rs v0.3.29 19/Jan/2024
