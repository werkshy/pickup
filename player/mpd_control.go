package player

// Implement control methods for MpdPlayer

import (
	"log"
	"strconv"
)

func (player MpdPlayer) Play() (err error) {
	log.Printf("mpd 'play'\n")
	// Play(-1) implies play from current place
	return player.conn.Play(-1)
}

func (player MpdPlayer) Stop() (err error) {
	log.Printf("mpd 'stop'\n")
	return player.conn.Stop()
}

func (player MpdPlayer) Pause() (err error) {
	log.Printf("mpd 'pause'\n")
	// get current pause state and toggle it
	attrs, err := player.conn.Status()
	if err != nil {
		log.Println("Error trying to get mpd status")
		log.Println(err)
		return err
	}
	if attrs["state"] == "pause" {
		log.Printf("Resuming playback")
		return player.conn.Pause(false)
	} else if attrs["state"] == "play" {
		log.Printf("Pausing playback")
		return player.conn.Pause(true)
	}
	return nil
}

func (player MpdPlayer) Prev() (err error) {
	log.Printf("mpd 'prev'\n")
	return player.conn.Previous()
}

func (player MpdPlayer) Next() (err error) {
	log.Printf("mpd 'next'\n")
	return player.conn.Next()
}

func (player MpdPlayer) VolumeDelta(volumeDelta int) (err error) {
	log.Printf("mpd 'volumeDelta' %d\n", volumeDelta)
	attrs, err := player.conn.Status()
	if err != nil {
		log.Println("Error trying to get mpd status")
		log.Println(err)
		return err
	}
	volume, err := strconv.Atoi(attrs["volume"])
	if err == nil {
		log.Printf("mpd 'volumeDelta' %d + %d\n", volume, volumeDelta)
		err = player.conn.SetVolume(volume + volumeDelta)
	}
	return err
}

func (player *MpdPlayer) Status() (status PlayerStatus, err error) {
	conn, err := player.getConnection()
	if err != nil {
		return status, err
	}
	// mpd status returns map[string] string
	attrs, err := conn.Status()
	if err != nil {
		log.Println("Error trying to get mpd status")
		log.Println(err)
		player.Reconnect()
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

	attrs, err = conn.CurrentSong()
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

func (player *MpdPlayer) DoControlCommand(cmd ControlCommand) (err error) {
	// TODO
	return nil
}
