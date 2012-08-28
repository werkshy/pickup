package handlers

import (
	"encoding/json"
	"net/http"
)

type errorMessage struct {
	Msg string
}

func writeError(w http.ResponseWriter, code int, msg string) {
	em := errorMessage{msg}
	j, _ := json.Marshal(em)
	w.WriteHeader(code)
	w.Write(j)
}
