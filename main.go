//usr/bin/env go run "$0" $@; exit
package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"github.com/werkshy/pickup/config"
	"github.com/werkshy/pickup/handlers"
	"github.com/werkshy/pickup/model"
	"github.com/werkshy/pickup/player"
	flag "launchpad.net/gnuflag"
)

var Port = 8080

func main() {
	var action = flag.String("action", "serve", "Action to perform (serve|refresh)")
	conf := config.Config{}
	conf.MusicDir = flag.String("music-dir", "/music", "Music dir")
	conf.MpdAddress = flag.String("mpd-address", "localhost:6600", "MPD address")
	conf.MpdPassword = flag.String("mpd-password", "", "MPD Password")

	var query = flag.String("query", "", "Search query")
	flag.Parse(true)

	fmt.Println("Action is:", *action)
	fmt.Printf("Mpd address: '%s'  password: '%s'\n", *conf.MpdAddress,
		*conf.MpdPassword)

	//collection := loadOrRefresh(*conf.MusicDir)
	music, err := model.RefreshMpd(&conf)
	if (err != nil) {
		log.Fatalln("Couldn't get files from mpd");
	}

	switch *action {
	case "stats":
		stats(music.Categories[0])
	case "search":
		search(music, *query)
	case "serve":
		serve(&conf, music)
	case "test-playback":
		testPlayback(&conf, music)
	case "refresh":
		os.Exit(0);
	default:
		fmt.Println("Unknown action", *action)
	}
}

func serve(conf *config.Config, music model.Collection) bool {
	categoryHandler := handlers.CategoryHandler{music}
	albumHandler := handlers.AlbumHandler{music}
	artistHandler := handlers.ArtistHandler{music}
	playlistHandler := handlers.NewPlaylistHandler(music, conf)
	controlHandler := handlers.NewControlHandler(conf)
	http.Handle("/categories/", categoryHandler)
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
	fmt.Printf("Serving from %s on %s\n", *conf.MusicDir, bind)
	http.ListenAndServe(bind, nil)
	return true
}

func stats(category *model.Category) {
	fmt.Printf("Stats: %d tracks, %d albums, %d artists\n",
		len(category.Tracks), len(category.Albums),
		len(category.Artists))
}

func search(music model.Collection, query string) {
	matching := model.Search(music, query)
	fmt.Printf("Matches for '%s':\n", query)
	stats(&matching)

	fmt.Println("\nMatching Tracks:")
	for _, track := range matching.Tracks {
		fmt.Printf("%-40s (%-20s)\n", track.Name, track.Artist)
	}

	fmt.Println("\n\nMatching Albums:")
	for _, album := range matching.Albums {
		fmt.Printf("%-40s (%s)\n", album.Name, album.Artist)
	}
}

func testPlayback(conf *config.Config, music model.Collection) error {
	playlist := player.NewMpdPlaylist(conf)
	playlist.Clear()

	// add any old album
	album := music.Categories[0].Artists[3].Albums[2]
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
