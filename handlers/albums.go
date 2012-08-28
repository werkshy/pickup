package handlers

import (
	"fmt"
	"encoding/json"
	"net/http"
	//"io/ioutil"
	"strings"
	"time"
	"pickup/model"
)

type AlbumHandler struct {
	Music model.Collection
}


// Return a list of albums or a specific album
func (h AlbumHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	path := r.URL.Path[len("/albums/"):]
    parts := strings.SplitN(path, "/", 2)
	fmt.Printf("Path: %s  parts: %q   len(parts): %d\n", r.URL.Path, parts,
			len(parts))
	// If only one part, we'll search for it
	if len(parts) == 1 {
		query := parts[0]
		if query != "" {
			fmt.Printf("Showing album search results for '%s'\n", query)
			h.searchAlbums(w, query)
		} else {
			fmt.Printf("Showing all albums\n")
			h.listAllAlbums(w)
		}
		return
	}
	// Otherwise we assume artist/album
	artist := parts[0]
	album := parts[1]

	fmt.Printf("Looking up album '%s - %s\n", artist, album)

    fmt.Fprintf(w, "\n<h1>Hello</h1><div>world</div>\n")
}

func (h AlbumHandler) listAllAlbums(w http.ResponseWriter) {
	t0 := time.Now()
    fmt.Printf("All albums (%d)\n", len(h.Music.Albums))
	// Convert to Album Summary to save on info
	albumSummaries := make([]model.AlbumSummary, len(h.Music.Albums))
	for i := 0; i< len(h.Music.Albums); i++ {
		albumSummaries[i] = model.NewAlbumSummary(h.Music.Albums[i])
	}
	j, _ := json.Marshal(albumSummaries)
	fmt.Println("Time to marshall all albums:", time.Since(t0))
	t1 := time.Now()
	w.Write(j)
	fmt.Println("Time to send all albums:", time.Since(t1))
}

func (h AlbumHandler) searchAlbums(w http.ResponseWriter, query string) {
	matches := model.SearchAlbums(h.Music, query)
	fmt.Printf("Found %d results\n", len(matches))
	for _, item := range matches{
		fmt.Printf("%s - %s\n", item.Artist, item.Name)
	}
	j, _ := json.Marshal(matches)
	w.Write(j)
	return
}
