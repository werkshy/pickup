package handlers

import (
	"io/ioutil"
	"log"
	"net/http"
	"time"
)

func Index(w http.ResponseWriter, r *http.Request) {
	t0 := time.Now()
	index, _ := ioutil.ReadFile("react/dist/index.html")
	w.Write(index)
	log.Printf("%-5s %-40s %v", r.Method, r.URL, time.Since(t0))
}
