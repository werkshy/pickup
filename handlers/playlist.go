package handlers

import (
	"encoding/json"
	"log"
	"net/http"
	//"io/ioutil"
	"strings"
	//"time"
	"pickup/model"
	"pickup/player"
)

type PlaylistHandler struct {
	Music model.Collection
}


// Return a list of albums or a specific album
func (h PlaylistHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	path := r.URL.Path[len("/playlist/"):]
    parts := strings.SplitN(path, "/", 2)
	log.Printf("Path: %s  parts: %q   len(parts): %d\n", r.URL.Path, parts,
			len(parts))
	// No parts: return current playlist
	if len(parts) == 0 || len(parts[0]) == 0 {
		log.Printf("Showing current playlist\n")
		h.currentPlaylist(w)
	} else {
		// Otherwise we have playlist command
		h.command(w, parts[0], parts[1:])
	}
}

func (h PlaylistHandler) currentPlaylist(w http.ResponseWriter) {
	playlist := player.NewMpdPlaylist(h.Music.MusicDir)

	// get the contents of the playlist
	currentTracks, err := playlist.List()
	if (err != nil) {
		log.Printf("Error getting playlist: %s", err)
		currentTracks = make([]string, 0)
	}
	log.Printf("Current playlist: (%d tracks)", len(currentTracks))
	j, _ := json.Marshal(currentTracks)
	w.Write(j)
}

func (h PlaylistHandler) command(w http.ResponseWriter,
		command string, args []string) {
	log.Printf("Received playlist command '%s'\n", command)
}
