package main

import (
	"./model"
	"./handlers"
	"flag"
	"fmt"
	"net/http"
)

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
	default:
		fmt.Println("Unknown action", *action)
	}
}

func serve(musicDir string, music model.Collection) bool {
	fmt.Println("Serving from", musicDir)
	albumHandler := handlers.AlbumHandler {music}
	http.Handle("/albums/", albumHandler)
	http.ListenAndServe(":8080", nil)
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
