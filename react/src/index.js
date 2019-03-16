console.log("hello world!");

import React from 'react';
import ReactDOM from 'react-dom';

import Playlist from './playlist';

import './styles/app.css';

const tracks = ["track 1", "track 2"];

function update() {
  const element = (
    <div>
      <h1>Hello, world!</h1>
      <h2>It is {new Date().toLocaleTimeString()}.</h2>
      <Playlist tracks={tracks} />
    </div>
  );
  ReactDOM.render(element, document.getElementById('index'));
}

update()
//setInterval(update, 10000);


module.hot.accept();
