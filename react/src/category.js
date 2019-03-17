import React, { Component } from 'react';

class CategoryArtist extends Component {
  render() {
    return (
      <li>
        <span className="artist-title" data-artist="{this.props.artist.Name}}" data-category="">
        {this.props.artist.Name}
        </span>
      </li>
    )
  }
}

class CategoryAlbum extends Component {
  render() {
    return (
      <li id="{this.props.name}">
        <span className="album-title abbreviated">{this.props.name}</span>
        <div className='album-actions add' title="Add '{this.props.name}'"></div>
        <div className='album-actions play' title="Play '{this.props.name}'"></div>
      </li>
    )
  }
}

class Category extends Component {
  constructor(props) {
    super(props);
    console.log(props);
    this.name = this.props.match.params.name;
    this.state = {Artists: [], AlbumNames: []}
  }

	componentDidMount() {
    this.getData();

	}

	componentWillUnmount() {
	}

	componentDidUpdate(prevProps) {
		let oldName = prevProps.match.params.name;
		let newName = this.props.match.params.name;
		if (newName !== oldName) {
			this.getData()
		}
		this.name = this.props.match.params.name;
	}

	// FIXME: make an endpoint that returns just one category
  getData() {
		fetch('/api/categories/')
          .then(response => response.json())
          .then(data => {
            let category = data.find(category => category.Name === this.name)
					  console.log("category", category)
						this.setState(category)
					});
	}

  render() {
    let albums;
    if (this.state.AlbumNames.length) {
      albums = (
        <>
        <h3>Albums</h3>
        <ul id="albumList">
            { this.state.AlbumsNames.map(album => (
              <CategoryAlbum name={album} />
            ))}
        </ul>
        </>
      )
    }

    return (
      <div>
				<h1>Category: {this.state.Name}</h1>
		    <div className="category multicolumn" data-category="{this.state.Name}">
        {albums}
        <h3>Artists</h3>
        <ul id="artistList">
              { this.state.Artists.map(artist => (
                <CategoryArtist artist={artist} key={artist.Name}/>
              ))}
        </ul>
      </div>
    </div>
    )
  }
}

export default Category;
