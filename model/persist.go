package model

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"time"
)

var collectionFile string = "collection.json"

func (coll Collection) Save() error {
	t0 := time.Now()
	fmt.Printf("Saving collection: type = %T\n", coll)
	j, err := json.Marshal(coll)
	if err != nil {
		fmt.Printf("Error")
	}
	ioutil.WriteFile(collectionFile, j, 0600)
	fmt.Println("Saved collection in ", time.Since(t0))
	return nil
}

func Load() (Collection, error) {
	t0 := time.Now()
	// TODO: check that file exists
	var collection Collection
	fmt.Printf("Reading saved collection from '%s'\n", collectionFile)
	data, err := ioutil.ReadFile(collectionFile)
	if err != nil {
		fmt.Printf("Error loading collection")
		// TODO: delete file
		return collection, err
	}
	fmt.Println("Loaded collection in", time.Since(t0), len(data)/1024, "kB")
	t1 := time.Now()
	err = json.Unmarshal(data, &collection)
	if err != nil {
		fmt.Printf("Error unmarshalling collection")
		// TODO: delete file
	}
	fmt.Println("Parsed collection in ", time.Since(t1))
	return collection, err

}
