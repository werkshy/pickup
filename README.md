# Pickup

[![CI](https://github.com/werkshy/pickup/actions/workflows/rust_ci.yaml/badge.svg)](https://github.com/werkshy/pickup/actions/workflows/rust_ci.yaml)

This is pickup, a web-based music player.

Pickup is my white whale... I've been playing with the idea since 2007. I have
a large music collection, and my navigation of choice is by album, with easy
search available. The "albums" interface in old Google Play Music (web) was
pretty close, but the uploader never worked for me and now the product is dead.
Pickup has been through several iterations, now it's a Rust API backend that
can play jukebox-style on the server, and I'm working towards browser playback
too.

## Status

You can add albums and tracks to the playlist, control playback and volume (for
the server-side jukebox mode).

## Background

I've written functional prototypes of this in C++, Python, C++ again (I was
trying to run it on an NSLU2 embedded Linux machine with 32MB RAM), Go/React,
and now Rust with htmx. It started off as a standalone player, then grew an
xmms2 backend, then became an mpd client, and now has its own built-in playback
engine. I'm writing it in Rust because I want it to be fast, run on low-powered
systems like Raspberry Pi, and I am enjoying learning more Rust.

## Design Requirements

- Run on embedded hardware - any tiny board that runs Linux would be nice, Raspberry
  Pi would be fine.
- Display results quickly even when the music is stored on a slow-ish network
  drive (i.e. some caching of available music).
- Include metadata (related artists, photos, album covers, etc.
- Show random albums to play. Shuffle-by-album.
- Assumes /some/nested/path/Artist/Album/Track.extension file format
- Must have: play now / add to main playlist
- Must have: play internet streams (e.g. DI Radio)
- Must have: responsive frontend, single-page-app feel.

## Design Approach

#### Rust Version:

- Built-in music player in the server and file indexing.
- I intend to add in-browser music playback as well.
- Server-rendered HTML frontend that uses htmx to keep the player persistent as
  you navigate.

## Screenshot

![Screenshot (Go/React version)](./screenshot.png)

## Development

Set up a rust toolchain e.g. `rustup toolchain install`

Set up playwright:
```
cd e2e
npm install
npx playwright install chromium
```

### Run tests

```
cargo test
cd e2e && npm test
```

## Docker

Images are published to GitHub Container Registry: `ghcr.io/werkshy/pickup`.

A `docker-compose.yml` is included. By default it mounts `./music` read-only at `/music`
and `./cache` at `/cache`, and serves on port 9090.

```bash
mkdir -p cache
docker compose up -d
```

**Using the Published Image**

Example docker-compose snippet:

```
services:
  pickup:
    image: ghcr.io/werkshy/pickup
    container_name: pickup
    restart: unless-stopped
    user: 1000:100
    volumes:
      - "/data/music:/music:ro"
      - "./pickup/cache:/cache"
    ports:
      - 9090:9090
```

**Environment variables**

| Variable       | Default               | Description                                            |
|----------------|-----------------------|--------------------------------------------------------|
| `MUSIC_DIR`    | `/music` (container)  | Root of the music collection; overridden by bind mount |
| `CACHE_DIR`    | `/cache` (container)  | Directory for the music file cache; overridden by bind mount |
| `PORT`         | `9090`                | HTTP server port                                       |
| `RUST_LOG`     | (unset)               | Filter; defaults to `info,symphonia_bundle_mp3=warn`  |

**Permissions**

The compose service runs as UID/GID 1000 by default. The `./cache` bind mount
must be writable by that user. Override with `PUID`/`PGID` e.g.
`user: "${PUID:-1000}:${PGID:-1000}"`.

**Audio (jukebox mode)**

Browser playback (the default mode) works without any audio device. To use
server‑side jukebox mode, map the host sound device into the container:

```
services:
  pickup:
    # devices:
    #   - /dev/snd:/dev/snd
    # group_add:
    #   - audio
```

**Refreshing the cache after library changes**

```bash
docker compose run --rm pickup refresh
```

**No authentication**

The server has no built‑in authentication and should not be exposed to the
public internet. Put it behind a reverse proxy or restrict port mapping to
`127.0.0.1`.

## Background

...
