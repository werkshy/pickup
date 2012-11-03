package handlers

import (
	"encoding/json"
	"errors"
	"log"
	"net/http"
	//"io/ioutil"
	//"strings"
	//"time"
	"pickup/model"
	"pickup/player"
)



type PlaylistHandler struct {
	Music model.Collection
}


// Return a list of albums or a specific album
func (h PlaylistHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	var err error = nil
	switch (r.Method) {
	case "GET":
		log.Printf("GET: Showing current playlist\n")
		err = h.currentPlaylist(w)
	case "POST":
		err = h.command(w, r)
	}

	if (err != nil) {
		log.Printf("Error detected in /playlist: %s", err)
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
}

func (h PlaylistHandler) currentPlaylist(w http.ResponseWriter) error {
	playlist := player.NewMpdPlaylist(h.Music.MusicDir)

	// get the contents of the playlist
	currentTracks, err := playlist.List()
	if (err != nil) {
		log.Printf("Error getting playlist: %s", err)
		return err
	}
	log.Printf("Current playlist: (%d tracks)", len(currentTracks))
	j, _ := json.Marshal(currentTracks)
	w.Write(j)
	return err
}

type PlaylistCommand struct {
	Command string
	Artist string
	Album string
}

// dispatch playlist commands (add, clear etc)
func (h PlaylistHandler) command(w http.ResponseWriter,
			r *http.Request) (err error) {
	var data PlaylistCommand
	err = JsonRequestToType(w, r, &data)
	if (err != nil) {
		return err
	}

	log.Printf("Received playlist command '%s'\n", data.Command)
	switch(data.Command) {
		case "add":
			err = h.add(data.Artist, data.Album)
		case "clear":
			err = h.clear();
	}
	return err
}

func (h PlaylistHandler) add(artist string, album string) (err error) {
	playlist := player.NewMpdPlaylist(h.Music.MusicDir)
	if artist == "" || album == "" {
		log.Printf("Don't play artists (or nulls)\n")
		return errors.New("Playing artists is not implemented")
	}
	log.Printf("Trying to add '%s'/'%s' to playlist\n", artist, album)
	albumData, err := model.GetAlbum(h.Music, artist, album)
	if err != nil {
		log.Printf("Album not found.")
		return err
	}
	playlist.Clear()
	return playlist.AddAlbum(*albumData)
}

func (h PlaylistHandler) clear() (err error) {
	playlist := player.NewMpdPlaylist(h.Music.MusicDir)
	return playlist.Clear()
}
