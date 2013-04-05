package handlers

import (
	"encoding/json"
	"net/http"
	//"io/ioutil"
	"log"
	"pickup/model"
	"time"
)

type CategoryHandler struct {
	Music model.Collection
}

// Return a list of all artists, albums etc
func (h CategoryHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	log.Printf("Retrieving entire collection\n")
	t0 := time.Now()

	// Convert to Summary to save on data passing etc
	summary := h.Music.GetSummary()
	log.Printf("Marshalling %d category summaries", len(summary))
	j, _ := json.Marshal(summary)
	log.Println("Time to marshall entire collection:", time.Since(t0))
	t1 := time.Now()
	w.Write(j)
	log.Println("Time to send entire collection:", time.Since(t1))
}

