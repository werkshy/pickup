package handlers

import (
	"encoding/json"
	"errors"
	"log"
	"net/http"
	//"io/ioutil"
	//"strings"
	//"time"
	"pickup/player"
	"pickup/config"
)



type ControlHandler struct {
	conf *config.Config
}

func NewControlHandler(conf *config.Config) (h ControlHandler) {
	return ControlHandler{conf}
}

// Return a list of albums or a specific album
func (h ControlHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	controls, err := player.NewMpdControls(h.conf)
	defer controls.Close()
	switch (r.Method) {
	case "GET":
		log.Printf("GET: return current status\n")
		err = h.currentStatus(w, controls)
	case "POST":
		err = h.command(w, r, controls)
	}

	if (err != nil) {
		log.Printf("Error detected in /control: %s\n", err)
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
}

func (h ControlHandler) currentStatus(w http.ResponseWriter,
			controls player.Controls) (err error) {
	/*
	controls, err := player.NewMpdControls()
	if (err != nil) {
		return err
	}
	defer controls.Close()
	*/

	// get the status
	status, err := controls.Status()
	if (err != nil) {
		return err
	}
	j, _ := json.Marshal(status)
	w.Write(j)
	return err
}

type ControlCommand struct {
	Command string
	VolumeDelta int
}

// dispatch control commands (vol, prev, next)
func (h ControlHandler) command(w http.ResponseWriter,
			r *http.Request, controls player.Controls) (err error) {
	var data ControlCommand
	/*
	controls, err := player.NewMpdControls()
	if (err != nil) {
		return err
	}
	defer controls.Close()
	*/
	err = JsonRequestToType(w, r, &data)
	if (err != nil) {
		return err
	}

	log.Printf("Received control command '%s'\n", data.Command)
	switch(data.Command) {
		case "prev":
			err = controls.Prev()
		case "next":
			err = controls.Next()
		case "stop":
			err = controls.Stop()
		case "play":
			err = controls.Play()
		case "pause":
			err = controls.Pause()
		case "volumeDelta":
			err = controls.VolumeDelta(data.VolumeDelta)
		default:
			log.Printf("Unknown command: %s\n", data.Command)
			err = errors.New("Unknown command " + data.Command)
	}
	return err
}

