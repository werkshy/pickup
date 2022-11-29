import React, { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { useInterval } from "./utils/useInterval";
import { playAlbum } from "./actions";

function CategoryArtist(props) {
  let target = `/artist/${props.category}/${props.artist.Name}`;
  return (
    <li>
      <span
        className="artist-title"
        data-artist="{this.props.artist.Name}"
        data-category=""
      >
        <Link to={target}>{props.artist.Name}</Link>
      </span>
    </li>
  );
}

function CategoryAlbum(props) {
  const add = () => {
    playAlbum(props.category, undefined, props.name, false);
  };

  const play = () => {
    playAlbum(props.category, undefined, props.name, true);
  };

  const target = `/album/${props.category}/${props.name}`;
  return (
    <li id={props.name}>
      <span className="album-title abbreviated">
        <Link to={target}>{props.name}</Link>
      </span>
      <div
        className="album-actions add"
        title={`Add '${props.name}'`}
        onClick={add}
      />
      <div
        className="album-actions play"
        title={`Play '${props.name}'`}
        onClick={play}
      />
    </li>
  );
}

export function Category() {
  let { name } = useParams();
  const [artists, setArtists] = useState([]);
  const [albumNames, setAlbumNames] = useState([]);

  // FIXME: make an endpoint that returns just one category
  const fetchData = async () => {
    const response = await fetch("/api/categories/");
    const data = await response.json();
    let category = data.find((category) => category.Name === name);
    console.log("category", category);
    setArtists(category.Artists);
    setAlbumNames(category.AlbumNames);
  };

  useEffect(() => {
    fetchData();
  }, [name]);

  useInterval(async () => {
    await fetchData();
  }, 1000);

  console.log(`Rendering category ${name}`);
  // Handle category albums without an artist:
  let albums;
  if (albumNames.length) {
    albums = (
      <>
        <h3>Albums</h3>
        <ul id="albumList">
          {albumNames.map((album) => (
            <CategoryAlbum
              name={album}
              category={name}
              key={`album__${album}`}
            />
          ))}
        </ul>
      </>
    );
  }

  return (
    <div>
      <h1>Category: {name}</h1>
      <div className="category multicolumn" data-category="{name}">
        {albums}
        <h3>Artists</h3>
        <ul id="artistList">
          {artists.map((artist) => (
            <CategoryArtist category={name} artist={artist} key={artist.Name} />
          ))}
        </ul>
      </div>
    </div>
  );
}
