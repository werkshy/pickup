// on ready
function initAlbums(App) {

	App.Album = Backbone.Model.extend({
		initialize: function() {
		},
		urlRoot: "/albums/",
		url: function() {
			return this.urlRoot + this.get("Artist") + "/" + this.get('Name');
		}
	});

	App.AlbumView = Backbone.View.extend({
			el : "#album",
			className : "album",
			initialize: function() {
				_.bindAll(this, "render")
				_.bindAll(this)
				this.template = Handlebars.compile($("#album-template").html())
				this.render();
			},
			render: function() {
				console.log("Render album '%s'", this.model.get("Name"));
				this.$el.html(this.template(this.model.attributes))
				App.showView("#albumView")
				return this;
			},
			events : {
				"click .play" : "playTrackNow",
				"click .add" : "addTrack",
			},
			playTrackNow: function(event) {
				var track = event.currentTarget.parentElement.id;
				console.log("Play track: %s/%s/%s", this.model.get('Artist'),
						this.model.get("Name"), track);
				playTrack(this.model.get('Artist'),
						this.model.get("Name"), track, true)
			},
			addTrack: function(event) {
				var track = event.currentTarget.parentElement.id;
				console.log("Add track: %s/%s/%s", this.model.get('Artist'),
						this.model.get("Name"), track);
				playTrack(this.model.get('Artist'),
						this.model.get("Name"), track, false)
			},
			playTrack: function(artist, album, track, immediate) {
				console.log("Playing track %s/%s/%s (%s)",
						artist, album, track, immediate);
				$.postJSON("/playlist/", {
							"command" : "add",
							"artist" : artist,
							"album" : album,
							"track" : track,
							"immediate" : immediate
						},
						function() {
							console.log("Play success!");
						}
				);
			}
	});

	App.TrackListItemView = Backbone.View.extend({
		tagName: "li",
		className: "album-track-list-item",
		template: Handlebars.compile($("#track-list-template").html()),
		events : {
			"click" : "play",
		},
		render: function () {
			var html = this.template(this.model.attributes);
			//console.log("Rendered %s to '%s'", this.model.attributes.Name, html)
			this.$el.html(html)
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

