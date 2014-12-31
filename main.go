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
	"github.com/werkshy/pickup/model"
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

	mpdChannel := make(chan *model.Collection)
	go initializeMpd(mpdChannel, &conf)
	music := <-mpdChannel
	log.Printf("Player with %d categories initialized in %v\n", len(music.Categories), time.Since(t0))

	serve(&conf, mpdChannel)
}

func initializeMpd(mpdChannel chan *model.Collection, conf *config.Config) {
	updateInterval := 60 * time.Second
	// TODO: should be passing a pointer to a collection I think
	//       and then update the pointer if it is valid on refresh, then
	//       delete the memory
	music, err := model.RefreshMpd(conf)
	if err != nil {
		log.Fatalf("Couldn't get files from mpd: \n%s\n", err)
	}
	lastUpdated := time.Now()
	bkgCollectionChannel := make(chan model.Collection)
	for {
		select {
		case mpdChannel <- &music:
			continue
		case newMusic := <-bkgCollectionChannel:
			log.Printf("MPD GOROUTINE: UPDATE COMPLETE\n")
			music = newMusic
		case <-time.After(100 * time.Millisecond):
			since := time.Since(lastUpdated)
			if time.Since(lastUpdated) > updateInterval {
				log.Printf("MPD GOROUTINE: Kicking off refresh after %v\n", since)
				go backgroundRefresh(bkgCollectionChannel, conf)
				lastUpdated = time.Now()
			}
		}
	}

}

func backgroundRefresh(bkgCollectionChannel chan model.Collection, conf *config.Config) {
	music, err := model.RefreshMpd(conf)
	if err != nil {
		log.Printf("Couldn't get files from mpd: \n%s\n", err)
	}
	bkgCollectionChannel <- music
}

func serve(conf *config.Config, mpdChannel chan *model.Collection) bool {
	categoryHandler := handlers.CategoryHandler{mpdChannel}
	albumHandler := handlers.AlbumHandler{mpdChannel}
	artistHandler := handlers.ArtistHandler{mpdChannel}
	playlistHandler := handlers.PlaylistHandler{mpdChannel, conf}
	controlHandler := handlers.NewControlHandler(conf)

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
	return true
}
