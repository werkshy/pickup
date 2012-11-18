package player

import (
	"code.google.com/p/gompd/mpd"
	"log"
	"strconv"
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
	Status() (status string, err error)
}

type PlayerStatus struct {
	State string
	Volume int
	CurrentArtist string
	CurrentAlbum string
	CurrentTrack string
	Elapsed float64
	Length float64
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

func (controls MpdControls) Status() (status PlayerStatus, err error) {
	// mpd status returns map[string] string
	attrs, err := controls.conn.Status()
	log.Printf("mpd 'status': %v\n", attrs)
	//var currentId = attrs["songid"]
	//var nextId = attrs["nextsongid"]
	status.Volume, err = strconv.Atoi(attrs["volume"])
	status.State = attrs["state"]
	if attrs["elapsed"] != ""  {
		status.Elapsed, err = strconv.ParseFloat(attrs["elapsed"], 64)
	}

	attrs, err = controls.conn.CurrentSong()
	log.Printf("mpd 'current song': %v\n", attrs)
	status.CurrentArtist = attrs["Artist"]
	status.CurrentAlbum = attrs["Album"]
	status.CurrentTrack = attrs["Title"]
	if attrs["Time"] != "" {
		status.Length, err = strconv.ParseFloat(attrs["Time"], 64)
	}


	return status, err
}
