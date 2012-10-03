
// on ready
$(function() {

	var t0 = new Date();
	window.App = {}

	App.showView = function(viewId) {
		console.log("Showing view '%s'", viewId);
		$(viewId).show();
		var allViews = ["#allArtistsView", "#artistView"]
		for (i=0; i<allViews.length; i++) {
			if (allViews[i] !== viewId) {
				$(allViews[i]).hide();
			}
		}
		console.log("Shown view '%s'", viewId);
	}

	initArtists(App);
	initRoutes(App);

	// Start the backbone router / history
	App.artists = new App.ArtistList();
	var t1 = new Date();
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
});
