import React, { Component } from 'react';


class PlaylistTrack extends Component {
  render() {
		let posInt = Math.trunc(this.props.track.Pos) + 1;
    return <li>{posInt} - {this.props.track.Artist} - {this.props.name}</li>;
  }
}

class Playlist extends Component {
  constructor(props) {
    super(props);
    this.state = {isVisible: false, tracks: []};

    // This binding is necessary to make `this` work in the callback
    this.toggleVisibility = this.toggleVisibility.bind(this);
  }

	componentDidMount() {
		this.timerID = setInterval(
			() => this.updateTracks(),
			2000
		);

	}

	componentWillUnmount() {
		clearInterval(this.timerID);
	}

  updateTracks() {
		fetch('/api/playlist/')
          .then(response => response.json())
          .then(data => {
					  console.log(data)
						this.setState({ tracks: data })
					});
	}

  toggleVisibility() {
    this.setState(state => ({
      isVisible: !state.isVisible
    }));
  }

  render() {
    return (
      <div id="playlist-wrapper" className={ this.state.isVisible ? 'open' : 'none'} >
        <div className="top" onClick={this.toggleVisibility}>Playlist</div>
        <div id="playlist-slider" style={{display: this.state.isVisible ? 'block' : 'none' }} >
          <div id="playlist">
            <ul id="playlist-tracks">
            <>
            { this.state.tracks.map(track => (
              <PlaylistTrack key={track.Pos} name={track.Name} track={track}/>
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
