package handlers

import (
	"encoding/json"
	"errors"
	"log"
	"net/http"
	//"io/ioutil"
	//"strings"
	//"time"
	"github.com/werkshy/pickup/config"
	"github.com/werkshy/pickup/model"
	"github.com/werkshy/pickup/player"
)

type PlaylistHandler struct {
	MpdChannel chan *model.Collection
	Conf       *config.Config
}

// Return a list of albums or a specific album
func (h PlaylistHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	defer controls.Close()
	playlist := player.NewMpdPlaylist(h.Conf)
	defer playlist.Close()

	switch r.Method {
	case "GET":
		log.Printf("GET: Showing current playlist\n")
		err = h.currentPlaylist(w, playlist)
	case "POST":
		err = h.command(w, r, playlist, controls)
	}

	if err != nil {
		log.Printf("Error detected in /playlist: %s", err)
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
}

func (h PlaylistHandler) currentPlaylist(w http.ResponseWriter,
	playlist player.Playlist) error {
	// get the contents of the playlist
	currentTracks, err := playlist.List()
	if err != nil {
		log.Printf("Error getting playlist: %s", err)
		return err
	}
	j, _ := json.Marshal(currentTracks)
	w.Write(j)
	return err
}

type PlaylistCommand struct {
	Command   string
	Category  string
	Artist    string
	Album     string
	Track     string
	Immediate bool
}

// dispatch playlist commands (add, clear etc)
func (h PlaylistHandler) command(w http.ResponseWriter, r *http.Request,
	playlist player.Playlist, controls player.Controls) (err error) {
	var data PlaylistCommand
	err = JsonRequestToType(w, r, &data)
	if err != nil {
		return err
	}

	log.Printf("Received playlist command '%s'\n", data.Command)
	switch data.Command {
	case "add":
		err = h.add(playlist, controls, data)
	case "clear":
		err = h.clear(playlist)
	}
	return err
}

func (h PlaylistHandler) add(playlist player.Playlist, controls player.Controls,
	data PlaylistCommand) (err error) {
	music := <-h.MpdChannel
	if data.Album == "" {
		log.Printf("Don't play artists (or nulls)\n")
		return errors.New("Playing artists is not implemented")
	}

	log.Printf("Trying to add %s/%s/%s/%s to playlist (%v)\n",
		data.Category, data.Artist, data.Album, data.Track, data.Immediate)

	var album *model.Album = nil
	var track *model.Track = nil
	if data.Track == "" {
		album, err = model.GetAlbum(music, data.Category, data.Artist,
			data.Album)
	} else {
		track, err = model.GetTrack(music, data.Category, data.Artist,
			data.Album, data.Track)
	}
	if err != nil {
		log.Printf("Album not found.")
		return err
	}

	if data.Immediate {
		err = playlist.Clear()
		if err != nil {
			log.Printf("Error clearing playlist")
			return err
		}
	}

	if track != nil {
		err = playlist.AddTrack(track)
	}
	if album != nil {
		err = playlist.AddAlbum(album)
	}
	if err != nil {
		log.Printf("Error adding album or track %s/%s", data.Album, data.Track)
		return err
	}
	if data.Immediate {
		err = controls.Play()
	}
	return err
}

func (h PlaylistHandler) clear(playlist player.Playlist) (err error) {
	return playlist.Clear()
}
