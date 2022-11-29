import React, { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";

import { playAlbum } from "./actions";

function ArtistAlbum(props) {
  const add = () => {
    playAlbum(props.category, props.artist, props.name, false);
  };

  const play = () => {
    playAlbum(props.category, props.artist, props.name, true);
  };

  let target =
    `/album/${props.category}/${props.artist}/${props.name}`;
  return (
    <li id={props.name}>
      <span className="album-title abbreviated">
        <Link to={target}>{props.name}</Link>
      </span>
      <div
        className="album-actions add"
        title="Add {props.name}"
        onClick={add}
      />
      <div
        className="album-actions play"
        title="Play {props.name}"
        onClick={play}
      />
    </li>
  );
}

// FIXME we aren't handling loose tracks under an artist (I don't really have that in my collection)

export function Artist() {
  let { category, artist } = useParams();
  const [albumNames, setAlbumNames] = useState([]);
  const [trackNames, setTrackNames] = useState([]);

  // FIXME: make (or use) an endpoint that returns just one artist
  // e.g. /api/artist/:category/:artist
  // or, perhaps we could assign a unique id to each artist, album and maybe even track
  const fetchData = async () => {
    const response = await fetch("/api/categories/");
    const data = await response.json();
    const categoryData = data.find((c) => c.Name === category);
    const artistData = categoryData.Artists.find((a) => a.Name === artist);
    console.log("artist", artistData);
    setAlbumNames(artistData.AlbumNames);
    setTrackNames(artistData.TrackNames);
  };

  useEffect(() => {
    fetchData();
  }, [artist]);

  console.log(`Artist '${artist}' state: `, { albumNames, trackNames });
  return (
    <div id="artistView" className="singlecolumn">
      <h2>{artist}</h2>
      <ul id="albumList">
        {albumNames.map((album) => (
          <ArtistAlbum
            key={album}
            category={category}
            artist={artist}
            name={album}
          />
        ))}
      </ul>
    </div>
  );
}
