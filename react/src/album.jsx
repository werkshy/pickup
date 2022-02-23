import React, { useEffect, useState } from "react";
import { useParams } from "react-router-dom";

import { playAlbum, playTrack } from "./actions";

function AlbumTrack(props) {
  const add = () => {
    playTrack(props.category, props.artist, props.album, props.track, false);
  };

  const play = () => {
    playTrack(props.category, props.artist, props.album, props.track, true);
  };

  // FIXME the titles aren't interpolating
  return (
    <li id="{{.}}">
      <span className="track-title abbreviated" onClick={play}>
        {props.track}
      </span>
      <div
        className="track-actions add"
        title={`Add ${props.track}`}
        onClick={add}
      />
      <div
        className="track-actions play"
        title={`Play ${props.track}`}
        onClick={play}
      >
        &nbsp;
      </div>
    </li>
  );
}

export function Album() {
  let { category, artist, album } = useParams();
  const [tracks, setTracks] = useState([]);

  // FIXME: make (or use) an endpoint that returns just one artist
  // e.g. /api/artist/:category/:artist
  // or, perhaps we could assign a unique id to each artist, album and maybe even track
  const fetchData = async () => {
    let url = "/api/albums/" + category + "/";
    if (artist) {
      url += artist + "/";
    }
    url += album;
    const response = await fetch(url);
    const data = await response.json();
    console.log("album data", data);
    setTracks(data.Tracks);
  };

  useEffect(() => {
    fetchData();
  }, [category, artist, album]);

  const add = () => {
    playAlbum(category, artist, album, false);
  };

  const play = () => {
    playAlbum(category, artist, album, true);
  };

  return (
    <div id="albumView" className="singlecolumn">
      <h2 className="artist-name">
        {artist ? artist + " - " : ""}
        {album}
        <span
          className="album-actions add"
          onClick={add}
          title={`Add album '${album}'`}
        />
        <span
          className="album-actions play"
          onClick={play}
          title={`Play album '${album}'`}
        />
      </h2>
      <ul id="trackList">
        {tracks.map((track) => (
          <AlbumTrack
            key={track}
            category={category}
            artist={artist}
            album={album}
            track={track}
          />
        ))}
      </ul>
    </div>
  );
}
