package model


import (
	"fmt"
	"io/ioutil"
	"path"
	"path/filepath"
	"time"
)

var MusicExtensions = map[string]bool {
		".mp3": true,
		".m4a": true,
	}

/**
 * Recursively process a directory, creating Tracks, Albums and Artists 
 * with the music found.
*/
func ProcessDir(dir string, parent string) (
			tracks []Track, albums []Album, artists []Artist) {
	//fmt.Println("Processing dir", dir)
	list, err := ioutil.ReadDir(dir)
	if err != nil {
		fmt.Println("Error reading", dir, err)
	}
	//var tracks []Track
	var currentArtist Artist
	var currentAlbum Album
	for _, entry := range list {
		//fmt.Println("Looking at", entry.Name())
		if entry.IsDir() {
			var subPath = filepath.Join(dir, entry.Name())
			subTracks, subAlbums, subArtists := ProcessDir(subPath, dir)
			//fmt.Printf("Found %d tracks in subdir %s\n", len(subTracks), subPath)
			//tracks = append(tracks, subTracks...)
			//albums = append(albums, subAlbums...)
			artists = append(artists, subArtists...)
			// If there were subtracks returned, create an album
			if len(subTracks) > 0 {
				album := Album{entry.Name(), subPath, subTracks,
						currentArtist.Name}
				//fmt.Printf("Created album %s\n", entry.Name())
				albums = append(albums, album)
			}

			// If there were albums returned, create an artist
			if len(subAlbums) > 0 {
				artist := Artist{ entry.Name(), subPath, subAlbums}
				artists = append(artists, artist)
				fmt.Printf("Created artist %s\n", entry.Name())
				for _, subAlbum := range subAlbums {
					fmt.Printf("\t %s - %s\n", artist.Name, subAlbum.Name);
					subAlbum.Artist = artist.Name
				}
			}

		} else {
			var ext = path.Ext(entry.Name())
			if ! MusicExtensions[ext] {
				continue
			}
			trackPath := filepath.Join(dir, entry.Name())
			track := Track{entry.Name(), trackPath, currentAlbum.Name,
					currentArtist.Name}
			tracks = append(tracks, track)
			//fmt.Printf("Found track %s in %s\n", track.Name, dir)
		}
	}
	return tracks, albums, artists
}

func Refresh(musicDir string) Collection {

	t0 := time.Now()
	fmt.Printf("Refreshing from '%s'\n", musicDir)
	_, _, artists := ProcessDir(musicDir, "")
	fmt.Printf("Found %d artists\n", len(artists))
	fmt.Println("Time to refresh music: ", time.Since(t0))

	t1 := time.Now()
	var albums =  make([]Album, 0, 10)
	var tracks =  make([]Track, 0, 10)

	for _, artist := range artists {
		for _, album := range artist.Albums {
			album.Artist = artist.Name
			albums = append(albums, album)
			for _, track := range album.Tracks {
				track.Artist = artist.Name
				track.Album = album.Name
				tracks = append(tracks, track)
			}
		}
	}
	fmt.Println("Time to sort music: ", time.Since(t1))
	return Collection {artists, albums, tracks}
}

