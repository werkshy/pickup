Pickup
======

This is pickup (in go), a web frontend for [Music Player Daemon](http://mpd.wikia.com/wiki/Music_Player_Daemon_Wiki) by Andy O'Neill
<andy@potatoriot.com>.

Pickup is my white whale... I've been playing with the idea since 2007. I have a
large music collection, and my navigation of choice is by album, with easy
search available. The "albums" interface in Google Play Music (web) is pretty
close, but the google play uploader sucks and misses half of my songs.


Status
------

It works, just about. You can add albums and tracks to the playlist, control
volume, skip tracks, start/stop playback. You can view the playlist. It runs on the Raspberry Pi with room to run mpd as well.

Background
-----------

I've written functional prototypes of this in C++, Python, C++ again (I was
trying to run it on an NSLU2 embedded Linux machine with 32MB RAM) and now Go.
It started off as a standalone player, then grew an xmms2 backend, and now is
going to use mpd for playback, since I already use mpd everywhere and it just
works. I'm writing it now in Go because I want to learn Go and I want to have
this system.

Design Requirements
--------------------

- Run on embedded hardware. NSLU2 would be nice, Raspberry Pi (256MB) would be
  fine.
- Display results quickly even when the music is stored on a slow-ish network
  drive (i.e. some caching of available music).
- Include more metadata than pure MPD, e.g. related artists, reviews etc. Can be
  loaded on the fly or stored locally.
- Show random albums to play. Shuffle-by-album.
- Assume /some/netsted/path/Artist/Album/Track.extension file format
- Must have: play now / add to main playlist
- Must have: play internet streams (e.g. DI Radio)
- Must have: responsive frontend, single-page-app feel.

Design Philosophy
-----------------

The Go implementation is a simple backend serving JSON to a javascript frontend,
(single-page app in Backbone.js.)

Roadmap
-------

See https://trello.com/board/pickup/515a58746cbd4fd847001505


