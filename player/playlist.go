package player

import (
	"log"

	"github.com/werkshy/gompd/mpd"
	"github.com/werkshy/pickup/config"
	"github.com/werkshy/pickup/model"
)

type PlaylistTrack struct {
	Pos    string
	Name   string
	Artist string
	Album  string
	Path   string
}

// In theory we could have different backends, so define an interface that will
// allow for that.
type Playlist interface {
	List() ([]PlaylistTrack, error) // what should this return? []Track?
	AddAlbum(*model.Album) error
	AddTrack(*model.Track) error
	AddTracks([]*model.Track) error
	Clear() error
	Close() error
}

// In practice I only care about mpd for playback at the moment, aside from
// potential memory issues on low-end hardware.
type MpdPlaylist struct {
	conn *mpd.Client
}

func (t *PlaylistTrack) CleanUp(file string) {
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

/**
 * Constructor of MpdPlaylist
 */
func NewMpdPlaylist(conf *config.Config) MpdPlaylist {
	conn, err := mpd.DialAuthenticated("tcp", *conf.MpdAddress,
		*conf.MpdPassword)
	if err != nil {
		log.Fatalln(err)
	}
	return MpdPlaylist{conn}
}

func (playlist MpdPlaylist) Close() (err error) {
	return playlist.conn.Close()
}

/**
 * Implement playlist interface via mpd
 */
func (playlist MpdPlaylist) List() (results []PlaylistTrack, err error) {
	info, err := playlist.conn.PlaylistInfo(-1, -1)
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
			entry["Path"]}
		track.CleanUp(entry["file"])
		results = append(results, track)
	}
	return results, nil
}

func (playlist MpdPlaylist) AddAlbum(album *model.Album) (err error) {
	log.Printf("Adding album %s - %s (%s)\n", album.Artist, album.Name,
		album.Path)
	return playlist.conn.Add(album.Path)
}

func (playlist MpdPlaylist) AddTrack(track *model.Track) (err error) {
	log.Printf("Adding track %v\n", track)
	return playlist.conn.Add(track.Path)
}

func (playlist MpdPlaylist) AddTracks(tracks []*model.Track) (err error) {
	for _, track := range tracks {
		log.Printf("Adding track %s\n", track)
		err := playlist.AddTrack(track)
		if err != nil {
			return err
		}
	}
	return nil
}

func (playlist MpdPlaylist) Clear() (err error) {
	log.Println("Clearing playlist")
	playlist.conn.Clear()
	return nil
}
