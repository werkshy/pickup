
// on ready
function initArtists(App) {

	App.Artist = Backbone.Model.extend({
		initialize: function() {
			//console.log("Initializing artist");
		},
		urlRoot: "/artists/",
		url: function() {
			return this.urlRoot + this.get('Name');
		},
		playAlbum: function(albumName, immediate) {
			var album = App.Route.getAlbum(this.get("Category"),
				this.get("Name"), albumName);
			album.play(immediate);
		}
	});

	App.ArtistView = Backbone.View.extend({
			el : "#content",
			className : "artist",
			initialize: function() {
				_.bindAll(this, "render")
				_.bindAll(this)
				this.template = Handlebars.compile($("#artist-template").html())
				this.render();
                this.name = this.model.get("Name");
			},
			events : {
				"click .play" : "playAlbum",
				"click .add" : "addAlbum",
				"click .album-title" : "showAlbum"
			},
			render: function() {
				console.log("Render artist '%s'", this.name);
				this.$el.html(this.template(this.model.attributes))
				//App.showView("#artistView")
				return this;
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
						this.model.get("Category"),
						this.name, album);
					App.router.navigate("albums/" + this.model.get("Category")
						+ "/" + this.model.get('Name') + "/" + album,
						{ 'trigger' : true});
			},
			close: function() {
                console.log("Closing artist view '%s'", this.name);
				this.unbind();
				this.undelegateEvents();
			},
	});

	App.Artists = Backbone.Collection.extend({
		model: App.Artist,
		url: "/artists/"
	});

}

