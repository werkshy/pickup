
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

	App.Route.album = function(artistQuery, albumQuery) {
		console.log("Route: show album '%s/%s'", albumQuery, artistQuery);

		console.log("Looking in %d artists", App.artists.length);
		var artists = App.artists.where({Name:artistQuery})
		console.log("Found %d artists", artists.length);
		if (artists.length != 1) {
			console.log("Yikes! too may artists");
		}
		var artist = artists[0];
		//console.log(artist)
		var album = new App.Album({
					"Artist" : artist.get("Name"),
					"Name" : albumQuery
		})
		console.log("Fetching album")
		album.fetch({
				success: function() {
					console.log("retrieved album")
					var albumView = new App.AlbumView({model:album})
				}
		})
	}

	// Set up the Backbone router
	App.Router = Backbone.Router.extend({
		routes: {
			"artists/:query" : "artistRoute",
			"artists" : "allArtists",
			"albums/:artistQuery/:albumQuery" : "albumRoute",
			"about": "showAbout",
			"*other"    : "defaultRoute"
		},
		allArtists: App.Route.allArtists,
		artistRoute: App.Route.artist,
		albumRoute: App.Route.album,
		showAbout: function() {
			console.log("Show about");
		},
		defaultRoute: function(other){
			App.Route.allArtists()
		}
	});

}
