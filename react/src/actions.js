
export function playAlbum(category, artist, album, immediate) {
  console.log("Playing album %s/%s (%s)", artist, album, immediate)
  let data = {
    "Command": "add",
    "Category": category,
    "Artist": artist,
    "Album": album,
    "Immediate": immediate
  }
  fetch("/api/playlist/", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  })
  .then(response => {
    console.log("Play success", response.status)
  })
}

export function playTrack(category, artist, album, track, immediate) {
  console.log("Playing track %s/%s/%s (%s)", artist, album, track, immediate)
  let data = {
    "Command": "add",
    "Category": category,
    "Artist": artist,
    "Album": album,
    "Track" : track,
    "Immediate": immediate
  }
  fetch("/api/playlist/", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  })
  .then(response => {
    console.log("Play success", response.status)
  })
}
