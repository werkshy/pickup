
// on ready
$(function() {

	window.App = {}

	App.Artist = Backbone.Model.extend({
		initialize: function() {
			//console.log("Initializing artist");
		},
		urlRoot: "/artists/",
		url: function() {
			return this.urlRoot + this.get('Name');
		}
	});

	App.ArtistView = Backbone.View.extend({
			el : "#artist",
			className : "artist",
			initialize: function() {
				this.render();
			},
			render: function() {
				console.log("Render artist '%s'", this.model.get("Name"));
				var that = this;
				this.$el.html(ich.artist(this.model.toJSON()));
				$(this.$el).find("li").click(function(event){
					console.log("Clicked");
					that.viewAlbum(event);
				});
				return this;
			},
			viewAlbum: function(event) {
				var album = event.currentTarget.id;
				console.log("Play Album: %s/%s", this.model.get('Name'), album);
			}
	});

	App.ArtistList = Backbone.Collection.extend({
		model: App.Artist,
		url: "/artists/"
	});

	App.showView = function(viewId) {
		console.log("Showing view '%s'", viewId);
		$(viewId).show();
		var allViews = ["#allArtistsView", "#artistView"]
		for (i=0; i<allViews.length; i++) {
			if (allViews[i] !== viewId) {
				$(allViews[i]).hide();
			}
		}
	}

	App.ArtistListView = Backbone.View.extend({
		el : "#artistList",
		className: "artist-list",

		// render on change to collection
		initialize: function() {
			//this.model.bind('change', this.render, this);
			this.collection.bind("reset", this.render, this);
		},
		render: function () {
			App.showView("#allArtistsView");
			var that = this;
			_.each(this.collection.models, function (item) {
				that.renderItem(item);
			}, this);
		},
		renderItem: function (item) {
			var itemView = new App.ArtistListItemView({
				model: item
			});
			this.$el.append(itemView.render().el);
		}
	});

	App.ArtistListItemView = Backbone.View.extend({
		tagName: "li",
		className: "artist-list-item",

		events : {
			"click" : "viewArtist",
		},
		render: function () {
			this.$el.html(ich.artistList(this.model.toJSON()));
			return this;
		},
		viewArtist: function() {
			// Todo swipe visible panel or tab left/right as we navigate
			App.showView("#artistView");
			console.log("view artist " + this.model.get('Name'));
			App.router.navigate("artists/" + this.model.get('Name'),
					{ 'trigger' : true});
		},
	});

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

	$("h1#home").click(function() {
		App.router.navigate("artists",
				{ 'trigger' : true});
	});

	// Set up the Backbone router
	var AppRouter = Backbone.Router.extend({
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

	// Initiate the router
	App.router = new AppRouter();
	// Start the backbone router / history
	App.artists = new App.ArtistList();
	App.artists.fetch({
			success: function() {
				console.log("Fetched all artists: %d", App.artists.length);
				Backbone.history.start();
				console.log("Rendering the artist list");
				App.artistListView = new App.ArtistListView({collection:App.artists});
				App.artistListView.render();
				console.log("Rendered all artists");
			}
	});

});
