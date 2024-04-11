# Pickup

[![CI](https://github.com/werkshy/pickup/actions/workflows/go-ci.yaml/badge.svg)](https://github.com/werkshy/pickup/actions/workflows/go-ci.yaml)

This is pickup (in go), a web frontend for [Music Player Daemon](http://mpd.wikia.com/wiki/Music_Player_Daemon_Wiki).

## Status

It works, more or less. You can add albums and tracks to the playlist, control
volume, skip tracks, start/stop playback. You can view the playlist. It runs on
the Raspberry Pi with room to run mpd as well. Because of the client-side
architecture it is very fast.

## Getting Started In Ubuntu

    # The React frontend requires the 'yarn' package manager -
    # install it like this:
    sudo apt install nodejs

    # This next command might require 'sudo' as well.
    npm install -g yarn

    # Requires go > v1.16, which is best installed via `snap` on Ubuntu.
    sudo snap install --classic go
    # Install yarn dependencies, build the frontend, and build the binary:
    make build

    # Run the binary
    ./pickup --help
    ./pickup

## Getting Started on macOS / brew

    # Requires go > 1.16 for the embed feature
    brew install yarn golang
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
