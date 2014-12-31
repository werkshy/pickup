package model

import (
	"fmt"
	"strings"
	"time"
)

/*
	Search in the music collection for items matching the query string.
	For now an in-memory linear scan is fine (about 4ms for album searches on my
	laptop)
*/
func Search(music *Collection, query string) (matching Category) {

	lQuery := strings.ToLower(query)
	t0 := time.Now()

	for _, category := range music.Categories {
		// TODO: use Item interface to reduce copy-paste
		for _, item := range category.Tracks {
			if strings.Contains(strings.ToLower(item.Name), lQuery) {
				matching.Tracks = append(matching.Tracks, item)
			}
		}
		fmt.Println("Time to search tracks: ", time.Since(t0))
		t1 := time.Now()
		for _, item := range category.Albums {
			if strings.Contains(strings.ToLower(item.Name), lQuery) {
				matching.Albums = append(matching.Albums, item)
			}
		}
		fmt.Println("Time to search albums: ", time.Since(t1))
		t2 := time.Now()
		for _, item := range category.Artists {
			if strings.Contains(item.Name, query) {
				matching.Artists = append(matching.Artists, item)
			}
		}
		fmt.Println("Time to search artists: ", time.Since(t2))
	}
	return matching
}

func SearchAlbums(music *Collection, query string) (matching []AlbumSummary) {
	t0 := time.Now()
	lQuery := strings.ToLower(query)
	for _, category := range music.Categories {
		for _, item := range category.Albums {
			if strings.Contains(strings.ToLower(item.Name), lQuery) {
				matching = append(matching, NewAlbumSummary(item))
			}
		}
	}
	fmt.Println("Time to search albums: ", time.Since(t0))
	return matching
}

func SearchArtists(music *Collection, query string) (matching []ArtistSummary) {
	t0 := time.Now()
	lQuery := strings.ToLower(query)
	for _, category := range music.Categories {
		for _, item := range category.Artists {
			if strings.Contains(strings.ToLower(item.Name), lQuery) {
				matching = append(matching, item.GetSummary())
			}
		}
	}
	fmt.Println("Time to search artists: ", time.Since(t0))
	return matching
}

func GetAlbum(music *Collection, categoryName string, artistName string, albumName string) (*Album, error) {
	for _, category := range music.Categories {
		if category.Name == categoryName {
			if artistName == "" {
				for _, album := range category.Albums {
					if album.Name == albumName {
						return album, nil
					}
				}
			}
			for _, artist := range category.Artists {
				if artist.Name == artistName {
					for _, album := range artist.Albums {
						if album.Name == albumName {
							return album, nil
						}
					}
				}
			}
		}
	}
	return nil, fmt.Errorf("Album not found: %s/%s/%s", categoryName,
		artistName, albumName)
}

func GetTrack(music *Collection, categoryName string,
	artistName string, albumName string, trackName string) (*Track, error) {
	var theAlbum *Album = nil
	for _, category := range music.Categories {
		if category.Name == categoryName {
			if artistName == "" {
				for _, album := range category.Albums {
					if album.Name == albumName {
						theAlbum = album
					}
				}
			} else {
				for _, artist := range category.Artists {
					if artist.Name == artistName {
						for _, album := range artist.Albums {
							if album.Name == albumName {
								theAlbum = album
							}
						}
					}
				}
			}
		}
		if theAlbum != nil {
			for _, track := range theAlbum.Tracks {
				if track.Name == trackName {
					return track, nil
				}
			}
		}
	}
	return nil, fmt.Errorf("Track not found: %s/%s/%s/%s", categoryName,
		artistName, albumName, trackName)
}
