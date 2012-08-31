package handlers

import (
	"fmt"
	"net/http"
	"io/ioutil"
)

func Index(w http.ResponseWriter, req *http.Request) {
	fmt.Printf("Serving index file to %s\n", req.RemoteAddr)
	index, _ := ioutil.ReadFile("static/index.html")
	w.Write(index)
}

