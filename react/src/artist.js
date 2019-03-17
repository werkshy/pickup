import React, { Component } from 'react';

import { playAlbum } from './actions'

class ArtistAlbum extends Component {
  constructor(props) {
    super(props);

    this.add = this.add.bind(this)
    this.play = this.play.bind(this)
  }

  add() {
    playAlbum(this.props.category, this.props.artist, this.props.name, false)
  }

  play() {
    playAlbum(this.props.category, this.props.artist, this.props.name, true)
  }

  render() {
    return (
      <li id={this.props.name}>
        <span className="album-title abbreviated">{this.props.name}</span>
        <div className='album-actions add' title="Add {this.props.name}" onClick={this.add}></div>
        <div className='album-actions play' title="Play {this.props.name}" onClick={this.play}></div>
      </li>
    )
  }
}

class Artist extends Component {
  constructor(props) {
    super(props);
    this.state = {Name: "", AlbumNames: [], TrackNames: []}
  }

  category() {
    return this.props.match.params.category;
  }

  artist() {
    return this.props.match.params.artist;
  }

	componentDidMount() {
    this.getData();

	}

	componentWillUnmount() {
	}

	componentDidUpdate(prevProps) {
		let oldPath = prevProps.match.params.category + prevProps.match.params.artist;
		let newPath = this.props.match.params.category + this.props.match.params.artist;
		if (newPath !== oldPath) {
			this.getData()
		}
	}

	// FIXME: make (or use) an endpoint that returns just one artist
  // e.g. /api/artist/:category/:artist
  // or, perhaps we could assign a unique id to each artist, album and maybe even track
  getData() {
		fetch('/api/categories/')
          .then(response => response.json())
          .then(data => {
            let category = data.find(category => category.Name === this.category())
            let artist = category.Artists.find(artist => artist.Name === this.artist())
					  console.log("artist", artist)
						this.setState(artist)
          });
	}


  render() {
    console.log("state", this.state)
    return (
      <div id="artistView" className="singlecolumn">
        <h2>{this.state.Name}</h2>
        <ul id="albumList">
            { this.state.AlbumNames.map(album => (
              <ArtistAlbum key={album} category={this.category()} artist={this.artist()} name={album} />
            ))}
        </ul>
      </div>
    )
  }
}

export default Artist

