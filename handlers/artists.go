package handlers

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strings"
	"time"

	"github.com/werkshy/pickup/model"
	"github.com/werkshy/pickup/player"
)

type ArtistHandler struct {
	player.Player
}

// Return a list of artists or search for an artist
func (h ArtistHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	t0 := time.Now()
	path := r.URL.Path[len("/artists/"):]
	parts := strings.SplitN(path, "/", 2)
	log.Printf("Path: %s  parts: %q   len(parts): %d\n", r.URL.Path, parts,
		len(parts))
	// If only one part, we'll search for it
	if len(parts) == 1 {
		query := parts[0]
		if query != "" {
			h.searchArtists(w, query)
		} else {
			h.listAllArtists(w)
		}
	} else {
		// Trailing slash gets you here
		h.showArtist(w, parts[0])
	}
	log.Printf("%-5s %-40s %v", r.Method, r.URL, time.Since(t0))
}

func (h ArtistHandler) listAllArtists(w http.ResponseWriter) {
	// TODO: list all artists
	log.Printf("TOOD: list all artists\n")
	/*
		music := h.GetMusic()
		t0 := time.Now()
		fmt.Printf("All artists (%d)\n", len(music.Artists))
		// Convert to Artist Summary to save on info
		artistSummaries := make([]model.ArtistSummary, len(music.Artists))
		for i := 0; i < len(music.Artists); i++ {
			artistSummaries[i] = music.Artists[i].GetSummary()
		}
		j, _ := json.Marshal(artistSummaries)
		fmt.Println("Time to marshall all artists:", time.Since(t0))
		t1 := time.Now()
		w.Write(j)
		fmt.Println("Time to send all artists:", time.Since(t1))
	*/
}

func (h ArtistHandler) searchArtists(w http.ResponseWriter, query string) {
	music, err := h.GetCollection()
	if err != nil {
		log.Printf("Failed to connect to mpd")
		writeError(w, http.StatusNotFound, "Problem with mpd")
	}
	matches := model.SearchArtists(music, query)
	log.Printf("Found %d artist results:\n", len(matches))
	for _, item := range matches {
		fmt.Printf("\t%s\n", item.Name)
	}
	j, _ := json.Marshal(matches)
	w.Write(j)
	return
}

func (h ArtistHandler) showArtist(w http.ResponseWriter, query string) {
	music, err := h.GetCollection()
	if err != nil {
		log.Printf("Failed to connect to mpd")
		writeError(w, http.StatusNotFound, "Problem with mpd")
	}
	matches := model.SearchArtists(music, query)
	fmt.Printf("Found %d artist results:\n", len(matches))
	for _, item := range matches {
		if item.Name == query {
			log.Printf("Found artist: '%s'\n", item.Name)
			j, _ := json.Marshal(item)
			w.Write(j)
			return
		}
	}
	log.Printf("Artist not found: %s %s\n", query)
	writeError(w, http.StatusNotFound, fmt.Sprintf("No artist found '%s'",
		query))
}
