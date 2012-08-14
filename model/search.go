package model

import (
	"fmt"
	"strings"
	"time"
)

/*
	Search in the music collection for items matching the query string
*/
func Search(music Collection, query string) (
		matching Collection) {
	lQuery := strings.ToLower(query)
	t0 := time.Now()

	// TODO: use Item interface to reduce copy-paste
	for _, item := range music.Tracks {
		if strings.Contains(strings.ToLower(item.Name), lQuery) {
			matching.Tracks = append(matching.Tracks, item)
		}
	}
	fmt.Println("Time to search tracks: ", time.Since(t0))
	t1 := time.Now()
	for _, item := range music.Albums {
		if strings.Contains(strings.ToLower(item.Name), lQuery) {
			matching.Albums = append(matching.Albums, item)
		}
	}
	fmt.Println("Time to search albums: ", time.Since(t1))
	t2 := time.Now()
	for _, item := range music.Artists {
		if strings.Contains(item.Name, query) {
			matching.Artists = append(matching.Artists, item)
		}
	}
	fmt.Println("Time to search artists: ", time.Since(t2))
	return matching
}

func SearchAlbums(music Collection, query string) (matching []AlbumSummary) {
	t0 := time.Now()
	lQuery := strings.ToLower(query)
	for _, item := range music.Albums {
		if strings.Contains(strings.ToLower(item.Name), lQuery) {
			matching = append(matching, item)
		}
	}
	fmt.Println("Time to search albums: ", time.Since(t0))
	return matching
}
