package handlers

import (
	"encoding/json"
	"net/http"
)

func JsonRequestToData(w http.ResponseWriter, r *http.Request) (
	data map[string]interface{}, err error) {
	// unmarshall to blank interface
	var idata interface{}
	err = json.NewDecoder(r.Body).Decode(&idata)
	// type assertion to map
	data = idata.(map[string]interface{})
	return data, err
}

func JsonRequestToType(w http.ResponseWriter, r *http.Request,
	data interface{}) (err error) {
	return json.NewDecoder(r.Body).Decode(&data)
}
