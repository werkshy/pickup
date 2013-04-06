package model

import (
	"github.com/werkshy/gompd/mpd"
	"log"
	"path"
	"pickup/config"
	"strings"
	"time"
)

func RefreshMpd(conf *config.Config) (Collection, error) {
	conn, err := mpd.DialAuthenticated("tcp", *conf.MpdAddress,
		*conf.MpdPassword)
	log.Println("Getting mpd files")
	t0 := time.Now()
	files, err := conn.GetFiles()
	log.Printf("Getting %v files from mpd took %d ms", len(files),
		time.Since(t0)/time.Millisecond)
	t1 := time.Now()

	// Files come back from mpd sorted, so we can track the current
	// artists/albums as we iterate through the files.
	rootCategory := NewCategory("Music")
	collection := Collection {
		make([]*Category, 0),
	}
	collection.addCategory(rootCategory)

	var currentArtist *Artist
	var currentAlbum *Album
	var currentCategory *Category = rootCategory
	for _, file := range files {
		parts := strings.Split(file, "/")
		nparts := len(parts)
		if nparts < 2 {
			log.Printf("Can't handle '%s'\n", file)
			continue
		}
		thisTrack := parts[nparts-1]
		thisAlbum := parts[nparts-2]
		//thisTrack += "" // leave me alone golang!

		// Occassionally I have e.g. _mp3/ folders that I want to ignore
		if strings.HasPrefix(thisAlbum, "_") {
			//log.Printf("Ignoring album '%s'\n", file)
			continue
		}

		npartsWithArtist := 3 // expect artist, album, track
		// If the path begins with _, it's a subcategory e.g. _Soundtracks
		if strings.HasPrefix(file, "_") {
			npartsWithArtist = 4 // category, artist, album, track
		}
		// Sanity check the path for too many or too few parts
		// one less that nparts is OK, it means a bare album
		if len(parts) < npartsWithArtist-1 || len(parts) > npartsWithArtist {
			log.Printf("%s has %d parts", file, len(parts))
			continue
		}

		// handle currentAlbum
		if currentAlbum == nil {
			currentAlbum = NewAlbum(thisAlbum)
			currentAlbum.Path = path.Dir(file)
		} else if currentAlbum.Name != thisAlbum {
			// handle finished album
			wrapUpAlbum(currentAlbum, currentArtist, currentCategory)
			currentAlbum = NewAlbum(thisAlbum)
			currentAlbum.Path = path.Dir(file)
		}

		// Handle currentArtist
		if nparts == npartsWithArtist {
			thisArtist := parts[nparts-3]
			// Create a new artist if the artist has changed
			if currentArtist == nil {
				currentArtist = NewArtist(thisArtist)
			} else if currentArtist.Name != thisArtist {
				// handle finished artist
				wrapUpArtist(currentArtist, currentCategory)
				currentArtist = NewArtist(thisArtist)
			}
		} else {
			// Looking at a bare album
			if currentArtist != nil {
				// handle finished album if currentArtist != nil
				wrapUpArtist(currentArtist, currentCategory)
			}
			currentArtist = nil
		}

		// handle currentCategory
		if strings.HasPrefix(file, "_") {
			// This file is part of a subcategory
			thisCategory := parts[0]
			if currentCategory == rootCategory {
				currentCategory = NewCategory(thisCategory)
			} else if currentCategory.Name != thisCategory {
				// handle finished subcategory
				wrapUpCategory(currentCategory, &collection)
				currentCategory = NewCategory(thisCategory)
			}
		} else {
			// this file is not in a subcategory, revert to root category
			if currentCategory != rootCategory {
				// handle finished sub category
				wrapUpCategory(currentCategory, &collection)
			}
			currentCategory = rootCategory
		}
		track := Track {thisTrack, file, currentAlbum.Name, ""}
		if currentAlbum != nil {
			track.Album = currentAlbum.Name
		}
		currentAlbum.Tracks = append(currentAlbum.Tracks, &track)
	}

	// handle final category, artist and album
	if currentAlbum != nil {
		wrapUpAlbum(currentAlbum, currentArtist, currentCategory)
	}
	if currentArtist != nil {
		// handle finished artist
		wrapUpArtist(currentArtist, currentCategory)
	}
	if currentCategory != nil {
		// handle finished category
		wrapUpCategory(currentCategory, &collection)
	}

	log.Printf("Sorting mpd results took %d ms\n", time.Since(t1)/time.Millisecond)
	log.Printf("Found %d categories\n", len(collection.Categories))
	for _, category := range collection.Categories {
		log.Printf("    %s\n", category.Name)
	}
	return collection, err
}

func wrapUpAlbum(album *Album, artist *Artist, category *Category) {
	album.Category = category.Name
	if (artist != nil) {
		album.Artist = artist.Name
		artist.Albums = append(artist.Albums, album)
	} else { // bare album, no artist
		category.Albums = append(category.Albums, album)
	}
}

func wrapUpArtist(artist *Artist, category *Category) {
	category.Artists = append(category.Artists, artist)
}

func wrapUpCategory(category *Category, collection *Collection) {
	log.Printf("Wrapping up category: %s", category.Name)
	collection.addCategory(category)
}
