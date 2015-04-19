//usr/bin/env go run "$0" "$@"; exit
package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/werkshy/pickup/config"
	"github.com/werkshy/pickup/handlers"
	"github.com/werkshy/pickup/player"
	flag "launchpad.net/gnuflag"
)

func main() {
	t0 := time.Now()
	conf := config.Config{}
	conf.MusicDir = flag.String("music-dir", "/music", "Music dir")
	conf.Port = flag.Int("port", 8080, "Pickup port")
	conf.MpdAddress = flag.String("mpd-address", "localhost:6600", "MPD address")
	conf.MpdPassword = flag.String("mpd-password", "", "MPD Password")

	flag.Parse(true)

	log.Printf("Mpd address: '%s'  password: '%s'\n", *conf.MpdAddress,
		*conf.MpdPassword)

	plyr, err := player.NewMpdPlayer(&conf)
	if err != nil {
		log.Fatalln("Failed to initialize mpd player", err)
	}
	music, err := plyr.GetCollection()
	if err != nil {
		log.Fatalln("Failed to retrieve collection", err)
	}
	log.Printf("Player with %d categories initialized in %v\n", len(music.Categories), time.Since(t0))

	serve(&conf, &plyr)
}

func serve(conf *config.Config, plyr player.Player) {
	categoryHandler := handlers.CategoryHandler{plyr}
	albumHandler := handlers.AlbumHandler{plyr}
	artistHandler := handlers.ArtistHandler{plyr}
	playlistHandler := handlers.PlaylistHandler{plyr}
	controlHandler := handlers.ControlHandler{plyr}

	http.Handle("/categories/", categoryHandler)
	http.Handle("/albums/", albumHandler)
	http.Handle("/artists/", artistHandler)
	http.Handle("/playlist/", playlistHandler)
	http.Handle("/control/", controlHandler)

	staticDir, _ := os.Getwd()
	staticDir = staticDir + "/static"
	log.Printf("Serving static files from %s\n", staticDir)
	// strip '/static' from the url to get the name of the file within the
	// static dir.
	http.Handle("/static/", http.StripPrefix("/static/",
		http.FileServer(http.Dir(staticDir))))
	http.HandleFunc("/", handlers.Index)
	var bind = fmt.Sprintf(":%d", *conf.Port)
	log.Printf("Serving from %s on %s\n", *conf.MusicDir, bind)
	http.ListenAndServe(bind, nil)
}
