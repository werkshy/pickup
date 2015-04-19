package player

import (
	"log"
	"time"

	"github.com/werkshy/gompd/mpd"
	"github.com/werkshy/pickup/config"
	"github.com/werkshy/pickup/model"
)

// Implement Player interface with MPD backend
type MpdPlayer struct {
	conn              *mpd.Client
	conf              *config.Config
	collectionChannel chan *model.Collection
	controlChannel    chan *ControlCommand
	playlistChannel   chan *PlaylistCommand
}

func NewMpdPlayer(conf *config.Config) (player MpdPlayer, err error) {
	conn, err := mpd.DialAuthenticated("tcp", *conf.MpdAddress,
		*conf.MpdPassword)
	if err != nil {
		log.Fatalln(err)
	}
	collectionChannel := make(chan *model.Collection)
	controlChannel := make(chan *ControlCommand)
	playlistChannel := make(chan *PlaylistCommand)
	player = MpdPlayer{conn, conf, collectionChannel, controlChannel, playlistChannel}
	go player.begin()
	return player, err
}

// Return an open connection to MPD if possible, else return nil
func (player *MpdPlayer) getConnection() (conn *mpd.Client, err error) {
	if player.conn == nil {
		err = player.Reconnect()
	}
	return player.conn, err
}

func (player *MpdPlayer) Reconnect() (err error) {
	log.Println("Attempting reconnect to mpd")
	if player.conn != nil {
		log.Println("Closing old connection")
		player.conn.Close()
		player.conn = nil
	}
	conn, err := mpd.DialAuthenticated("tcp", *player.conf.MpdAddress,
		*player.conf.MpdPassword)
	if err != nil {
		log.Println("Error trying to reconnect")
		log.Println(err)
		player.conn = nil
	} else {
		log.Println("Successful reconnect")
		player.conn = conn
	}
	return err
}

func (player *MpdPlayer) Close() (err error) {
	return player.conn.Close()
}

func (player *MpdPlayer) begin() {
	updateInterval := 60 * time.Second
	music, err := player.RefreshCollection()

	if err != nil {
		log.Fatalf("Couldn't get files from mpd: \n%s\n", err)
	}
	lastUpdated := time.Now()
	bkgCollectionChannel := make(chan model.Collection)
	for {
		select {
		case player.collectionChannel <- &music:
			continue
		case controlCommand := <-player.controlChannel:
			log.Printf("Received control command %v\n", controlCommand)
			// TODO
			continue
		case playlistCommand := <-player.playlistChannel:
			log.Printf("Received playlist command %v\n", playlistCommand)
			// TODO
			continue
		case newMusic := <-bkgCollectionChannel:
			log.Printf("PLAYER GOROUTINE: UPDATE COMPLETE\n")
			music = newMusic
		case <-time.After(100 * time.Millisecond):
			since := time.Since(lastUpdated)
			if time.Since(lastUpdated) > updateInterval {
				log.Printf("PLAYER GOROUTINE: Kicking off refresh after %v\n", since)
				go backgroundRefresh(bkgCollectionChannel, player)
				lastUpdated = time.Now()
			}
		}
	}
}

// these can all be defined on Player
func (player *MpdPlayer) GetCollection() (collection *model.Collection, err error) {
	return <-player.collectionChannel, nil
}

func backgroundRefresh(bkgCollectionChannel chan model.Collection, player *MpdPlayer) {
	music, err := player.RefreshCollection()
	if err != nil {
		log.Printf("Couldn't get files from mpd: \n%s\n", err)
	}
	bkgCollectionChannel <- music
}
