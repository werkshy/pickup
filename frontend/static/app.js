// Pickup web UI glue. htmx does the swapping; this only handles bits that
// are pure client state. Delegated listeners on document, so they keep
// working across htmx swaps.

// The queue drawer is opened and closed purely in the browser: the server
// never needs to know whether it is visible.
document.addEventListener("click", (event) => {
    if (event.target.closest("#drawer-toggle")) {
        document.body.classList.toggle("queue-open");
    }
    if (event.target.closest("#drawer-close")) {
        document.body.classList.remove("queue-open");
    }
});

// Queue drag-and-drop reordering. Rows carry their server-side queue index
// in data-index; a drop posts to the same /web/queue/move action as the other
// queue changes, so the drawer and player bar are re-rendered out-of-band.
// The playing track is locked at the front: it cannot be dragged, and nothing
// can be dropped ahead of it.
let draggingIndex = null;

function queueRow(target) {
    if (!(target instanceof Element)) return null;
    return target.closest("#queue-list li[data-index]");
}

function playingQueueIndex() {
    const playing = document.querySelector("#queue-list li.playing[data-index]");
    return playing ? Number(playing.dataset.index) : null;
}

function moveToIndex(from, target, after) {
    let to = Number(target.dataset.index) + (after ? 1 : 0);
    if (from < to) to -= 1;
    return to;
}

function dropAllowed(to) {
    const playing = playingQueueIndex();
    return playing === null || to > playing;
}

function clearDropMarks() {
    for (const row of document.querySelectorAll(
        "#queue-list li.drop-before, #queue-list li.drop-after",
    )) {
        row.classList.remove("drop-before", "drop-after");
    }
}

document.addEventListener("dragstart", (event) => {
    const row = queueRow(event.target);
    if (!row) return;
    const from = Number(row.dataset.index);
    const playing = playingQueueIndex();
    if (playing !== null && from === playing) {
        event.preventDefault();
        return;
    }
    draggingIndex = from;
    row.classList.add("dragging");
    if (event.dataTransfer) {
        event.dataTransfer.effectAllowed = "move";
        event.dataTransfer.setData("text/plain", String(from));
    }
});

document.addEventListener("dragover", (event) => {
    if (draggingIndex === null) return;
    const row = queueRow(event.target);
    if (!row) {
        clearDropMarks();
        return;
    }
    const rect = row.getBoundingClientRect();
    const after = event.clientY > rect.top + rect.height / 2;
    const to = moveToIndex(draggingIndex, row, after);
    clearDropMarks();
    if (!dropAllowed(to) || to === draggingIndex) {
        if (event.dataTransfer) event.dataTransfer.dropEffect = "none";
        return;
    }
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    row.classList.add(after ? "drop-after" : "drop-before");
});

document.addEventListener("drop", (event) => {
    if (draggingIndex === null) return;
    const row = queueRow(event.target);
    if (!row) {
        clearDropMarks();
        return;
    }
    const rect = row.getBoundingClientRect();
    const after = event.clientY > rect.top + rect.height / 2;
    const from = draggingIndex;
    const to = moveToIndex(from, row, after);
    clearDropMarks();
    if (!dropAllowed(to) || to === from) return;
    event.preventDefault();
    draggingIndex = null;
    if (window.htmx) {
        window.htmx.ajax("POST", `/web/queue/move/${from}/${to}`, { swap: "none" });
    }
});

document.addEventListener("dragend", () => {
    draggingIndex = null;
    clearDropMarks();
    for (const row of document.querySelectorAll("#queue-list li.dragging")) {
        row.classList.remove("dragging");
    }
});
