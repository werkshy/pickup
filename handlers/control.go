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
	controls := player.NewMpdControls()

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
			r *http.Request) (err error) {
	var data ControlCommand
	controls := player.NewMpdControls()
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
		default:
			log.Printf("Unknown command: %s\n", data.Command)
			err = errors.New("Unknown command " + data.Command)
	}
	return err
}

