import React, { Component } from 'react';


class PlaylistTrack extends Component {
  render() {
    return <li key={this.props.name}>{this.props.name}</li>;
  }
}

class Playlist extends Component {
  constructor(props) {
    super(props);
    this.state = {isVisible: false};

    // This binding is necessary to make `this` work in the callback
    this.toggleVisibility = this.toggleVisibility.bind(this);
  }

  toggleVisibility() {
    this.setState(state => ({
      isVisible: !state.isVisible
    }));
    console.log("Toggled playlist visibility: " + this.state.isVisible)
  }

  render() {
    return (
      <div id="playlist-wrapper" className={ this.state.isVisible ? 'open' : 'none'} >
        <div className="top" onClick={this.toggleVisibility}>Playlist</div>
        <div id="playlist-slider" style={{display: this.state.isVisible ? 'block' : 'none' }} >
          <div id="playlist">
            <ul id="playlist-tracks">
            <>
            { this.props.tracks.map(track => (
              <PlaylistTrack name={track} />
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
