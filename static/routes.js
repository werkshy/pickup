
function initRoutes(App) {

	App.Route = {}
	App.Route.allArtists = function() {
		console.log("Route: show all artists (%d)", App.artists.length)
		App.showView("#allArtistsView");
	}

	App.Route.artist = function(query) {
		console.log("Route: show artist '%s'", query);

		console.log("Looking in %d artists", App.artists.length);
		var artists = App.artists.where({Name:query})
		var artist = artists[0];
		console.log("Found %d artists matching %s", artists.length, query);
		var artistView = new App.ArtistView({model:artist})
	}

	/** Wire up the controls in the top of the page **/
	initControls();

	// Set up the Backbone router
	App.Router = Backbone.Router.extend({
		routes: {
			"artists/:query" : "artistRoute",
			"artists" : "allArtists",
			"about": "showAbout",
			"*other"    : "defaultRoute"
		},
		allArtists: App.Route.allArtists,
		artistRoute: App.Route.artist,
		showAbout: function() {
			console.log("Show about");
		},
		defaultRoute: function(other){
			console.log("Invalid. You attempted to reach:" + other);
		}
	});

}
