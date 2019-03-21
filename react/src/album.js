import React, { Component } from 'react';

import { playAlbum, playTrack } from './actions'

class AlbumTrack extends Component {
  constructor(props) {
    super(props);

    this.add = this.add.bind(this)
    this.play = this.play.bind(this)
  }

  add() {
    playTrack(this.props.category, this.props.artist, this.props.album, this.props.track, false)
  }

  play() {
    playTrack(this.props.category, this.props.artist, this.props.album, this.props.track, true)
  }

  // FIXME the titles aren't interpolating
  render() {
    return (
      <li id="{{.}}">
        <span className="track-title abbreviated">{this.props.track}</span>
        <div className='track-actions add' title="Add {this.props.track}" onClick={this.add}></div>
        <div className='track-actions play' title="Play {this.props.track}" onClick={this.play}>&nbsp;</div>
      </li>
    )
  }
}

class Album extends Component {
  constructor(props) {
    super(props);
    this.state = { category: this.category(), artist: this.artist(), album: this.album(), Tracks: [] }

    this.add = this.add.bind(this)
    this.play = this.play.bind(this)
  }

  category() {
    return this.props.match.params.category;
  }

  artist() {
    return this.props.match.params.artist;
  }

  album() {
    return this.props.match.params.album;
  }

	componentDidMount() {
    this.getData();

	}

	componentWillUnmount() {
	}

	componentDidUpdate(prevProps) {
		let oldPath = prevProps.match.params.category + "/" + prevProps.match.params.artist + "/" + prevProps.match.params.album
		let newPath = this.props.match.params.category + "/" + this.props.match.params.artist + "/" + prevProps.match.params.album
		if (newPath !== oldPath) {
			this.getData()
		}
	}

	// FIXME: make (or use) an endpoint that returns just one artist
  // e.g. /api/artist/:category/:artist
  // or, perhaps we could assign a unique id to each artist, album and maybe even track
  getData() {
		let url = "/api/albums/" + this.category() + "/" + this.artist() + "/" + this.album()
		fetch(url)
          .then(response => response.json())
          .then(data => {
            console.log("album data", data)
						this.setState(data)
          });
	}

  add() {
    playAlbum(this.category(), this.artist(), this.album(), false)
  }

  play() {
    playAlbum(this.category(), this.artist(), this.album(), true)
  }


  render() {
    console.log("state", this.state)
    return (
      <div id="albumView" className="singlecolumn">
        <h2 className="artist-name">{this.artist()} - {this.album()}</h2>
        <h2><span>{this.album()}</span>
          <span className='album-actions add' onClick={this.add}></span>
          <span className='album-actions play' onClick={this.play}></span>
        </h2>
        <ul id="trackList">
            { this.state.Tracks.map(track => (
              <AlbumTrack key={track} category={this.category()} artist={this.artist()} album={this.album()}
                track={track} />
            ))}
        </ul>
      </div>
    )
  }
}

export default Album

