import React, { Component } from "react";

let initialData = {
  State: "stop",
  Volume: 100,
  CurrentArtist: "...",
  CurrentAlbum: "...",
  CurrentTrack: "...",
  Elapsed: 0,
  Length: 0,
};

Number.prototype.toMMSS = function () {
  let rounded = Math.floor(this);
  let minutes = Math.floor(rounded / 60);
  var seconds = Math.floor(rounded % 60);

  if (minutes < 10) {
    minutes = "0" + minutes;
  }
  if (seconds < 10) {
    seconds = "0" + seconds;
  }
  return minutes + ":" + seconds;
};

class Controls extends Component {
  constructor(props) {
    super(props);
    this.state = initialData;

    this.play = this.play.bind(this);
    this.pause = this.pause.bind(this);
    this.stop = this.stop.bind(this);
    this.nextTrack = this.nextTrack.bind(this);
    this.prevTrack = this.prevTrack.bind(this);
    this.volumeUp = this.volumeUp.bind(this);
    this.volumeDown = this.volumeDown.bind(this);
  }

  componentDidMount() {
    this.updateStatus();
    this.timerID = setInterval(() => this.updateStatus(), 1000);
  }

  componentWillUnmount() {
    clearInterval(this.timerID);
  }

  updateStatus() {
    fetch("/api/control/")
      .then((response) => response.json())
      .then((data) => {
        //console.log("Control: ", data)
        this.setState(data);
      });
  }

  postCommand(command, opts) {
    console.log("Posting command: ", command, opts);
    let data = Object.assign(
      {
        command: command,
      },
      opts
    );
    fetch("/api/control/", {
      method: "POST",
      body: JSON.stringify(data),
      headers: {
        "Content-Type": "application/json",
      },
    })
      .then((response) => response.json())
      .then((data) => {
        this.setState(data);
      });
  }

  play() {
    this.postCommand("play");
  }
  stop() {
    this.postCommand("stop");
  }
  pause() {
    this.postCommand("pause");
  }
  nextTrack() {
    this.postCommand("next");
  }
  prevTrack() {
    this.postCommand("prev");
  }
  volumeUp() {
    this.postCommand("volumeDelta", { volumeDelta: 5 });
  }

  volumeDown() {
    this.postCommand("volumeDelta", { volumeDelta: -5 });
  }

  render() {
    let trackInfo;
    if (this.state.CurrentArtist) {
      trackInfo = (
        <>
          <span className="artist">{this.state.CurrentArtist}</span>
          <span> - </span>
          <span className="track">{this.state.CurrentTrack}</span>
        </>
      );
    }
    return (
      <div id="controls">
        &nbsp;
        <span id="go-home" title="Go to Artists page">
          Home
        </span>
        <span className="control-break">|</span>
        <span>vol {this.state.Volume}%</span>
        <div
          id="vol-down"
          className="button"
          title="Decrease Volume"
          onClick={this.volumeDown}
        />
        <div
          id="vol-up"
          className="button"
          title="Increase Volume"
          onClick={this.volumeUp}
        />
        <span className="control-break">|</span>
        <div
          id="stop"
          className="button"
          title="Stop playback"
          onClick={this.stop}
        />
        <div
          id="play"
          className="button"
          title="Start playback"
          onClick={this.play}
        />
        <div
          id="pause"
          className="button"
          title="Pause playback"
          onClick={this.pause}
        />
        <span className="control-break">|</span>
        <div
          id="prev-track"
          className="button"
          title="Play previous track"
          onClick={this.prevTrack}
        />
        <span> track</span>
        <div
          id="next-track"
          className="button"
          title="Play next track"
          onClick={this.nextTrack}
        />
        <span className="control-break">|</span>
        {trackInfo}
        <span className="control-break">|</span>
        <span>
          {this.state.Elapsed.toMMSS()} / {this.state.Length.toMMSS()}
        </span>
        <span className="lower" />
      </div>
    );
  }
}

export default Controls;
