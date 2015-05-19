package model

type Collection struct {
	Categories []*Category
}

func (c *Collection) GetSummary() []CategorySummary {
	summaries := make([]CategorySummary, len(c.Categories))
	for i := 0; i < len(c.Categories); i++ {
		summaries[i] = c.Categories[i].GetSummary()
	}
	return summaries
}

func (c *Collection) AddCategory(category *Category) {
	c.Categories = append(c.Categories, category)
}

type Category struct {
	Name    string
	Artists []*Artist
	Albums  []*Album
	Tracks  []*Track
}

func NewCategory(name string) *Category {
	c := &Category{}
	c.Name = name
	return c
}

type CategorySummary struct {
	Name       string
	Artists    []ArtistSummary
	AlbumNames []string
}

func (c *Category) GetSummary() CategorySummary {
	artistSummaries := make([]ArtistSummary, len(c.Artists))
	for i := 0; i < len(c.Artists); i++ {
		artistSummaries[i] = c.Artists[i].GetSummary()
	}
	albumNames := make([]string, len(c.Albums))
	for i := 0; i < len(c.Albums); i++ {
		albumNames[i] = c.Albums[i].Name
	}
	return CategorySummary{c.Name, artistSummaries, albumNames}

}

type Item interface {
	GetName() string
	SubItems() []*Item
}

/**
 * Track struct
 */
type Track struct {
	Name   string
	Path   string
	Album  string
	Artist string
}

func (t Track) SubItems() []*Item {
	return nil
}

func (t Track) GetName() string {
	return t.Name
}

/**
 * Album struct
 */
type Album struct {
	Name     string
	Path     string
	Tracks   []*Track
	Artist   string
	Category string
}

type AlbumSummary struct {
	Name   string
	Artist string
	Tracks []string
}

func (a Album) SubItems() []*Track {
	return a.Tracks
}

/*
 * Convert Album to AlbumSummary
 */
func NewAlbumSummary(a *Album) AlbumSummary {
	trackNames := make([]string, len(a.Tracks))
	for i := 0; i < len(a.Tracks); i++ {
		trackNames[i] = a.Tracks[i].Name
	}
	return AlbumSummary{a.Name, a.Artist, trackNames}
}

func NewAlbum(name string) *Album {
	album := &Album{}
	album.Name = name
	return album
}

/**
 * Artist struct
 */
type Artist struct {
	Name   string
	Path   string
	Albums []*Album
}

func (a Artist) SubItems() []*Album {
	return a.Albums
}

func (a Artist) GetName() string {
	return a.Name
}

type ArtistSummary struct {
	Name       string
	AlbumNames []string
}

func NewArtist(name string) *Artist {
	artist := &Artist{}
	artist.Name = name
	return artist
}

/*
 * Convert Artist to ArtistSummary
 */
func (a *Artist) GetSummary() ArtistSummary {
	names := make([]string, len(a.Albums))
	for i := 0; i < len(a.Albums); i++ {
		names[i] = a.Albums[i].Name
	}
	return ArtistSummary{a.Name, names}
}
