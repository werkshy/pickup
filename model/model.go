package model

type Collection struct {
	MusicDir string
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

type AlbumSummary struct {
	Name string
	Artist string
	Tracks []string
}

func (a Album) SubItems() []Track {
	return a.Tracks
}

/*
 * Convert Album to AlbumSummary
 */
func NewAlbumSummary(a Album) AlbumSummary {
	trackNames := make([]string, len(a.Tracks))
	for i := 0; i< len(a.Tracks); i++ {
		trackNames[i] = a.Tracks[i].Name
	}
	return AlbumSummary{a.Name, a.Artist, trackNames}
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

func (a Artist) GetName() string {
	return a.Name
}

type ArtistSummary struct {
	Name string
	AlbumNames []string
}

/*
 * Convert Artist to ArtistSummary
 */
func NewArtistSummary(a Artist) ArtistSummary {
	names := make([]string, len(a.Albums))
	for i := 0; i< len(a.Albums); i++ {
		names[i] = a.Albums[i].Name
	}
	return ArtistSummary{a.Name, names}
}


