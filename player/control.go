package player

import (
	"code.google.com/p/gompd/mpd"
	"log"
)

// Define the interface for a player
type Controls interface {
	Play() error
//	Stop() error
//	Volume(newVol int) error
//	VolumeDown() error
//	VolumeUp() error
//	GetVolume() (int, error)
}

// Implementation of player interface via mpd
type MpdControls struct {
	conn *mpd.Client
}

func NewMpdControls() MpdControls {
	conn, err := mpd.Dial("tcp", "localhost:6600")
	if err != nil {
		log.Fatalln(err)
	}
	return MpdControls { conn }
}

func (controls MpdControls) Play() (err error) {
	log.Printf("mpd 'play'\n")
	err = controls.conn.Play(0)
	return err
}
