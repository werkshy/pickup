package handlers

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strings"

	"github.com/werkshy/pickup/model"
	//"time"
)

type ArtistHandler struct {
	Music model.Collection
}

// Return a list of albums or a specific album
func (h ArtistHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	path := r.URL.Path[len("/artists/"):]
	parts := strings.SplitN(path, "/", 2)
	fmt.Printf("Path: %s  parts: %q   len(parts): %d\n", r.URL.Path, parts,
		len(parts))
	// If only one part, we'll search for it
	if len(parts) == 1 {
		query := parts[0]
		if query != "" {
			fmt.Printf("Showing artist search results for '%s'\n", query)
			h.searchArtists(w, query)
		} else {
			fmt.Printf("Showing all artists\n")
			h.listAllArtists(w)
		}
		return
	}
	fmt.Printf("Artist '%s' specified\n", parts[0])
	h.showArtist(w, parts[0])
}

func (h ArtistHandler) listAllArtists(w http.ResponseWriter) {
	// TODO: list all artists
	log.Printf("TOOD: list all artists\n")
	/*
		t0 := time.Now()
		fmt.Printf("All artists (%d)\n", len(h.Music.Artists))
		// Convert to Artist Summary to save on info
		artistSummaries := make([]model.ArtistSummary, len(h.Music.Artists))
		for i := 0; i < len(h.Music.Artists); i++ {
			artistSummaries[i] = h.Music.Artists[i].GetSummary()
		}
		j, _ := json.Marshal(artistSummaries)
		fmt.Println("Time to marshall all artists:", time.Since(t0))
		t1 := time.Now()
		w.Write(j)
		fmt.Println("Time to send all artists:", time.Since(t1))
	*/
}

func (h ArtistHandler) searchArtists(w http.ResponseWriter, query string) {
	matches := model.SearchArtists(h.Music, query)
	fmt.Printf("Found %d artist results:\n", len(matches))
	for _, item := range matches {
		fmt.Printf("\t%s\n", item.Name)
	}
	j, _ := json.Marshal(matches)
	w.Write(j)
	return
}

func (h ArtistHandler) showArtist(w http.ResponseWriter, query string) {
	matches := model.SearchArtists(h.Music, query)
	fmt.Printf("Found %d artist results:\n", len(matches))
	for _, item := range matches {
		if item.Name == query {
			fmt.Printf("Found artist: '%s'\n", item.Name)
			j, _ := json.Marshal(item)
			w.Write(j)
			return
		}
	}
	fmt.Printf("Artist not found: %s %s\n", query)
	writeError(w, http.StatusNotFound, fmt.Sprintf("No artist found '%s'",
		query))
}
