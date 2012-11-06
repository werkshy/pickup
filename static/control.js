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
