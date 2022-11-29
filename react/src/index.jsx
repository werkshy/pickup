import React from "react";
import { render } from "react-dom";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";

import { Playlist } from "./playlist";
import { Controls } from "./controls";
import { Category } from "./category";
import { CategoryList } from "./category_list";
import { Artist } from "./artist";
import { Album } from "./album";
import { NotFound } from "./notfound";

import "./styles/app.css";

function App() {
  return (
    <div>
      <Controls />
      <CategoryList />
      <Routes>
        <Route path="/" element={<Navigate replace={true} to="/category/Music" />} />
        <Route path="/category/:name" element={<Category />} />
        <Route path="/artist/:category/:artist" element={<Artist />} />
        <Route path="/album/:category/:artist/:album" element={<Album />} />
        <Route path="/album/:category/:album" element={<Album />} />
        <Route path="/*" element={<NotFound />} />
      </Routes>
      <Playlist />
    </div>
  );
}

render(
  <BrowserRouter>
    <App />
  </BrowserRouter>,
  document.getElementById("index"),
);
