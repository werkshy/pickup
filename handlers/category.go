package handlers

import (
	"encoding/json"
	"net/http"
	//"io/ioutil"
	"log"
	"time"

	"github.com/werkshy/pickup/model"
)

type CategoryHandler struct {
	MpdChannel chan *model.Collection
}

// Return a list of all artists, albums etc
//func (h CategoryHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
func (h CategoryHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	t0 := time.Now()
	music := <-h.MpdChannel

	// Convert to Summary to save on data passing etc
	summary := music.GetSummary()
	j, _ := json.Marshal(summary)
	w.Write(j)
	log.Printf("%-5s %-40s %v", r.Method, r.URL, time.Since(t0))
}
