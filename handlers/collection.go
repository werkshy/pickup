package handlers

import (
	"encoding/json"
	"fmt"
	"net/http"
	//"io/ioutil"
	"log"
	"pickup/model"
	"time"
)

type CollectionHandler struct {
	Music model.Collection
}

// Return a list of all artists, albums etc
func (h CollectionHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	log.Printf("Retrieving entire collection\n")
	t0 := time.Now()

	// Convert to Artist Summary to save on info
	summary := h.Music.GetSummary()
	j, _ := json.Marshal(summary)
	fmt.Println("Time to marshall entire collection:", time.Since(t0))
	t1 := time.Now()
	w.Write(j)
	fmt.Println("Time to send entire collection:", time.Since(t1))
}

