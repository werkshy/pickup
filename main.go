//usr/bin/env go run "$0" "$@"; exit
package main

import (
	"embed"
	"fmt"
	"io/fs"
	"log"
	"net/http"
	"time"

	"github.com/werkshy/pickup/config"
	"github.com/werkshy/pickup/handlers"
	"github.com/werkshy/pickup/player"
	flag "github.com/juju/gnuflag"
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

// Embed the compiled frontend files
//go:embed react/dist
var embedded embed.FS

func serve(conf *config.Config, plyr player.Player) {
	categoryHandler := handlers.CategoryHandler{Player: plyr}
	albumHandler := handlers.AlbumHandler{Player: plyr}
	artistHandler := handlers.ArtistHandler{Player: plyr}
	playlistHandler := handlers.PlaylistHandler{Player: plyr}
	controlHandler := handlers.ControlHandler{Player: plyr}

	http.Handle("/categories/", categoryHandler)
	http.Handle("/albums/", albumHandler)
	http.Handle("/artists/", artistHandler)
	http.Handle("/playlist/", playlistHandler)
	http.Handle("/control/", controlHandler)

	// Repeat the above handlers for the React URLs, which are nested under '/api' to allow for proxying
	// through webpack-dev-server
	http.Handle("/api/categories/", categoryHandler)
	http.Handle("/api/albums/", albumHandler)
	http.Handle("/api/artists/", artistHandler)
	http.Handle("/api/playlist/", playlistHandler)
	http.Handle("/api/control/", controlHandler)

	// Serve static assets from at path /assets/ from dir react/dist/assets
	assetsDir, err := fs.Sub(embedded, "react/dist/assets")
	if err != nil {
		log.Fatal(err)
	}
	// strip '/static' from the url to get the name of the file within the static dir.
	http.Handle("/assets/", http.StripPrefix("/assets/",
		http.FileServer(http.FS(assetsDir))))

	// Serve webpack-built js files at path /react-static
	distDir, err := fs.Sub(embedded, "react/dist")
	// strip '/static' from the url to get the name of the file within the static dir.
	http.Handle("/static/", http.StripPrefix("/static/",
		http.FileServer(http.FS(distDir))))

	http.HandleFunc("/", index_handler)
	var bind = fmt.Sprintf(":%d", *conf.Port)
	log.Printf("Serving from %s on %s\n", *conf.MusicDir, bind)
	http.ListenAndServe(bind, nil)
}

func index_handler(w http.ResponseWriter, r *http.Request) {
	t0 := time.Now()
	index, _ := embedded.ReadFile("react/dist/index.html")
	w.Write(index)
	log.Printf("%-5s %-40s %v", r.Method, r.URL, time.Since(t0))
}
