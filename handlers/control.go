package handlers

import (
	"encoding/json"
	"log"
	"net/http"
	"time"

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

// dispatch control commands (vol, prev, next)
func (h ControlHandler) command(w http.ResponseWriter, r *http.Request) (err error) {
	var cmd player.ControlCommand
	err = JsonRequestToType(w, r, &cmd)
	if err != nil {
		return err
	}
	log.Printf("Received control command '%s'\n", cmd.Command)
	err = h.HandleControlCommand(&cmd)

	return err
}
