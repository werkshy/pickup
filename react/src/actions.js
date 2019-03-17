
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
