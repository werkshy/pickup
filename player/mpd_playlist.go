package player

// Implement playlist functions for MPD backand

import (
	"errors"
	"log"

	"github.com/werkshy/pickup/model"
)

/**
 * Implement playlist interface via mpd
 */
func (player MpdPlayer) List() (results []PlaylistTrack, err error) {
	info, err := player.conn.PlaylistInfo(-1, -1)
	if err != nil {
		log.Printf("Failed to get playlist info from mpd\n")
		return nil, err
	}
	for _, entry := range info {
		//log.Printf("%q\n", entry)
		track := PlaylistTrack{
			entry["Pos"],
			entry["Title"],
			entry["Artist"],
			entry["Album"],
			entry["Path"],
			"mpd"}
		track.cleanUp(entry["file"])
		results = append(results, track)
	}
	return results, nil
}

func (player MpdPlayer) AddAlbum(album *model.Album) (err error) {
	log.Printf("Adding album %s - %s (%s)\n", album.Artist, album.Name,
		album.Path)
	return player.conn.Add(album.Path)
}

func (player MpdPlayer) AddTrack(track *model.Track) (err error) {
	log.Printf("Adding track %v\n", track)
	return player.conn.Add(track.Path)
}

func (player MpdPlayer) AddTracks(tracks []*model.Track) (err error) {
	for _, track := range tracks {
		log.Printf("Adding track %s\n", track)
		err := player.AddTrack(track)
		if err != nil {
			return err
		}
	}
	return nil
}

func (player MpdPlayer) Clear() (err error) {
	log.Println("Clearing playlist")
	player.conn.Clear()
	return nil
}

func (t *PlaylistTrack) cleanUp(file string) {
	if t.Name != "" {
		return
	}
	_, artist, album, track, err := model.PathToParts(file)
	if err != nil {
		track = "unknown"
		artist = "unknown"
		album = "unknown"
	}
	if t.Name == "" {
		t.Name = track
	}
	if t.Artist == "" {
		t.Artist = artist
	}
	if t.Album == "" {
		t.Album = album
	}
}

func (player *MpdPlayer) doPlaylistCommand(cmd *PlaylistCommand, music model.Collection) (err error) {
	switch cmd.Command {
	case "add":
		err = player.doAddCommand(cmd, music)
	case "clear":
		err = player.Clear()
	}
	return err
}

func (player *MpdPlayer) doAddCommand(cmd *PlaylistCommand, music model.Collection) (err error) {
	if cmd.Album == "" {
		log.Printf("Don't play artists (or nulls)\n")
		return errors.New("Playing artists is not implemented")
	}

	log.Printf("Trying to add %s/%s/%s/%s to playlist (%v)\n",
		cmd.Category, cmd.Artist, cmd.Album, cmd.Track, cmd.Immediate)

	var album *model.Album = nil
	var track *model.Track = nil
	if cmd.Track == "" {
		album, err = model.GetAlbum(&music, cmd.Category, cmd.Artist,
			cmd.Album)
	} else {
		track, err = model.GetTrack(&music, cmd.Category, cmd.Artist,
			cmd.Album, cmd.Track)
	}
	if err != nil {
		log.Printf("Album not found.")
		return err
	}

	if cmd.Immediate {
		err = player.Clear()
		if err != nil {
			log.Printf("Error clearing playlist")
			return err
		}
	}

	if track != nil {
		err = player.AddTrack(track)
	}
	if album != nil {
		err = player.AddAlbum(album)
	}
	if err != nil {
		log.Printf("Error adding album or track %s/%s", cmd.Album, cmd.Track)
		return err
	}
	if cmd.Immediate {
		err = player.Play()
	}
	return err
}
