package player


import (
	"pickup/model"
	"code.google.com/p/gompd/mpd"
	"log"
)

// In theory we could have different backends, so define an interface that will
// allow for that.
type Playlist interface {
	List() ([]string, error) // what should this return? []Track?
	AddAlbum(model.Album) error
	AddTrack(model.Track) error
	AddTracks([]model.Track) error
	Clear() error
}

// In practice I only care about mpd for playback at the moment, aside from
// potential memory issues on low-end hardware.
type MpdPlaylist struct {
	conn *mpd.Client
	musicDir string
}

/**
 * Constructor of MpdPlaylist
 */
func NewMpdPlaylist(musicDir, mpdHost, mpdPassword string) MpdPlaylist {
	conn, err := mpd.DialAuthenticated("tcp", mpdHost, mpdPassword)
	if err != nil {
		log.Fatalln(err)
	}
	return MpdPlaylist { conn, musicDir }
}

/**
 * Implement playlist interface via mpd
 */
func (playlist MpdPlaylist) List() (results []string, err error) {
	log.Printf("Listing playlist\n")
	info, err := playlist.conn.PlaylistInfo(-1, -1)
	if (err != nil) {
		log.Printf("Failed to get playlist info\n")
		return nil, err
	}
	log.Printf("mpd returned %d tracks in playlist\n", len(info))
	for _, entry := range info {
		//log.Printf("%q\n", entry)
		results = append(results, entry["file"])
	}
	return results, nil
}

func (playlist MpdPlaylist) AddAlbum(album model.Album) (err error) {
	log.Printf("Adding album %s - %s (%s)\n", album.Artist, album.Name,
			album.Path)
	uri := playlist.pathToUri(album.Path)
	log.Printf("uri: %s\n", uri)
	return playlist.conn.Add(uri)
}

func (playlist MpdPlaylist) AddTrack(track model.Track) (err error) {
	log.Printf("Adding track %s\n", track)
	uri := playlist.pathToUri(track.Path)
	return playlist.conn.Add(uri)
}

func (playlist MpdPlaylist) AddTracks(tracks []model.Track) (err error) {
	for _, track := range tracks {
		log.Printf("Adding track %s\n", track)
		err := playlist.AddTrack(track)
		if (err != nil) {
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

func (playlist MpdPlaylist) pathToUri(path string) (uri string){
	return path[len(playlist.musicDir) + 1:]
}
