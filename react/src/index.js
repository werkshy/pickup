import React from 'react';
import ReactDOM from 'react-dom';
import { Route, Link, BrowserRouter as Router, Switch, Redirect } from 'react-router-dom'

import Playlist from './playlist';
import Controls from './controls';
import Category from './category';
import Clock from './clock'; // FIXME dummy component for dev only
import NotFound from './notfound';

import './styles/app.css';

class RedirectToDefault extends React.Component {
	render() {
		return <Redirect to='/category/Music' />
	}
}

function init() {
	const nav = (
		<ul>
			<li><Link to="/category/Music">Home</Link></li>
			<li><Link to="/category/_Metal">Metal</Link></li>
			<li><Link to="/clock">Clock</Link></li>
		</ul>
	)

  const routing = (
    <Router>
      <Controls />
      {nav}
      <Switch>
        <Route exact path="/" component={RedirectToDefault} />
        <Route path="/category/:name" component={Category} />
        <Route path="/clock" component={Clock} />
        <Route component={NotFound} />
      </Switch>
      <Playlist />
    </Router>
  );
  ReactDOM.render(routing, document.getElementById('index'));
}

init()

module.hot.accept();
