
function initControls(App) {
	console.log("Setting up control classes")
	App.Control = Backbone.Model.extend({
		initialize: function() {
			console.log("Initializing artist");
			_.bindAll(this, "tick")
			this.url = "/control"
			setInterval(this.tick, 1000);
		},
		tick : function() {
			var elapsed = this.get("Elapsed")
			if (elapsed !== 'undefined' && this.get("State") === "play") {
				this.set("Elapsed", elapsed + 1 );
			}
		},
	});

	App.ControlView = Backbone.View.extend({
			// TODO: bind events
			el : "#controls",
			className : "controls",
			events : {
				"click span#go-home" : "goHome",
				"click #controls #next-track" : "nextTrack",
				"click #controls #prev-track" : "prevTrack",
				"click #controls #play" : "play",
				"click #controls #stop" : "stop",
				"click #controls #pause" : "pause",
				"click #controls #vol-up" : "volUp",
				"click #controls #vol-down" : "volDown",
				"click #controls .artist" : "gotoArtist",
			},
			initialize: function() {
				_.bindAll(this, "render")
				_.bindAll(this)
				this.model.bind("change", this.render)
				this.template = Handlebars.compile($("#control-template").html())
				this.render();
			},
			render: function() {
				this.$el.html(this.template(this.model.attributes))
				return this;
			},
			goHome: function () {
				console.log("Going to home page")
				App.router.navigate("artists", { 'trigger' : true});
			},
			postCommand: function(command) {
				console.log("Posting command: %s", command["command"])
				$.postJSON("/control/", command,
						function() {
							console.log("Control success");
						}
				);
				App.control.fetch()
			},
			nextTrack: function() {
				this.postCommand({"command" : "next"})
			},
			prevTrack: function() {
				this.postCommand({"command" : "prev"})
			},
			play: function() {
				this.postCommand({"command" : "play"})
			},
			stop: function() {
				this.postCommand({"command" : "stop"})
			},
			pause: function() {
				this.postCommand({"command" : "pause"})
			},
			volUp: function() {
				this.postCommand({
					"command" : "volumeDelta",
					"volumeDelta" : 5,
				})
			},
			volDown: function() {
				console.log("Sending volume down");
				this.postCommand({
					"command" : "volumeDelta",
					"volumeDelta" : -5,
				})
			},
			gotoArtist: function () {
				console.log("Going to see %s", this.model.get('CurrentArtist'))
				App.router.navigate("artists/" + this.model.get('CurrentArtist'),
					{ 'trigger' : true});
			}
	});

}

function playAlbum(artist, album, immediate) {
	console.log("Playing album %s/%s (%s)", artist, album, immediate);
	$.postJSON("/playlist/", {
				"command" : "add",
				"artist" : artist,
				"album" : album,
				"immediate" : immediate
			},
			function() {
				console.log("Play success!");
			}
	);
}

