import React, { Component } from "react";

class PlaylistTrack extends Component {
  render() {
    let posInt = Math.trunc(this.props.track.Pos) + 1;
    return (
      <li>
        <span className="track-number">{posInt}</span>
        <span className="track-title abbreviated">{this.props.track.Name}</span>
        <span className="track-artist abbreviated">
          {this.props.track.Artist}
        </span>
        <span className="track-album abbreviated">
          {this.props.track.Album}
        </span>
      </li>
    );
  }
}

class Playlist extends Component {
  constructor(props) {
    super(props);
    this.state = { isVisible: false, tracks: [] };

    // This binding is necessary to make `this` work in the callback
    this.toggleVisibility = this.toggleVisibility.bind(this);
  }

  componentDidMount() {
    this.timerID = setInterval(() => this.updateTracks(), 2000);
  }

  componentWillUnmount() {
    clearInterval(this.timerID);
  }

  updateTracks() {
    fetch("/api/playlist/")
      .then(response => response.json())
      .then(data => {
        this.setState({ tracks: data });
      });
  }

  toggleVisibility() {
    this.setState(state => ({
      isVisible: !state.isVisible
    }));
  }

  render() {
    return (
      <div
        id="playlist-wrapper"
        className={this.state.isVisible ? "open" : "none"}
      >
        <div className="top" onClick={this.toggleVisibility}>
          <span>Playlist</span>
          <div className="arrow up-arrow" />
        </div>
        <div
          id="playlist-slider"
          style={{ display: this.state.isVisible ? "block" : "none" }}
        >
          <div id="playlist">
            <ul id="playlist-tracks">
              <>
                {this.state.tracks.map(track => (
                  <PlaylistTrack key={track.Pos} track={track} />
                ))}
              </>
            </ul>
          </div>
        </div>
      </div>
    );
  }
}

export default Playlist;
