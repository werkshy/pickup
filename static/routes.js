
function initRoutes(App) {

	App.Route = {}
	App.Route.category = function(categoryName) {
		console.log("Route: show category (%s)", categoryName);
		var categories = App.categories.where({Name:categoryName})
		var category = categories[0];
		if (typeof App.appView !== 'undefined') {
			App.appView.close();
		}
		App.appView = new App.CategoryView({model:category});
	}

	App.Route.artist = function(categoryName, artistName) {
		console.log("Route: show artist '%s/%s'", categoryName, artistName);
		var categories = App.categories.where({Name:categoryName})
		var category = categories[0];
		console.log("Category has %d artists", category.artists.size());
		var artists = category.artists.where({Name:artistName})
		console.log("Found %d matching artists", artists.length);
		var artist = artists[0];
		if (typeof App.appView !== 'undefined') {
			App.appView.close(); // TODO move all this to AppView
		}

		App.appView = new App.ArtistView({model:artist})

	}

	App.Route.album = function(q1, q2, q3) {
		var album = null;
		if (typeof q3 !== 'undefined') {
			// category, artist, album
			album = App.Route.getAlbum(q1, q2, q3);
			console.log("Route: show album '%s/%s/%s'", q1, q2, q3);
		} else {
			album = App.Route.getAlbum(q1, null, q3);
			console.log("Route: show album '%s/%s'", q1, q2);
		}

		console.log("Fetching album")
		album.fetch({
				success: function() {
					console.log("retrieved album")
					// TODO: move this into an app view
					// Gotta kill the existing view otherwise the events fire
					// on zombie views.
					if (typeof App.appView !== "undefined") {
						App.appView.close();
					}
					App.appView = new App.AlbumView({model:album})
				}
		})
	}

	App.Route.getAlbum = function(categoryName, artistName, albumName) {
		var category = App.categories.where({Name:categoryName})[0]
		if (typeof artistName === 'undefined') {
			// no artist, bare album
			artistName  = "Various Artists";
		} else {
			var artist = category.artists.where({Name:artistName})[0]
		}
		return new App.Album({
					"Category" : categoryName,
					"Artist" : artistName,
					"Name" : albumName
		})
	}

	// Set up the Backbone router
	App.Router = Backbone.Router.extend({
		routes: {
			"artists/:category/:artist" : "artistRoute",
			"categories/:category" : "category",
			"albums/:category/:artistQuery/:albumQuery" : "albumRoute",
			"albums/:category/:album" : "albumRoute",
			"about": "showAbout",
			"*other"    : "defaultRoute"
		},
		category: App.Route.category,
		artistRoute: App.Route.artist,
		albumRoute: App.Route.album,
		showAbout: function() {
			console.log("Show about");
		},
		defaultRoute: function(other){
			App.Route.category("Music")
		}
	});

}
