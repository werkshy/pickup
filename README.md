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
engine. going to use mpd for playback, since I already use mpd everywhere and
it just works. I'm writing it in Rust because I want it to be fast, run on
low-powered systems like Raspberry Pi, and I am enjoying learning more Rust.

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
