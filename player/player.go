package player

// Define an interface for a goroutine-separated player

import "github.com/werkshy/pickup/model"

// What we return to describe the current playlist
type PlaylistTrack struct {
	Pos     string
	Name    string
	Artist  string
	Album   string
	Path    string
	Backend string
}

// What we return when we ask for current status
type PlayerStatus struct {
	State         string
	Volume        int
	CurrentArtist string
	CurrentAlbum  string
	CurrentTrack  string
	Elapsed       int
	Length        int
}

type PlaylistCommand struct {
}

type ControlCommand struct {
}

// In theory we could have different backends, so define an interface that will
// allow for that.
type Player interface {
	GetCollection() (*model.Collection, error)
	RefreshCollection() (model.Collection, error)

	// Playlist methods
	List() ([]PlaylistTrack, error)
	AddAlbum(*model.Album) error
	AddTrack(*model.Track) error
	AddTracks([]*model.Track) error
	Clear() error
	DoPlaylistCommand(cmd PlaylistCommand) error

	// Player control methods
	Play() error
	Stop() error
	Pause() error
	Prev() error
	Next() error
	VolumeDelta(volumeDelta int) error
	//	VolumeDown() error
	//	VolumeUp() error
	//	GetVolume() (int, error)
	Status() (status PlayerStatus, err error)
	DoControlCommand(cmd ControlCommand) error

	// Cleanup
	Close() error
}
