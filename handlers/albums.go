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

type AlbumHandler struct {
	player.Player
}

// Return a list of albums or a specific album
func (h AlbumHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	t0 := time.Now()
	path := r.URL.Path[len("/albums/"):]
	parts := strings.SplitN(path, "/", 3)
	log.Printf("Path: %s  parts: %q   len(parts): %d\n", r.URL.Path, parts,
		len(parts))
	// If only one part, we'll search for it
	if len(parts) == 1 {
		query := parts[0]
		if query != "" {
			h.searchAlbums(w, query)
		} else {
			h.listAllAlbums(w)
		}
	} else if len(parts) == 2 {
		// category/album
		h.getAlbum(w, parts[0], "", parts[1])
	} else {
		// Otherwise we assume category/artist/album
		h.getAlbum(w, parts[0], parts[1], parts[2])
	}
	log.Printf("%-5s %-40s %v", r.Method, r.URL, time.Since(t0))
}

func (h AlbumHandler) getAlbum(w http.ResponseWriter,
	categoryName string, artistName string, albumName string) {
	log.Printf("Looking up album '%s/%s/%s\n", categoryName,
		artistName, albumName)
	music, err := h.GetCollection()
	if err != nil {
		log.Printf("Failed to connect to mpd")
		writeError(w, http.StatusNotFound, "Problem with mpd")
	}
	album, err := model.GetAlbum(music, categoryName, artistName, albumName)

	if err == nil {
		log.Printf("Found album: %s/%s; %d tracks", album.Artist, album.Name, len(album.Tracks))
		summary := model.NewAlbumSummary(album)
		j, _ := json.Marshal(summary)
		w.Write(j)
		return
	}
	log.Printf("Did not find album: %s/%s/%s (%v)", categoryName, artistName,
		albumName, err)

	writeError(w, http.StatusNotFound, fmt.Sprintf("Album not found '%s'",
		albumName))
}

func (h AlbumHandler) listAllAlbums(w http.ResponseWriter) {
	// TODO: list all albums
	log.Printf("TOOD: list all albums\n")
	/*
		music := h.GetMusic()
		t0 := time.Now()
		fmt.Printf("All albums (%d)\n", len(music.Albums))
		// Convert to Album Summary to save on info
		albumSummaries := make([]model.AlbumSummary, len(music.Albums))
		for i := 0; i < len(music.Albums); i++ {
			albumSummaries[i] = model.NewAlbumSummary(music.Albums[i])
		}
		j, _ := json.Marshal(albumSummaries)
		fmt.Println("Time to marshall all albums:", time.Since(t0))
		t1 := time.Now()
		w.Write(j)
		fmt.Println("Time to send all albums:", time.Since(t1))
	*/
}

func (h AlbumHandler) searchAlbums(w http.ResponseWriter, query string) {
	log.Printf("Searching for albums matching  '%s'\n", query)
	music, err := h.GetCollection()
	if err != nil {
		log.Printf("Failed to connect to mpd")
		writeError(w, http.StatusNotFound, "Problem with mpd")
	}
	matches := model.SearchAlbums(music, query)
	log.Printf("Found %d results\n", len(matches))
	for _, item := range matches {
		log.Printf("%s - %s\n", item.Artist, item.Name)
	}
	j, _ := json.Marshal(matches)
	w.Write(j)
	return
}
