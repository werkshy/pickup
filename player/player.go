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
	Command   string
	Category  string
	Artist    string
	Album     string
	Track     string
	Immediate bool
}

type ControlCommand struct {
	Command     string
	VolumeDelta int
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

	// Player control methods
	Status() (status PlayerStatus, err error)
	HandleControlCommand(cmd *ControlCommand) error
	HandlePlaylistCommand(cmd *PlaylistCommand) error

	// Cleanup
	Close() error
}
