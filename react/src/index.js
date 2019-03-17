console.log("hello world!");

import React from 'react';
import ReactDOM from 'react-dom';

import Playlist from './playlist';

import './styles/app.css';

const tracks = ["track 1", "track 2"];

class Clock extends React.Component {
	constructor(props) {
		super(props);
		this.state = {date: new Date()};
	}

	componentDidMount() {
		this.timerID = setInterval(
			() => this.tick(),
			1000
		);

	}

	componentWillUnmount() {
		clearInterval(this.timerID);
	}

	tick() {
		this.setState({
			date: new Date()
		});
	}

	render() {
		return (
			<div>
			<h1>Hello, world!</h1>
			<h2>It is {this.state.date.toLocaleTimeString()}.</h2>
			</div>
		);
	}
}

function init() {
  const element = (
    <div>
			<Clock />
      <Playlist />
    </div>
  );
  ReactDOM.render(element, document.getElementById('index'));
}

init()

module.hot.accept();
