// Browser-mode playback. The server keeps the session queue and serves
// track files (/track/{id}); this script owns the <audio> element: it
// decides what to play, and reports a finished or skipped track back via
// /session/queue/next so the server pops it off the queue.
//
// The player bar carries the current session track in its data attributes
// (data-track-id, data-title, data-artist, data-autoplay). Every server
// render - a full page load or an out-of-band swap after an action - tells
// the player what it should be playing through those; everything the server
// cannot know (paused, position, volume) is kept up to date by this script.
//
// Everything is delegated on document, so listeners keep working across
// htmx swaps.

// The browser player's element. It is attached to the (hidden) DOM so that
// devtools and tests can inspect it.
const audio = new Audio();
audio.hidden = true;

// The track id currently loaded in the audio element.
let currentTrackId = null;
// One advance at a time: the ended event and a skip click must not queue up.
let advancing = false;
// True while the user is dragging the seek slider, so time updates and the
// jukebox poll do not yank the thumb out from under them.
let seeking = false;
// Holds the thumb after a jukebox seek is released until the server answers.
let seekReleaseTimer = null;

function playerBar() {
    return document.getElementById("player-bar");
}

function browserMode() {
    const bar = playerBar();
    return bar !== null && bar.dataset.mode === "browser";
}

function trackUrl(id) {
    return "/track/" + encodeURIComponent(id);
}

function mmss(seconds) {
    const total = Math.max(0, Math.floor(seconds));
    return Math.floor(total / 60) + ":" + String(total % 60).padStart(2, "0");
}

// --- the bar's display ---

// Update the now-playing area and the bar's data attributes to a track.
// The autoplay flag is cleared once acted on, so later swaps (e.g. a queue
// add) never restart the player on their own.
function setNowPlaying(trackId, title, artist) {
    const bar = playerBar();
    if (bar) {
        bar.dataset.trackId = trackId || "";
        bar.dataset.title = title || "";
        bar.dataset.artist = artist || "";
        bar.dataset.autoplay = "false";
    }
    const nowPlaying = document.getElementById("now-playing");
    if (!nowPlaying) return;
    nowPlaying.replaceChildren();
    if (trackId) {
        const titleEl = document.createElement("span");
        titleEl.className = "np-title";
        titleEl.textContent = title;
        nowPlaying.append(titleEl);
        if (artist) {
            const artistEl = document.createElement("span");
            artistEl.className = "np-artist";
            artistEl.textContent = artist;
            nowPlaying.append(artistEl);
        }
    } else {
        const muted = document.createElement("span");
        muted.className = "muted";
        muted.textContent = "Nothing playing";
        nowPlaying.append(muted);
    }
}

function setPaused(paused) {
    const play = document.getElementById("transport-play");
    const pause = document.getElementById("transport-pause");
    if (play && pause) {
        play.hidden = !paused;
        pause.hidden = paused;
    }
}

function setProgress(current, duration) {
    const progress = document.getElementById("progress");
    if (progress && Number.isFinite(duration) && duration > 0) {
        progress.textContent = mmss(current) + " / " + mmss(duration);
    }
    setSeek(current, duration);
}

// Point the seek slider at a position/duration. A track with no known
// duration cannot be scrubbed yet.
function setSeek(current, duration) {
    const seek = document.getElementById("seek");
    if (!seek) return;
    if (!Number.isFinite(duration) || duration <= 0) {
        seek.disabled = true;
        seek.max = "0";
        if (!seeking) seek.value = "0";
        return;
    }
    seek.disabled = false;
    seek.max = String(duration);
    if (!seeking) {
        seek.value = String(Math.min(Math.max(0, current), duration));
    }
}

// Clear the readout and disable the slider (no track loaded).
function resetProgress() {
    const progress = document.getElementById("progress");
    if (progress) progress.textContent = "";
    const seek = document.getElementById("seek");
    if (seek) {
        seek.disabled = true;
        seek.max = "0";
        seek.value = "0";
    }
}

