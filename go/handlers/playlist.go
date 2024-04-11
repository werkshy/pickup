package handlers

import (
	"encoding/json"
	"log"
	"net/http"
	"time"

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
	currentTracks, err := h.List()
	if err != nil {
		log.Printf("Error getting playlist: %s", err)
		return err
	}
	j, err := json.Marshal(currentTracks)
	if err != nil {
		log.Printf("Error marshalling playlist: %s", err)
		return err
	}
	w.Write(j)
	return err
}

// dispatch playlist commands (add, clear etc)
func (h PlaylistHandler) command(w http.ResponseWriter, r *http.Request) (err error) {
	var cmd player.PlaylistCommand
	err = JsonRequestToType(w, r, &cmd)
	if err != nil {
		return err
	}
	log.Printf("Received playlist command '%s'\n", cmd.Command)
	err = h.HandlePlaylistCommand(&cmd)

	return err
}
