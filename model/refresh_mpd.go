package model

import (
	"code.google.com/p/gompd/mpd"
	"log"
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
	rootCollection := NewCollection("Music")

	var currentArtist *Artist
	var currentAlbum *Album
	var currentCollection *Collection = rootCollection
	/*
		subCollections := make([]Collection, 0)
	*/
	for _, file := range files {
		parts := strings.Split(file, "/")
		nparts := len(parts)
		if nparts < 2 {
			log.Printf("Can't handle '%s'\n", file)
			continue
		}
		thisTrack := parts[nparts-1]
		thisAlbum := parts[nparts-2]
		thisTrack += "" // leave me alone golang!

		// Occassionally I have e.g. _mp3/ folders that I want to ignore
		if strings.HasPrefix(thisAlbum, "_") {
			//log.Printf("Ignoring album '%s'\n", file)
			continue
		}

		npartsWithArtist := 3 // expect artist, album, track
		// If the path begins with _, it's a subcollection e.g. _Soundtracks
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
		} else if currentAlbum.Name != thisAlbum {
			// handle finished album
			wrapUpAlbum(currentAlbum, currentArtist, currentCollection)
			currentAlbum = NewAlbum(thisAlbum)
		}

		// Handle currentArtist
		if nparts == npartsWithArtist {
			thisArtist := parts[nparts-3]
			// Create a new artist if the artist has changed
			if currentArtist == nil {
				currentArtist = NewArtist(thisArtist)
			} else if currentArtist.Name != thisArtist {
				// handle finished artist
				wrapUpArtist(currentArtist, currentCollection)
				currentArtist = NewArtist(thisArtist)
			}
		} else {
			// Looking at a bare album
			if currentArtist != nil {
				// handle finished album if currentArtist != nil
				wrapUpArtist(currentArtist, currentCollection)
			}
			currentArtist = nil
		}

		// handle currentCollection
		if strings.HasPrefix(file, "_") {
			// This file is part of a subcollection
			thisSubCollection := parts[0]
			if currentCollection == rootCollection {
				currentCollection = NewCollection(thisSubCollection)
			} else if currentCollection.Name != thisSubCollection {
				// handle finished subcollection
				wrapUpSubCollection(currentCollection, rootCollection)
				currentCollection = NewCollection(thisSubCollection)
			}
		} else {
			// this file is not in a subcollection, revert to root collection
			if currentCollection != rootCollection {
				// handle finished sub collection
				wrapUpSubCollection(currentCollection, rootCollection)
			}
			currentCollection = rootCollection
		}
	}

	// handle final collection, artist and album
	if currentAlbum != nil {
		wrapUpAlbum(currentAlbum, currentArtist, currentCollection)
	}
	if currentArtist != nil {
		// handle finished artist
		wrapUpArtist(currentArtist, currentCollection)
	}
	if currentCollection != nil {
		// handle finished collection
		wrapUpSubCollection(currentCollection, rootCollection)
	}

	log.Printf("Sorting mpd results took %d ms", time.Since(t1)/time.Millisecond)
	return *rootCollection, err
}

func wrapUpAlbum(album *Album, artist *Artist, collection *Collection) {
	if (artist != nil) {
		artist.Albums = append(artist.Albums, album)
	} else { // bare album, no artist
		collection.Albums = append(collection.Albums, album)
	}
}

func wrapUpArtist(artist *Artist, collection *Collection) {
	collection.Artists = append(collection.Artists, artist)
}

func wrapUpSubCollection(subCollection *Collection, rootCollection *Collection) {
	if subCollection != rootCollection {
		rootCollection.SubCollections = append(
					rootCollection.SubCollections, subCollection)
	}
}