// In jukebox mode the server renders the position into #progress's data
// attributes on every poll; the slider itself is never swapped, so dragging
// it is never interrupted.
function syncJukeboxSeek() {
    if (browserMode()) return;
    const progress = document.getElementById("progress");
    const seek = document.getElementById("seek");
    if (!progress || !seek) return;
    const duration = Number(progress.dataset.duration);
    const position = Number(progress.dataset.position);
    if (
        progress.dataset.duration === undefined ||
        !Number.isFinite(duration) ||
        duration <= 0
    ) {
        seek.disabled = true;
        seek.max = "0";
        if (!seeking) seek.value = "0";
        return;
    }
    seek.disabled = false;
    seek.max = String(duration);
    if (!seeking) {
        const value = Number.isFinite(position)
            ? Math.min(Math.max(0, position), duration)
            : 0;
        seek.value = String(value);
    }
}

// The server always renders the slider at its default volume, so after a swap
// the slider has to be pointed back at what the audio element is really doing.
function setVolume() {
    const slider = document.getElementById("volume");
    if (slider) slider.value = String(Math.round(audio.volume * 100));
}

// --- playback ---

function playTrack(track, autoplay) {
    currentTrackId = track.id;
    audio.src = trackUrl(track.id);
    resetProgress();
    setNowPlaying(track.id, track.title, track.artist);
    if (autoplay) {
        audio.play().catch(() => {
            // Autoplay refusals (no user gesture yet) leave the track loaded
            // and paused; the play button takes it from there.
            setPaused(true);
        });
    } else {
        setPaused(true);
    }
}

// Finish the current track: pop it off the session queue and play the next
// one. Called when a track ends and when the skip button is pressed, so it
// always starts what it gets back.
async function advance() {
    if (advancing || !browserMode()) return;
    advancing = true;
    try {
        const response = await fetch("/session/queue/next", { method: "POST" });
        if (!response.ok) return;
        const body = await response.json();
        if (body.track) {
            playTrack(body.track, true);
        } else {
            // The queue has run out.
            currentTrackId = null;
            audio.pause();
            audio.removeAttribute("src");
            setNowPlaying(null, "", "");
            setPaused(true);
            resetProgress();
        }
        refreshQueueDrawer();
    } finally {
        advancing = false;
    }
}

// The queue's front just changed server-side, so the drawer's contents need
// a refresh. The fragment renders the session queue when the browser is in
// browser mode.
function refreshQueueDrawer() {
    if (window.htmx) {
        window.htmx.ajax("GET", "/fragments/queue", { target: "#queue-list", swap: "outerHTML" })
            .then(updateQueueHighlight);
    }
}

// The server cannot know whether the browser is sounding the front track,
// so the drawer never marks anything as playing; this does, from the audio
// element's real state. "Playing" means sounding now, or paused part-way
// through (the same open-lifecycle the jukebox drawer highlights).
function updateQueueHighlight() {
    const current = document.querySelector("#queue-list li");
    if (!current || current.classList.contains("queue-empty")) return;
    const active = currentTrackId !== null &&
        (!audio.paused || (audio.currentTime > 0 && !audio.ended));
    current.classList.toggle("playing", active);
}

// Align the player with what the server last rendered into the bar: a page
// load (resume at the front of the session queue, paused) or an out-of-band
// bar swap after an action.
function syncFromBar() {
    const bar = playerBar();
    if (!bar) return;
    if (bar.dataset.mode !== "browser") {
        // Switched to jukebox mode: the browser player stands down (the
        // session queue is kept server-side, so switching back resumes).
        audio.pause();
        return;
    }

    const id = bar.dataset.trackId || "";
    const wasPlaying = currentTrackId !== null && !audio.paused && !audio.ended;

    if (!id) {
        // The session queue is empty: nothing to be playing.
        audio.pause();
        audio.removeAttribute("src");
        currentTrackId = null;
        setNowPlaying(null, "", "");
        setPaused(true);
        resetProgress();
        return;
    }

    if (id !== currentTrackId) {
        // A new front track. Start it when the action said so, or when its
        // predecessor was sounding (removing/skipping the playing track).
        const autoplay = bar.dataset.autoplay === "true" || wasPlaying;
        playTrack(
            { id: id, title: bar.dataset.title || "", artist: bar.dataset.artist || "" },
            autoplay,
        );
    } else {
        // Same track: only the display may need refreshing after the swap.
        const restart = bar.dataset.autoplay === "true";
        setNowPlaying(id, bar.dataset.title || "", bar.dataset.artist || "");
        if (restart) {
            audio.currentTime = 0;
            audio.play().catch(() => setPaused(true));
        }
    }

    // Every server render puts the transport back to its default (play
    // visible, empty progress, default volume). Re-apply the audio element's
    // real state so an out-of-band swap never desyncs the bar.
    setPaused(audio.paused);
    setProgress(audio.currentTime, audio.duration);
    setVolume();
}

