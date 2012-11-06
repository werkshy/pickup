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

/**
 * Wire up the controls div
 * Called from routes.js
 */
function initControls() {
	$("span#go-home").click(function() {
		App.router.navigate("artists",
				{ 'trigger' : true});
	});

	$("#controls #next-track").click(function() {
		console.log("Sending 'next' command");
		$.postJSON("/control/", {
					"command" : "next",
				},
				function() {
					console.log("Next track success!");
				}
		);
	});
	$("#controls #prev-track").click(function() {
		console.log("Sending 'prev' command");
		$.postJSON("/control/", {
					"command" : "prev",
				},
				function() {
					console.log("Prev track success!");
				}
		);
	});
	$("#controls #stop").click(function() {
		console.log("Sending 'stop' command");
		$.postJSON("/control/", {
					"command" : "stop",
				},
				function() {
					console.log("Stop playing success!");
				}
		);
	});
	$("#controls #play").click(function() {
		console.log("Sending 'play' command");
		$.postJSON("/control/", {
					"command" : "play",
				},
				function() {
					console.log("Start playing success!");
				}
		);
	});
	$("#controls #pause").click(function() {
		console.log("Sending 'pause' command");
		$.postJSON("/control/", {
					"command" : "pause",
				},
				function() {
					console.log("pause playing success!");
				}
		);
	});
}
