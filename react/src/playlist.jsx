import React, { useEffect, useState } from "react";
import { useInterval } from "./utils/useInterval";

function PlaylistTrack(props) {
  let trackNum = Math.trunc(props.track.Pos) + 1;
  return (
    <li>
      <span className="track-number">{trackNum}</span>
      <span className="track-title abbreviated">{props.track.Name}</span>
      <span className="track-artist abbreviated">{props.track.Artist}</span>
      <span className="track-album abbreviated">{props.track.Album}</span>
    </li>
  );
}

export function Playlist() {
  const [isVisible, setVisible] = useState(false);
  const [tracks, setTracks] = useState([]);

  const toggleVisibility = () => {
    setVisible(!isVisible);
  };

  const updateTracks = async () => {
    const response = await fetch("/api/playlist/");
    const data = await response.json();
    setTracks(data);
  };

  useEffect(() => {
    updateTracks();
  }, []);

  useInterval(async () => {
    await updateTracks();
  }, 1000);

  return (
    <div id="playlist-wrapper" className={isVisible ? "open" : "none"}>
      <div className="top" onClick={toggleVisibility}>
        <span>Playlist</span>
        <div className="arrow up-arrow" />
      </div>
      <div id="playlist-slider" style={{ display: isVisible ? "block" : "none" }}>
        <div id="playlist">
          <ul id="playlist-tracks">
            <>
              {tracks.map((track) => (
                <PlaylistTrack key={track.Pos} track={track} />
              ))}
            </>
          </ul>
        </div>
      </div>
    </div>
  );
}
