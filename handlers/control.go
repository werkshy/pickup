package handlers

import (
	"encoding/json"
	"errors"
	"log"
	"net/http"
	"time"
	//"io/ioutil"
	//"strings"
	//"time"

	"github.com/werkshy/pickup/player"
)

type ControlHandler struct {
	player.Player
}

// Return a list of albums or a specific album
func (h ControlHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	t0 := time.Now()
	var err error
	switch r.Method {
	case "GET":
		err = h.currentStatus(w)
	case "POST":
		err = h.command(w, r)
	}

	if err != nil {
		log.Printf("Error detected in /control: %s\n", err)
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
	log.Printf("%-5s %-40s %v", r.Method, r.URL, time.Since(t0))
}

func (h ControlHandler) currentStatus(w http.ResponseWriter) (err error) {

	// get the status
	status, err := h.Status()
	if err != nil {
		return err
	}
	j, _ := json.Marshal(status)
	w.Write(j)
	return err
}

type ControlCommand struct {
	Command     string
	VolumeDelta int
}

// dispatch control commands (vol, prev, next)
func (h ControlHandler) command(w http.ResponseWriter, r *http.Request) (err error) {
	var data ControlCommand
	err = JsonRequestToType(w, r, &data)
	if err != nil {
		return err
	}

	// TODO move this into Player
	log.Printf("Received control command '%s'\n", data.Command)
	switch data.Command {
	case "prev":
		err = h.Prev()
	case "next":
		err = h.Next()
	case "stop":
		err = h.Stop()
	case "play":
		err = h.Play()
	case "pause":
		err = h.Pause()
	case "volumeDelta":
		err = h.VolumeDelta(data.VolumeDelta)
	default:
		log.Printf("Unknown command: %s\n", data.Command)
		err = errors.New("Unknown command " + data.Command)
	}
	return err
}
