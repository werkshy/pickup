
// on ready
function initArtists(App) {

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
				$(this.$el).find(".play").click(function(event){
					that.playAlbum(event);
				});
				$(this.$el).find(".add").click(function(event){
					that.addAlbum(event);
				});
				App.showView("#artistView")
				return this;
			},
			playAlbum: function(event) {
				var album = event.currentTarget.parentElement.id;
				console.log("Play album: %s/%s", this.model.get('Name'), album);
				playAlbum(this.model.get('Name'), album, true)
			},
			addAlbum: function(event) {
				var album = event.currentTarget.parentElement.id;
				console.log("Add album: %s/%s", this.model.get('Name'), album);
				playAlbum(this.model.get('Name'), album, false)
			}
	});

	App.ArtistList = Backbone.Collection.extend({
		model: App.Artist,
		url: "/artists/"
	});

	App.ArtistListView = Backbone.View.extend({
		el : "#artistList",
		className: "artist-list",

		// render on change to collection
		initialize: function() {
			//this.model.bind('change', this.render, this);
			this.collection.bind("reset", this.render, this);
		},
		render: function () {
			// Render and show the artist list view
			var that = this;
			this.renderDontShow()
			App.showView("#allArtistsView");
			return this;
		},
		renderDontShow: function () {
			// render the artist list view but don't switch to that view
			var t0 = new Date();
			var that = this;
			_.each(this.collection.models, function (item) {
					that.renderItem(item);
				}, this);
			var t1 = new Date();
			console.log("Took %d ms to render", t1-t0);
			return this;
		},
		renderItem: function (item) {
			// render a single artists <li> in the artist list view
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
			console.log("view artist " + this.model.get('Name'));
			App.router.navigate("artists/" + this.model.get('Name'),
					{ 'trigger' : true});
		},
	});
}