// --- wiring ---

document.addEventListener("click", (event) => {
    if (!browserMode()) return;
    const bar = playerBar();
    if (event.target.closest("#transport-play")) {
        if (currentTrackId) {
            audio.play();
        } else if (bar && bar.dataset.trackId) {
            // Nothing loaded yet (e.g. a queue add while stopped): start the
            // front of the session queue.
            playTrack(
                { id: bar.dataset.trackId, title: bar.dataset.title || "", artist: bar.dataset.artist || "" },
                true,
            );
        }
    }
    if (event.target.closest("#transport-pause")) audio.pause();
    if (event.target.closest("#transport-stop")) {
        audio.pause();
        audio.currentTime = 0;
    }
    if (event.target.closest("#transport-skip")) advance();
});

document.addEventListener("input", (event) => {
    if (event.target.id === "seek") {
        seeking = true;
        if (!browserMode()) return; // jukebox seeks when the slider is released
        const duration = audio.duration;
        if (Number.isFinite(duration) && duration > 0) {
            audio.currentTime = Number(event.target.value);
            setProgress(audio.currentTime, duration);
        }
        return;
    }
    if (!browserMode()) return;
    if (event.target.id === "volume") {
        audio.volume = Number(event.target.value) / 100;
        try {
            localStorage.setItem("pickup_volume", event.target.value);
        } catch {
            // Private browsing etc: the volume just does not persist.
        }
    }
});

document.addEventListener("change", (event) => {
    if (event.target.id !== "seek") return;
    if (browserMode()) {
        seeking = false;
        const duration = audio.duration;
        if (Number.isFinite(duration) && duration > 0) {
            audio.currentTime = Number(event.target.value);
            setProgress(audio.currentTime, duration);
        }
        return;
    }
    // The jukebox slider's hx-post has fired. Keep the thumb where the user
    // left it until the server's response (or the next poll) catches up.
    clearTimeout(seekReleaseTimer);
    seekReleaseTimer = setTimeout(() => {
        seeking = false;
        syncJukeboxSeek();
    }, 750);
});

audio.addEventListener("play", () => { setPaused(false); updateQueueHighlight(); });
audio.addEventListener("pause", () => { setPaused(true); updateQueueHighlight(); });
audio.addEventListener("timeupdate", () => setProgress(audio.currentTime, audio.duration));
audio.addEventListener("loadedmetadata", () => setProgress(audio.currentTime, audio.duration));
audio.addEventListener("durationchange", () => setProgress(audio.currentTime, audio.duration));
audio.addEventListener("ended", () => advance());

// Restore the volume the browser chose last time, and sync with the bar the
// server rendered.
(function init() {
    document.body.append(audio);
    try {
        const saved = localStorage.getItem("pickup_volume");
        if (saved !== null) {
            audio.volume = Number(saved) / 100;
            const slider = document.getElementById("volume");
            if (slider) slider.value = saved;
        }
    } catch {
        // No storage access: the default volume applies.
    }
    syncFromBar();
    syncJukeboxSeek();
    document.body.addEventListener("htmx:afterSwap", syncJukeboxSeek);
    document.body.addEventListener("htmx:oobAfterSwap", () => {
        syncFromBar();
        syncJukeboxSeek();
        // The drawer may have been swapped out-of-band in the same response.
        updateQueueHighlight();
    });
})();
