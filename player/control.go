package player

import (
	"code.google.com/p/gompd/mpd"
	"log"
)

// Define the interface for a player
type Controls interface {
	Play() error
	Stop() error
	Pause() error
	Prev() error
	Next() error
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
	return controls.conn.Play(0)
}

func (controls MpdControls) Stop() (err error) {
	log.Printf("mpd 'stop'\n")
	return controls.conn.Stop()
}

func (controls MpdControls) Pause() (err error) {
	log.Printf("mpd 'pause'\n")
	// TODO: get current pause state and toggle it
	return controls.conn.Pause(true)
}

func (controls MpdControls) Prev() (err error) {
	log.Printf("mpd 'prev'\n")
	return controls.conn.Previous()
}

func (controls MpdControls) Next() (err error) {
	log.Printf("mpd 'next'\n")
	return controls.conn.Next()
}
