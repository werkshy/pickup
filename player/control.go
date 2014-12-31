package player

import (
	"log"
	"strconv"

	"github.com/werkshy/gompd/mpd"
	"github.com/werkshy/pickup/config"
)

type PlayerStatus struct {
	State         string
	Volume        int
	CurrentArtist string
	CurrentAlbum  string
	CurrentTrack  string
	Elapsed       int
	Length        int
}

// Define the interface for a player
type Controls interface {
	Play() error
	Stop() error
	Pause() error
	Prev() error
	Next() error
	VolumeDelta(volumeDelta int) error
	//	VolumeDown() error
	//	VolumeUp() error
	//	GetVolume() (int, error)
	Status() (status PlayerStatus, err error)
}

// Implementation of player interface via mpd
type MpdControls struct {
	conn *mpd.Client
}

func NewMpdControls(conf *config.Config) (controls MpdControls,
	err error) {
	conn, err := mpd.DialAuthenticated("tcp", *conf.MpdAddress,
		*conf.MpdPassword)
	if err != nil {
		log.Println("Error trying to get MPD client")
		log.Println(err)
	}
	controls = MpdControls{conn}
	return controls, err
}

func (controls MpdControls) Close() (err error) {
	return controls.conn.Close()
}

func (controls MpdControls) Play() (err error) {
	log.Printf("mpd 'play'\n")
	// Play(-1) implies play from current place
	return controls.conn.Play(-1)
}

func (controls MpdControls) Stop() (err error) {
	log.Printf("mpd 'stop'\n")
	return controls.conn.Stop()
}

func (controls MpdControls) Pause() (err error) {
	log.Printf("mpd 'pause'\n")
	// get current pause state and toggle it
	attrs, err := controls.conn.Status()
	if err != nil {
		log.Println("Error trying to get mpd status")
		log.Println(err)
		return err
	}
	if attrs["state"] == "pause" {
		log.Printf("Resuming playback")
		return controls.conn.Pause(false)
	} else if attrs["state"] == "play" {
		log.Printf("Pausing playback")
		return controls.conn.Pause(true)
	}
	return nil
}

func (controls MpdControls) Prev() (err error) {
	log.Printf("mpd 'prev'\n")
	return controls.conn.Previous()
}

func (controls MpdControls) Next() (err error) {
	log.Printf("mpd 'next'\n")
	return controls.conn.Next()
}

func (controls MpdControls) VolumeDelta(volumeDelta int) (err error) {
	log.Printf("mpd 'volumeDelta' %d\n", volumeDelta)
	attrs, err := controls.conn.Status()
	if err != nil {
		log.Println("Error trying to get mpd status")
		log.Println(err)
		return err
	}
	volume, err := strconv.Atoi(attrs["volume"])
	if err == nil {
		log.Printf("mpd 'volumeDelta' %d + %d\n", volume, volumeDelta)
		err = controls.conn.SetVolume(volume + volumeDelta)
	}
	return err
}

func (controls MpdControls) Status() (status PlayerStatus, err error) {
	// mpd status returns map[string] string
	attrs, err := controls.conn.Status()
	if err != nil {
		log.Println("Error trying to get mpd status")
		log.Println(err)
		return status, err
	}
	//log.Printf("mpd 'status': %v\n", attrs)
	//var currentId = attrs["songid"]
	//var nextId = attrs["nextsongid"]
	status.Volume, err = strconv.Atoi(attrs["volume"])
	status.State = attrs["state"]
	if attrs["elapsed"] != "" {
		elapsed, _ := strconv.ParseFloat(attrs["elapsed"], 64)
		status.Elapsed = int(elapsed)
	}

	attrs, err = controls.conn.CurrentSong()
	if err != nil {
		log.Println("Error trying to get mpd current song")
		log.Println(err)
		return status, err
	}
	status.CurrentArtist = attrs["Artist"]
	status.CurrentAlbum = attrs["Album"]
	status.CurrentTrack = attrs["Title"]
	if attrs["Time"] != "" {
		length, _ := strconv.ParseFloat(attrs["Time"], 64)
		status.Length = int(length)
	}

	return status, err
}
