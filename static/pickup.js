Artist = Backbone.Model.extend({
	initialize: function(){
		console.log("Initializing artist");
	}
});


var jsonArtist = {
	"Name":"Smash",
	"AlbumNames":["All By Myself EP","Icon","Self Abused"]
}


$(function() {
	// demo data for development
	var artist = new Artist;
	artist.set(jsonArtist);

	// Set up the Backbone router
	var AppRouter = Backbone.Router.extend({
		routes: {
			"artists/:query" : "artistRoute",
			"about": "showAbout",
			"*other"    : "defaultRoute"
		},
		artistRoute: function(query) {
			console.log("Artist route: " +  query);
		},
		showAbout: function() {
			console.log("Show about");
		},
		defaultRoute: function(other){
			console.log("Invalid. You attempted to reach:" + other);
		}
	});

	// Initiate the router
	var app_router = new AppRouter();
	// Start the backbone router / history
	Backbone.history.start();
});
