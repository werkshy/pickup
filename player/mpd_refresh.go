package player

import (
	"log"
	"path"
	"strings"
	"time"

	"github.com/werkshy/pickup/model"
)

func (player *MpdPlayer) RefreshCollection() (model.Collection, error) {

	log.Println("Getting mpd files")
	t0 := time.Now()
	files, err := player.conn.GetFiles()
	if err != nil {
		return model.Collection{}, err
	}

	log.Printf("Getting %v files from mpd took %d ms", len(files),
		time.Since(t0)/time.Millisecond)
	t1 := time.Now()

	// Files come back from mpd sorted, so we can track the current
	// artists/albums as we iterate through the files.
	rootCategory := model.NewCategory("Music")
	collection := model.Collection{
		make([]*model.Category, 0),
	}
	collection.AddCategory(rootCategory)

	var currentArtist *model.Artist
	var currentAlbum *model.Album
	var currentCategory *model.Category = rootCategory
	for _, file := range files {
		category, artist, album, track, err := model.PathToParts(file)
		if err != nil {
			log.Printf("Error at %s: %v\n", file, err)
			continue
		}

		// Occassionally I have e.g. _mp3/ folders that I want to ignore
		if strings.HasPrefix(album, "_") {
			log.Printf("Ignoring album '%s'\n", file)
			continue
		}

		// handle currentAlbum
		if currentAlbum == nil {
			currentAlbum = model.NewAlbum(album)
			currentAlbum.Path = path.Dir(file)
		} else if currentAlbum.Name != album {
			// handle finished album
			wrapUpAlbum(currentAlbum, currentArtist, currentCategory)
			currentAlbum = model.NewAlbum(album)
			currentAlbum.Path = path.Dir(file)
		}

		// Handle currentArtist
		if artist != "" {
			// Create a new artist if the artist has changed
			if currentArtist == nil {
				currentArtist = model.NewArtist(artist)
			} else if currentArtist.Name != artist {
				// handle finished artist
				wrapUpArtist(currentArtist, currentCategory)
				currentArtist = model.NewArtist(artist)
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
		if category != "" {
			// This file is part of a subcategory
			if currentCategory == rootCategory {
				currentCategory = model.NewCategory(category)
			} else if currentCategory.Name != category {
				// handle finished subcategory
				wrapUpCategory(currentCategory, &collection)
				currentCategory = model.NewCategory(category)
			}
		} else {
			// this file is not in a subcategory, revert to root category
			if currentCategory != rootCategory {
				// handle finished sub category
				wrapUpCategory(currentCategory, &collection)
			}
			currentCategory = rootCategory
		}
		currentTrack := model.Track{track, file, currentAlbum.Name, ""}
		if currentAlbum != nil {
			currentTrack.Album = currentAlbum.Name
		}
		currentAlbum.Tracks = append(currentAlbum.Tracks, &currentTrack)
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
	//for _, category := range collection.Categories {
	//	log.Printf("    %s\n", category.Name)
	//}
	return collection, err
}

func wrapUpAlbum(album *model.Album, artist *model.Artist, category *model.Category) {
	album.Category = category.Name
	if artist != nil {
		album.Artist = artist.Name
		artist.Albums = append(artist.Albums, album)
	} else { // bare album, no artist
		category.Albums = append(category.Albums, album)
	}
}

func wrapUpArtist(artist *model.Artist, category *model.Category) {
	category.Artists = append(category.Artists, artist)
}

func wrapUpCategory(category *model.Category, collection *model.Collection) {
	log.Printf("Wrapping up category: %s", category.Name)
	collection.AddCategory(category)
}
