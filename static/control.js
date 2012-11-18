
function initControls(App) {
	console.log("Setting up control classes")
	App.Control = Backbone.Model.extend({
		initialize: function() {
			console.log("Initializing artist");
			this.url = "/control"
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
			},
			initialize: function() {
				_.bindAll(this, "render")
				_.bindAll(this)
				this.model.bind("change", this.render)
				this.template = Handlebars.compile($("#control-template").html())
				this.render();
			},
			render: function() {
				console.log("Render control")
				this.$el.html(this.template(this.model.attributes))
				return this;
			},
			goHome: function () {
				console.log("Going to home page")
				App.router.navigate("artists", { 'trigger' : true});
			},
			postCommand: function(command) {
				$.postJSON("/control/", command,
						function() {
							console.log("Control success");
						}
				);
			},
			nextTrack: function() {
				console.log("Sending 'next' command");
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

function volumeChange(delta) {
	console.log("Changing volume by %d", delta);
	$.postJSON("/control/", {
				"command" : "volumeDelta",
				"volumeDelta" : delta,
			},
			function() {
				console.log("Volume change success!");
			}
	);
}

