
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
			console.log("Category %s has %d artists",
				this.name, this.artists.size())
			// TODO: handle bare albums (missing server side)
		},
		urlRoot: "/cateories/",
		url: function() {
			return this.urlRoot;
		},
	});

	App.Categories = Backbone.Collection.extend({
		model: App.Category,
		url: "/categories/"
	});


	App.CategoryView = Backbone.View.extend({
		className: "category",
		el : "#content",
		events : {
			"click .artist-title" : "viewArtist",
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
		close: function() {
			this.unbind();
			this.undelegateEvents();
		},
	});

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
