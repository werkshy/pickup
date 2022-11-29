import React, { useEffect, useState } from "react";
import { useInterval } from "./utils/useInterval";

let initialData = {
  State: "stop",
  Volume: 100,
  CurrentArtist: "...",
  CurrentAlbum: "...",
  CurrentTrack: "...",
  Elapsed: 0,
  Length: 0,
};

function toMMSS(num) {
  let rounded = Math.floor(num);
  let minutes = Math.floor(rounded / 60);
  var seconds = Math.floor(rounded % 60);

  return (
    `${minutes.toString(10).padStart(2, "0")}:${seconds.toString(10).padStart(2, "0")}`
  );
}

export function Controls() {
  const [data, setData] = useState(initialData);

  const fetchData = async () => {
    const response = await fetch("/api/control/");
    const json = await response.json();
    setData(json);
  };

  useEffect(() => {
    fetchData();
  }, []);

  useInterval(async () => {
    await fetchData();
  }, 1000);

  const postCommand = async (command, opts) => {
    let data = Object.assign(
      {
        command: command,
      },
      opts
    );
    const response = await fetch("/api/control/", {
      method: "POST",
      body: JSON.stringify(data),
      headers: {
        "Content-Type": "application/json",
      },
    });
    const json = await response.json();
    setData(json);
  };

  let trackInfo = <span>Not playing</span>;
  if (data.CurrentArtist) {
    trackInfo = (
      <>
        <span className="artist">{data.CurrentArtist}</span>
        <span> - </span>
        <span className="track">{data.CurrentTrack}</span>
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
      <span>vol {data.Volume}%</span>
      <div
        id="vol-down"
        className="button"
        title="Decrease Volume"
        onClick={() => postCommand("volumeDelta", { volumeDelta: -5 })}
      />
      <div
        id="vol-up"
        className="button"
        title="Increase Volume"
        onClick={() => postCommand("volumeDelta", { volumeDelta: 5 })}
      />
      <span className="control-break">|</span>
      <div
        id="stop"
        className="button"
        title="Stop playback"
        onClick={() => postCommand("stop")}
      />
      <div
        id="play"
        className="button"
        title="Start playback"
        onClick={() => postCommand("play")}
      />
      <div
        id="pause"
        className="button"
        title="Pause playback"
        onClick={() => postCommand("pause")}
      />
      <span className="control-break">|</span>
      <div
        id="prev-track"
        className="button"
        title="Play previous track"
        onClick={() => postCommand("prev")}
      />
      <span> track</span>
      <div
        id="next-track"
        className="button"
        title="Play next track"
        onClick={() => postCommand("next")}
      />
      <span className="control-break">|</span>
      {trackInfo}
      <span className="control-break">|</span>
      <span>
        {toMMSS(data.Elapsed)} / {toMMSS(data.Length)}
      </span>
      <span className="lower" />
    </div>
  );
}
