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
)



type ControlHandler struct {
	controls player.MpdControls
}

func NewControlHandler(addr, password string) (h ControlHandler, err error) {
	controls, err := player.NewMpdControls(addr, password)
	return ControlHandler{controls}, err
}

// Return a list of albums or a specific album
func (h ControlHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	var err error = nil
	switch (r.Method) {
	case "GET":
		log.Printf("GET: return current status\n")
		err = h.currentStatus(w)
	case "POST":
		err = h.command(w, r)
	}

	if (err != nil) {
		log.Printf("Error detected in /control: %s\n", err)
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
}

func (h ControlHandler) currentStatus(w http.ResponseWriter) (err error) {
	/*
	controls, err := player.NewMpdControls()
	if (err != nil) {
		return err
	}
	defer controls.Close()
	*/

	// get the status
	status, err := h.controls.Status()
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
			r *http.Request) (err error) {
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
			err = h.controls.Prev()
		case "next":
			err = h.controls.Next()
		case "stop":
			err = h.controls.Stop()
		case "play":
			err = h.controls.Play()
		case "pause":
			err = h.controls.Pause()
		case "volumeDelta":
			err = h.controls.VolumeDelta(data.VolumeDelta)
		default:
			log.Printf("Unknown command: %s\n", data.Command)
			err = errors.New("Unknown command " + data.Command)
	}
	return err
}

