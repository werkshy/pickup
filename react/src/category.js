import React, { Component } from "react";
import { Link } from "react-router-dom";
import { playAlbum, playTrack } from "./actions";

class CategoryArtist extends Component {
  render() {
    let target =
      "/artist/" + this.props.category + "/" + this.props.artist.Name;
    return (
      <li>
        <span
          className="artist-title"
          data-artist="{this.props.artist.Name}"
          data-category=""
        >
          <Link to={target}>{this.props.artist.Name}</Link>
        </span>
      </li>
    );
  }
}

class CategoryAlbum extends Component {
  constructor(props) {
    super(props);
    this.add = this.add.bind(this);
    this.play = this.play.bind(this);
  }

  render() {
    const target = "/album/" + this.props.category + "/" + this.props.name;
    return (
      <li id={this.props.name}>
        <span className="album-title abbreviated">
          <Link to={target}>{this.props.name}</Link>
        </span>
        <div
          className="album-actions add"
          title={`Add '${this.props.name}'`}
          onClick={this.add}
        />
        <div
          className="album-actions play"
          title={`Play '${this.props.name}'`}
          onClick={this.play}
        />
      </li>
    );
  }

  add() {
    playAlbum(this.props.category, undefined, this.props.name, false);
  }

  play() {
    playAlbum(this.props.category, undefined, this.props.name, true);
  }
}

class Category extends Component {
  constructor(props) {
    super(props);
    console.log(props);
    this.state = { Artists: [], AlbumNames: [] };
  }

  componentDidMount() {
    this.getData();
  }

  componentWillUnmount() {}

  componentDidUpdate(prevProps) {
    let oldCategory = prevProps.match.params.category;
    let newCategory = this.props.match.params.category;
    if (newCategory !== oldCategory) {
      this.getData();
    }
  }

  category() {
    return this.props.match.params.category;
  }

  // FIXME: make an endpoint that returns just one category
  getData() {
    fetch("/api/categories/")
      .then((response) => response.json())
      .then((data) => {
        let category = data.find(
          (category) => category.Name === this.category()
        );
        console.log("category", category);
        this.setState(category);
      });
  }

  render() {
    // Handle category albums without an artist:
    let albums;
    if (this.state.AlbumNames.length) {
      albums = (
        <>
          <h3>Albums</h3>
          <ul id="albumList">
            {this.state.AlbumNames.map((album) => (
              <CategoryAlbum
                name={album}
                category={this.category()}
                key={"album__" + album}
              />
            ))}
          </ul>
        </>
      );
    }

    return (
      <div>
        <h1>Category: {this.state.Name}</h1>
        <div className="category multicolumn" data-category="{this.state.Name}">
          {albums}
          <h3>Artists</h3>
          <ul id="artistList">
            {this.state.Artists.map((artist) => (
              <CategoryArtist
                category={this.category()}
                artist={artist}
                key={artist.Name}
              />
            ))}
          </ul>
        </div>
      </div>
    );
  }
}

export default Category;
