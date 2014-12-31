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
	log.Printf("Retrieving entire collection\n")
	t0 := time.Now()
	music := <-h.MpdChannel

	// Convert to Summary to save on data passing etc
	summary := music.GetSummary()
	log.Printf("Marshalling %d category summaries", len(summary))
	j, _ := json.Marshal(summary)
	log.Println("Time to marshall entire collection:", time.Since(t0))
	t1 := time.Now()
	w.Write(j)
	log.Println("Time to send entire collection:", time.Since(t1))
}
