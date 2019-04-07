import React from "react";
import ReactDOM from "react-dom";
import {
  Route,
  Link,
  BrowserRouter as Router,
  Switch,
  Redirect
} from "react-router-dom";

import Playlist from "./playlist";
import Controls from "./controls";
import Category from "./category";
import CategoryList from "./category_list";
import Artist from "./artist";
import Album from "./album";
import NotFound from "./notfound";

import "./styles/app.css";

class RedirectToDefault extends React.Component {
  render() {
    return <Redirect to="/category/Music" />;
  }
}

function init() {
  const routing = (
    <Router>
      <Controls />
      <CategoryList />
      <Switch>
        <Route exact path="/" component={RedirectToDefault} />
        <Route path="/category/:category" component={Category} />
        <Route path="/artist/:category/:artist" component={Artist} />
        <Route path="/album/:category/:artist/:album" component={Album} />
        <Route component={NotFound} />
      </Switch>
      <Playlist />
    </Router>
  );
  ReactDOM.render(routing, document.getElementById("index"));
}

init();

module.hot.accept();
