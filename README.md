# Pickup

[![CI](https://github.com/werkshy/pickup/actions/workflows/ci.yaml/badge.svg)](https://github.com/werkshy/pickup/actions/workflows/ci.yaml)

This is pickup (in go), a web frontend for [Music Player Daemon](http://mpd.wikia.com/wiki/Music_Player_Daemon_Wiki).

Pickup is my white whale... I've been playing with the idea since 2007. I have a
large music collection, and my navigation of choice is by album, with easy
search available. The "albums" interface in old Google Play Music (web) was pretty
close, but the uploaded never worked for me and now the product is dead.

## Status

It works, more or less. You can add albums and tracks to the playlist, control
volume, skip tracks, start/stop playback. You can view the playlist. It runs on
the Raspberry Pi with room to run mpd as well. Because of the client-side
architecture it is very fast.

In mid-2021, I have started rewriting the backend in Rust. The two motiviations for this are:
- This is the project I usually use to learn a new language or framework
- I want to get away from mpd and have a single executable that can do the web part, plus control the playlist and actually play music, and set us up for playback in the browser. This is basically a complete rewrite anyway.

## Getting Started In Ubuntu

    # Requires go > v1.16, which is best installed via `snap` on Ubuntu.
    sudo snap install --classic go

    go build
    ./pickup --help
    ./pickup

## Getting Started on macOS / brew

    # Requires go > 1.16 for the embed feature
    brew install golang
    make

## Getting Started Cross-Compiling for Raspberry Pi

Assuming you installed go as a snap, you should be able to cross-compile for
the Pi like this:

    GOOS=linux GOARCH=arm GOARM=5 go build

Now copy the pickup binary to your pi and run

    ./pickup

## Install As A Service

Copy init.d/pickup to /etc/init.d

Edit the script and set these vars to suit your setup. I'm lazy and just run as
my main user, definitely don't do this on the internet!

    SCRIPT=/home/oneill/pickup/pickup
    RUNAS=oneill

## Background

I've written functional prototypes of this in C++, Python, C++ again (I was
trying to run it on an NSLU2 embedded Linux machine with 32MB RAM) and now Go.
It started off as a standalone player, then grew an xmms2 backend, and now is
going to use mpd for playback, since I already use mpd everywhere and it just
works. I'm writing it now in Go because I want to learn Go and I want to have
this system. This is my first real project in go: any code review, criticism or
contributions would be much appreciated.

## Design Requirements

- Run on embedded hardware - any tiny board that runs Linux would be nice, Raspberry
  Pi would be fine.
- Display results quickly even when the music is stored on a slow-ish network
  drive (i.e. some caching of available music).
- Include more metadata than pure MPD, e.g. related artists, reviews etc. Can be
  loaded on the fly or stored locally.
- Show random albums to play. Shuffle-by-album.
- Assume /some/netsted/path/Artist/Album/Track.extension file format
- Must have: play now / add to main playlist
- Must have: play internet streams (e.g. DI Radio)
- Must have: responsive frontend, single-page-app feel.

## Design Approach

- The Go implementation is a simple backend serving JSON to a React frontend.
- The frontend loads the entire music collection up front. This takes less than
  a second and makes navigating around the collection extremely fast.

## Roadmap

- Backend: I'm working on a re-write of the backend in Rust, and dropping the
  mpd dependency.
- Frontend: Upgrade to Typescript, maybe start using create-react-app to make
  the build more vanilla.
- Frontend: Allow playback in the browser.

## Screenshot

![Screenshot](./screenshot.png)
