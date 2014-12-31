package player

import (
	"log"
	"time"

	"github.com/werkshy/gompd/mpd"
	"github.com/werkshy/pickup/config"
	"github.com/werkshy/pickup/model"
)

// TODO: merge interface of Playlist andMpdControls into this (or compose)
type Player interface {
	GetMusic() *model.Collection
	GetControls() Controls
	GetPlaylist() Playlist
	Close() error
}

/*
TODO:
	GetPlaylist()
	GetStatus()
	GetMusic() -> GetCollection()
	PlaylistCommand()
	ControlCommand()
*/

type MpdPlayer struct {
	conn              *mpd.Client
	conf              *config.Config
	collectionChannel chan *model.Collection
	controlsChannel   chan *MpdControls
	playlistChannel   chan *MpdPlaylist
}

func NewMpdPlayer(conf *config.Config) (player MpdPlayer, err error) {
	conn, err := mpd.DialAuthenticated("tcp", *conf.MpdAddress,
		*conf.MpdPassword)
	if err != nil {
		log.Fatalln(err)
	}
	collectionChannel := make(chan *model.Collection)
	controlsChannel := make(chan *MpdControls)
	playlistChannel := make(chan *MpdPlaylist)
	player = MpdPlayer{conn, conf, collectionChannel, controlsChannel, playlistChannel}
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
	controls, err := NewMpdControls(player.conf)
	if err != nil {
		log.Fatalf("Couldn't initialize  from mpd: \n%s\n", err)
	}
	playlist := NewMpdPlaylist(player.conf)
	lastUpdated := time.Now()
	bkgCollectionChannel := make(chan model.Collection)
	for {
		select {
		case player.collectionChannel <- &music:
			continue
		// TODO this'll go away, replace with CommandChannel etc
		case player.controlsChannel <- &controls:
			continue
		case player.playlistChannel <- &playlist:
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
	return <-player.collectionChannel
}

func (player MpdPlayer) GetControls() Controls {
	return <-player.controlsChannel
}

func (player MpdPlayer) GetPlaylist() Playlist {
	return <-player.playlistChannel
}

func backgroundRefresh(bkgCollectionChannel chan model.Collection, conf *config.Config) {
	music, err := model.RefreshMpd(conf)
	if err != nil {
		log.Printf("Couldn't get files from mpd: \n%s\n", err)
	}
	bkgCollectionChannel <- music
}
