// on ready
function initPlaylist(App) {
	App.PlaylistTrack = Backbone.Model.extend({
		initialize: function() {
		},
		urlRoot: "/playlist/",
		url: function() {
			return this.urlRoot;
		}
	});

	App.Playlist = Backbone.Collection.extend({
		model: App.PlaylistTrack,
		url: "/playlist/"
	});

	// TODO: need to make the playlist a collection, and flesh out the model for
	// each track a bit more.
	App.PlaylistView = Backbone.View.extend({
			el : "#playlist-wrapper",
			tracksEl : "#playlist-tracks",
			className : "playlist-tracks",
			initialize: function() {
				_.bindAll(this, "render")
				_.bindAll(this)
				this.$tracksEl = $(this.tracksEl)
				this.collection.bind("change", this.render)
				this.collection.bind("reset", this.render)
				this.render();
			},
			events : {
				"click .top" : "toggleVisible",
				"click .bottom" : "toggleVisible",
			},
			render: function() {
				var that = this;
				this.$tracksEl.html("")
				_.each(this.collection.models, function (item) {
						that.renderItem(item);
					}, this);
			},
			renderItem: function (item) {
				// render a single artists <li> in the artist list view
				var itemView = new App.PlaylistItemView({
					model: item
				});
				this.$tracksEl.append(itemView.render().el);
			},
			toggleVisible: function(event) {
				// if it us currently hidden, this is 'none'
				var oldState = $("#playlist-slider").css('display');
				$("#playlist-slider").slideToggle(150);
				var visible = (oldState == 'none');
				if (visible) {
					$("#playlist-wrapper").addClass("open")
				} else {
					$("#playlist-wrapper").removeClass("open")
				}
			}
	});

	App.PlaylistItemView = Backbone.View.extend({
		tagName: "li",
		className: "playlist-list-item",
		template: Handlebars.compile($("#playlist-item-template").html()),
		events : {
		},
		render: function () {
			var html = this.template(this.model.attributes);
			this.$el.html(html)
			return this;
		},
	});

}

