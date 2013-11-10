// on ready
function initAlbums(App) {

	App.Album = Backbone.Model.extend({
		initialize: function() {
		},
		urlRoot: "/albums/",
		url: function() {
			_.bindAll(this)
			return this.urlRoot + this.get("Category") + "/"
					+ this.get("Artist") + "/" + this.get('Name');
		},
		play: function(immediate) {
			console.log("Playing album %s/%s (%s)", this.get('Artist'),
					this.get('Name'), immediate);
			$.postJSON("/playlist/", {
						"Command" : "add",
						"Category" : this.get("Category"),
						"Artist" : this.get('Artist'),
						"Album" : this.get('Name'),
						"Immediate" : immediate
					},
					function() {
						console.log("Play success!");
					}
			);
		},
		playTrack: function(track, immediate) {
			console.log("Playing track %s/%s/%s (%s)",
					this.get('Artist'), this.get('Album'), track, immediate);
			$.postJSON("/playlist/", {
						"Command" : "add",
						"Category" : this.get("Category"),
						"Artist" : this.get('Artist'),
						"Album" : this.get('Name'),
						"Track" : track,
						"Immediate" : immediate
					},
					function() {
						console.log("Play success!");
					}
			);
		},
	});

	App.Albums = Backbone.Collection.extend({
		model: App.Album,
		url: "/albums/"  // uh, not really
	});

    App.AlbumView = Backbone.View.extend({
        el : "#content",
        className : "album",
        initialize: function() {
            _.bindAll(this);
            this.name = this.model.get("Name");
            this.template = Handlebars.compile($("#album-template").html());
            this.render();
        },
        render: function() {
            console.log("Render album '%s'", this.name);
            this.$el.html(this.template(this.model.attributes))
        //App.showView("#albumView")
        return this;
        },
        events : {
            "click .play" : "playTrackNow",
            "click .add" : "addTrack",
            "click .artist-name" : "showArtist",
        },
        playTrackNow: function(event) {
            var track = event.currentTarget.parentElement.id;
            console.log("Play track: %s/%s/%s", this.model.get('Artist'),
                    name, track);
            this.model.playTrack(track, true);
        },
        addTrack: function(event) {
            var track = event.currentTarget.parentElement.id;
            console.log("Add track: %s/%s/%s", this.model.get('Artist'),
                    name, track);
            this.model.playTrack(track, false)
        },
        showArtist: function() {
            console.log("Switch from album view to artist", this.model.get("Artist"));
            App.router.navigate("artists/" + this.model.get("Category")
                    + "/" + this.model.get('Artist'),
                    { 'trigger' : true});
        },
        close: function() {
            console.log("Closing AlbumView '%s'", this.name);
            this.unbind();
            this.undelegateEvents();
        },
    });

    // FIXME: unused?
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
        close: function() {
            this.unbind();
            this.undelegateEvents();
        },
	});
}

