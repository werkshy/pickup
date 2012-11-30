
// on ready
$(function() {

	// tiny jquery plugin for posting JSON
	// http://rohanradio.com/blog/2011/02/22/posting-json-with-jquery/
	jQuery.extend({
		postJSON: function(url, data, callback) {
		  return jQuery.ajax({
			type: "POST",
			url: url,
			data: JSON.stringify(data),
			success: callback,
			dataType: "json",
			contentType: "application/json",
			processData: false  // don't form-encode the data
		  });
		}
  });

	var t0 = new Date();
	window.App = {}

	// TODO: this isn't so pretty, I'd like to do better but without
	// re-rendering the entire artist list when we switch to it
	App.showView = function(viewId) {
		console.log("Showing view '%s'", viewId);
		$(viewId).show();
		var allViews = ["#allArtistsView", "#artistView", "#albumView"]
		for (i=0; i<allViews.length; i++) {
			if (allViews[i] !== viewId) {
				$(allViews[i]).hide();
			}
		}
		console.log("Shown view '%s'", viewId);
	}

	initArtists(App);
	initAlbums(App);
	initControls(App);
	initRoutes(App);

	// Start the backbone router / history after we retrieve the artists
	// collection.
	App.artists = new App.ArtistList();
	var t1 = new Date();
	// fetch all data and initialize view on success
	App.artists.fetch({
			success: function() {
				var t2 = new Date();
				console.log("Fetched all artists (%d) in %d ms", 
						App.artists.length, t2-t1);
				console.log("Rendering the artist list");
				App.artistListView = new App.ArtistListView({collection:App.artists});
				console.log("Initiating the router");
				App.router = new App.Router();
				Backbone.history.start();
				// render the artist list in the background, in case it's not
				// the first thing we're looking at
				setTimeout(function() {
					App.artistListView.renderDontShow();
					console.log("Rendered all artists");
					}, 10);
			}
	});
	// Set up player controls
	console.log("Initializing controls")
	App.control = new App.Control()
	App.controlView = new App.ControlView({model:App.control})
	App.control.fetch()
	// Set up polling for control status
	setInterval(function() {
		console.log("Fetching controls");
		App.control.fetch()
	}, 2500);
});
