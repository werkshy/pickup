//usr/bin/env go run "$0" "$@"; exit
package main

import (
	"flag"
	"fmt"
	"net/http"
	"log"
	"os"
	"pickup/handlers"
	"pickup/model"
	"pickup/player"
)

var Port = 8080

func main() {
	var action = flag.String("action", "serve", "Action to perform (serve|refresh)")
	var musicDir = flag.String("music-dir", "/music", "Music dir")
	var query = flag.String("query", "", "Search query")
	flag.Parse()

	fmt.Println("Action is:", *action)

	collection := loadOrRefresh(*musicDir)

	switch *action {
	case "stats":
		stats(collection)
	case "search":
		search(collection, *query)
	case "serve":
		serve(*musicDir, collection)
	case "save":
		save(collection)
	case "test-playback":
		testPlayback(collection)
	default:
		fmt.Println("Unknown action", *action)
	}
}

func serve(musicDir string, music model.Collection) bool {
	albumHandler := handlers.AlbumHandler{music}
	artistHandler := handlers.ArtistHandler{music}
	playlistHandler, err := handlers.NewPlaylistHandler(music)
	controlHandler, err := handlers.NewControlHandler()
	if err != nil {
		log.Fatalln("Couldn't connect to MPD")
	}
	http.Handle("/albums/", albumHandler)
	http.Handle("/artists/", artistHandler)
	http.Handle("/playlist/", playlistHandler)
	http.Handle("/control/", controlHandler)
	staticDir, _ := os.Getwd()
	staticDir = staticDir + "/static"
	fmt.Printf("Serving static files from %s\n", staticDir)
	// strip '/static' from the url to get the name of the file within the
	// static dir.
	http.Handle("/static/", http.StripPrefix("/static/",
			http.FileServer(http.Dir(staticDir))))
	http.HandleFunc("/", handlers.Index)
	var bind = fmt.Sprintf(":%d", Port)
	fmt.Printf("Serving from %s on %s\n", musicDir, bind)
	http.ListenAndServe(bind, nil)
	return true
}

func stats(music model.Collection) {
	fmt.Printf("%d tracks, %d albums, %d artists\n",
		len(music.Tracks), len(music.Albums),
		len(music.Artists))
}

func search(music model.Collection, query string) {
	fmt.Println("All music:")
	stats(music)
	matching := model.Search(music, query)
	fmt.Printf("Matches for '%s':\n", query)
	stats(matching)

	fmt.Println("\nMatching Tracks:")
	for _, track := range matching.Tracks {
		fmt.Printf("%-40s (%-20s)\n", track.Name, track.Artist)
	}

	fmt.Println("\n\nMatching Albums:")
	for _, album := range matching.Albums {
		fmt.Printf("%-40s (%s)\n", album.Name, album.Artist)
	}
}

func loadOrRefresh(musicDir string) model.Collection {
	collection, err := model.Load()
	if err != nil {
		fmt.Printf("No collection loaded, refreshing\n")
		collection = model.Refresh(musicDir)
	}
	return collection
}

func save(music model.Collection) error {
	err := music.Save()
	if err != nil {
		fmt.Println(err)
	}
	return err
}

func testPlayback(music model.Collection) error {
	playlist := player.NewMpdPlaylist(music.MusicDir)
	playlist.Clear()

	// add any old album
	album := music.Albums[2]
	log.Printf("Playing album %s - %s\n", album.Artist, album.Name)
	playlist.AddAlbum(album)

	// get the contents of the playlist
	currentTracks, err := playlist.List()
	log.Printf("Current playlist: (%d tracks)", len(currentTracks))
	for _, track := range currentTracks {
		log.Printf("%s\n", track)
	}
	return err
}
