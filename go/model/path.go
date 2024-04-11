package model

import (
	"errors"
	"log"
	"strings"
)

/*
Convert a path (e.g. returned by MPD) into it's track name, album, artist etc
*/
func PathToParts(path string) (category string, artist string, album string, track string, err error) {
	parts := strings.Split(path, "/")
	nparts := len(parts)
	if nparts < 2 {
		log.Printf("Can't handle '%s'\n", path)
		return "", "", "", "", errors.New("Too few parts")
	}
	track = parts[nparts-1]
	album = parts[nparts-2]

	// Occassionally I have e.g. _mp3/ folders that I want to ignore
	if strings.HasPrefix(album, "_") {
		//log.Printf("Ignoring album '%s'\n", file)
		return "", "", album, track, nil
	}

	npartsWithArtist := 3 // expect artist, album, track
	// If the path begins with _, it's a subcategory e.g. _Soundtracks
	if strings.HasPrefix(path, "_") {
		npartsWithArtist = 4 // category, artist, album, track
	}
	// Sanity check the path for too many or too few parts
	// one less that nparts is OK, it means a bare album
	if len(parts) < npartsWithArtist-1 || len(parts) > npartsWithArtist {
		log.Printf("%s has %d parts", path, len(parts))
		return "", "", "", "", errors.New("Wrong number of parts")
	}

	// Handle currentArtist
	if nparts == npartsWithArtist {
		artist = parts[nparts-3]
	}

	// handle currentCategory
	if strings.HasPrefix(path, "_") {
		// This file is part of a subcategory
		category = parts[0]
	}
	return
}
