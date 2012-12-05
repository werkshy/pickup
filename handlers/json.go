package handlers

import (
	"encoding/json"
	"io/ioutil"
	"net/http"
)

func JsonRequestToData(w http.ResponseWriter, r *http.Request) (
	data map[string]interface{}, err error) {
	body, err := ioutil.ReadAll(r.Body)
	if err != nil {
		return data, err
	}
	// unmarshall to blank interface
	var idata interface{}
	err = json.Unmarshal(body, &idata)
	// type assertion to map
	data = idata.(map[string]interface{})
	return data, err
}

func JsonRequestToType(w http.ResponseWriter, r *http.Request,
	data interface{}) (err error) {
	body, err := ioutil.ReadAll(r.Body)
	if err != nil {
		return err
	}
	err = json.Unmarshal(body, &data)
	return err
}
