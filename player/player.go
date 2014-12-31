package player

import (
	"log"
	"time"

	"github.com/werkshy/gompd/mpd"
	"github.com/werkshy/pickup/config"
	"github.com/werkshy/pickup/model"
)

// TODO: merge interface of Playlist and Controls into this (or compose)
type Player interface {
	GetMusic()
	Close()
}

type MpdPlayer struct {
	conn       *mpd.Client
	conf       *config.Config
	mpdChannel chan *model.Collection
}

func NewMpdPlayer(conf *config.Config) (player MpdPlayer, err error) {
	conn, err := mpd.DialAuthenticated("tcp", *conf.MpdAddress,
		*conf.MpdPassword)
	if err != nil {
		log.Fatalln(err)
	}
	mpdChannel := make(chan *model.Collection)
	player = MpdPlayer{conn, conf, mpdChannel}
	go player.begin()
	return player, err
}

func (player MpdPlayer) Close() (err error) {
	return player.conn.Close()
}

func (player MpdPlayer) begin() {
	updateInterval := 60 * time.Second
	music, err := model.RefreshMpd(player.conf)
	if err != nil {
		log.Fatalf("Couldn't get files from mpd: \n%s\n", err)
	}
	lastUpdated := time.Now()
	bkgCollectionChannel := make(chan model.Collection)
	for {
		select {
		case player.mpdChannel <- &music:
			continue
		case newMusic := <-bkgCollectionChannel:
			log.Printf("MPD GOROUTINE: UPDATE COMPLETE\n")
			music = newMusic
		case <-time.After(100 * time.Millisecond):
			since := time.Since(lastUpdated)
			if time.Since(lastUpdated) > updateInterval {
				log.Printf("MPD GOROUTINE: Kicking off refresh after %v\n", since)
				go backgroundRefresh(bkgCollectionChannel, player.conf)
				lastUpdated = time.Now()
			}
		}
	}

}

func (player MpdPlayer) GetMusic() *model.Collection {
	return <-player.mpdChannel
}

func (player MpdPlayer) GetChannel() chan *model.Collection {
	return player.mpdChannel
}

func backgroundRefresh(bkgCollectionChannel chan model.Collection, conf *config.Config) {
	music, err := model.RefreshMpd(conf)
	if err != nil {
		log.Printf("Couldn't get files from mpd: \n%s\n", err)
	}
	bkgCollectionChannel <- music
}
