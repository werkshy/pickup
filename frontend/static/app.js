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
