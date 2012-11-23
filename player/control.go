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
	VolumeDelta(volumeDelta int) error
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
	addr string
	password string
}

func NewMpdControls(addr, password string) (controls MpdControls,
			err error) {
	log.Printf("Creating MpdControls instance (%s / %s)\n", addr, password)
	controls = MpdControls { nil, addr, password }
	err = controls.connect()
	controls.conn.Ping()
	log.Println("Pinged OK")
	controls.Status()
	return controls, err
}

func (controls *MpdControls) connect() (err error) {
	log.Println("Connecting to mpd")
	controls.conn, err = mpd.DialAuthenticated("tcp", controls.addr,
			controls.password)
	if err != nil {
		log.Println("Error trying to get MPD client")
		log.Println(err)
	}
	return err
}

func (controls MpdControls) Close() (err error) {
	log.Println("Closing mpd connection (controls)")
	return controls.conn.Close()
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

func (controls *MpdControls) Status() (status PlayerStatus, err error) {
	// mpd status returns map[string] string
	attrs, err := controls.conn.Status()
	if err != nil {
		log.Println("Error trying to get mpd status")
		log.Println(err)
		controls.connect()
		return status, err
	}
	log.Printf("mpd 'status': %v\n", attrs)
	//var currentId = attrs["songid"]
	//var nextId = attrs["nextsongid"]
	status.Volume, err = strconv.Atoi(attrs["volume"])
	status.State = attrs["state"]
	if attrs["elapsed"] != ""  {
		status.Elapsed, err = strconv.ParseFloat(attrs["elapsed"], 64)
	}

	attrs, err = controls.conn.CurrentSong()
	if err != nil {
		log.Println("Error trying to get mpd current song")
		log.Println(err)
		return status, err
	}
	log.Printf("mpd 'current song': %v\n", attrs)
	status.CurrentArtist = attrs["Artist"]
	status.CurrentAlbum = attrs["Album"]
	status.CurrentTrack = attrs["Title"]
	if attrs["Time"] != "" {
		status.Length, err = strconv.ParseFloat(attrs["Time"], 64)
	}


	return status, err
}
