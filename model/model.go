package model

type Collection struct {
	Artists []Artist
	Albums []Album
	Tracks []Track
}


type Item interface {
	GetName() string
	SubItems() []Item
}

/**
 * Track struct
 */
type Track struct {
	Name string
	Path string
	Album string
	Artist string
}

func (t Track) SubItems() []Item {
	return nil
}

func (t Track) GetName() string {
	return t.Name
}


/**
 * Album struct
 */
type Album struct {
	Name string
	Path string
	Tracks []Track
	Artist string
}

type AlbumSummary interface {
	GetName() string
	GetArtist() string
}

func (a Album) SubItems() []Track {
	return a.Tracks
}

func (a Album) GetName() string {
	return a.Name
}

func (a Album) GetArtist() string {
	return a.Artist
}

/**
 * Artist struct
 */
type Artist struct {
	Name string
	Path string
	Albums []Album
}

func (a Artist) SubItems() []Album {
	return a.Albums
}

func (a Artist) getName() string {
	return a.Name
}

