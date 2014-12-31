package handlers

import (
	"encoding/json"
	"errors"
	"log"
	"net/http"
	"time"

	"github.com/werkshy/pickup/model"
	"github.com/werkshy/pickup/player"
)

type PlaylistHandler struct {
	player.Player
}

// Return a list of albums or a specific album
func (h PlaylistHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	t0 := time.Now()
	var err error

	switch r.Method {
	case "GET":
		err = h.currentPlaylist(w)
	case "POST":
		err = h.command(w, r)
	}

	if err != nil {
		log.Printf("Error detected in /playlist: %s", err)
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
	log.Printf("%-5s %-40s %v", r.Method, r.URL, time.Since(t0))
}

func (h PlaylistHandler) currentPlaylist(w http.ResponseWriter) error {
	// get the contents of the playlist
	playlist := h.GetPlaylist()
	currentTracks, err := playlist.List()
	if err != nil {
		log.Printf("Error getting playlist: %s", err)
		return err
	}
	j, _ := json.Marshal(currentTracks)
	w.Write(j)
	return err
}

type PlaylistCommand struct {
	Command   string
	Category  string
	Artist    string
	Album     string
	Track     string
	Immediate bool
}

// dispatch playlist commands (add, clear etc)
func (h PlaylistHandler) command(w http.ResponseWriter, r *http.Request) (err error) {
	var data PlaylistCommand
	err = JsonRequestToType(w, r, &data)
	if err != nil {
		return err
	}
	log.Printf("Received playlist command '%s'\n", data.Command)

	switch data.Command {
	case "add":
		err = h.add(data)
	case "clear":
		err = h.clear()
	}
	return err
}

func (h PlaylistHandler) add(data PlaylistCommand) (err error) {
	music := h.Player.GetMusic()
	if data.Album == "" {
		log.Printf("Don't play artists (or nulls)\n")
		return errors.New("Playing artists is not implemented")
	}

	log.Printf("Trying to add %s/%s/%s/%s to playlist (%v)\n",
		data.Category, data.Artist, data.Album, data.Track, data.Immediate)

	var album *model.Album = nil
	var track *model.Track = nil
	if data.Track == "" {
		album, err = model.GetAlbum(music, data.Category, data.Artist,
			data.Album)
	} else {
		track, err = model.GetTrack(music, data.Category, data.Artist,
			data.Album, data.Track)
	}
	if err != nil {
		log.Printf("Album not found.")
		return err
	}

	playlist := h.GetPlaylist()
	if data.Immediate {
		err = playlist.Clear()
		if err != nil {
			log.Printf("Error clearing playlist")
			return err
		}
	}

	if track != nil {
		err = playlist.AddTrack(track)
	}
	if album != nil {
		err = playlist.AddAlbum(album)
	}
	if err != nil {
		log.Printf("Error adding album or track %s/%s", data.Album, data.Track)
		return err
	}
	if data.Immediate {
		controls := h.GetControls()
		err = controls.Play()
	}
	return err
}

func (h PlaylistHandler) clear() (err error) {
	playlist := h.GetPlaylist()
	return playlist.Clear()
}
