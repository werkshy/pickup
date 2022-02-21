import React, { Component } from "react";
import { Link } from "react-router-dom";

class Category extends Component {
  render() {
    let target = "/category/" + this.props.category;
    return (
      <li>
        <span className="title" data-name="{this.props.category}">
          <Link to={target}>{this.props.category}</Link>
        </span>
      </li>
    );
  }
}

class CategoryList extends Component {
  constructor(props) {
    super(props);
    this.state = { categories: [] };
  }

  // FIXME we should really be using some global state for this
  componentDidMount() {
    this.getData();
  }

  componentWillUnmount() {}

  // FIXME: use global state for this
  getData() {
    fetch("/api/categories/")
      .then((response) => response.json())
      .then((data) => {
        let categories = data.map((category) => category.Name);
        this.setState({ categories: categories });
      });
  }

  render() {
    return (
      <ul className="categories">
        <li className="header">Categories:</li>
        {this.state.categories.map((category) => (
          <Category category={category} key={category} />
        ))}
      </ul>
    );
  }
}

export default CategoryList;
