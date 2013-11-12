
function initCategories() {

	/**
	 * Category model: a category is a list of artists and possibly 'loose'
	 * albums.
	 */
	App.Category = Backbone.Model.extend({
		initialize: function() {
			this.name = this.get("Name")
			this.artists = new App.Artists(this.attributes.Artists)
			// Set the category name on each artist, for navigation later
			this.artists.each(function(artist, index, list) {
				artist.set("Category", this.name)
			}, this);
			console.log("Category %s has %d artists and %d albums",
				this.name, this.artists.size(), this.attributes.AlbumNames)
		},
		urlRoot: "/cateories/",
		url: function() {
			return this.urlRoot;
		},
		playAlbum: function(albumName, immediate) {
			var album = App.Route.getAlbum(this.get("Name"), undefined, albumName);
			album.play(immediate);
		}
	});

	App.Categories = Backbone.Collection.extend({
		model: App.Category,
		url: "/categories/"
	});


    /**
     * CategoryView is the list of artists and albums in a category.
     */
	App.CategoryView = Backbone.View.extend({
		className: "category",
		el : "#content",
		events : {
			"click .artist-title" : "viewArtist",
            "click .play" : "playAlbum",
            "click .add" : "addAlbum",
            "click .album-title" : "showAlbum"
		},
		// render on change to collection
		initialize: function() {
			this.name = this.model.get("Name");
			console.log("Initializing category view ", this.name);
			//this.model.bind('change', this.render, this);
			this.model.bind("reset", this.render, this);
			this.template = Handlebars.compile($("#category-template").html())
			this.render();
		},
		render: function () {
			// Render and show the artist list view for a single category
			console.log("Rendering category ", this.name);
            console.log("Category model: ", this.model);
			t0 = new Date();
			var el = this.template(this.model.toJSON())
			this.$el.html(el);
			t1 = new Date();
			console.log("Rendered category '%s' in %d ms", this.name, t1-t0);
			return el;
		},
		viewArtist: function(ev) {
			var artist = $(ev.target).data("artist");
			var category = $(ev.target).closest(".category").data("category")
			// TODO swipe visible panel or tab left/right as we navigate
			console.log("view artist %s/%s", category, artist);
			App.router.navigate("artists/" + category + "/" + artist,
					{ 'trigger' : true});
		},
        playAlbum: function(event) {
            var album = event.currentTarget.parentElement.id;
            console.log("Play album: %s/%s", this.name, album);
            this.model.playAlbum(album, true)
        },
        addAlbum: function(event) {
            var album = event.currentTarget.parentElement.id;
            console.log("Add album: %s/%s", this.name, album);
            this.model.playAlbum(album, false)
        },
        showAlbum: function(event) {
            var album = event.currentTarget.parentElement.id;
            console.log("Navigating to album: %s/%s/%s",
                    this.name, album);
                App.router.navigate("albums/" + this.model.get('Name') + "/" + album,
                    { 'trigger' : true});
        },
		close: function() {
            console.log("Closing category view '%s'", this.name);
			this.unbind();
			this.undelegateEvents();
		},
	});

    /**
     * CategoriesView is the always-visible top category/nav list
     */
	App.CategoriesView = Backbone.View.extend({
		el: "#categories",
		events: {
			"click ul.categories li span" : "viewCategory",
		},
		initialize: function() {
			_.bindAll(this)
			this.template = Handlebars.compile($("#categories-template").html())
		},
		render: function() {
			console.log("Rendering categories view")
			el = this.template({
					categories:this.collection.toJSON()
			});
			this.$el.html(el);
		},
		viewCategory: function(ev) {
			var category = $(ev.target).data("name");
			// TODO swipe visible panel or tab left/right as we navigate
			console.log("view category %s", category);
			App.router.navigate("categories/" + category,
					{ 'trigger' : true});
		}
	});



}
