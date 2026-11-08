const { test, expect } = require("@playwright/test");

/**
 * Walk the collection via the JSON API for a folder with tracks, preferring
 * one with at least two so the advance/skip flows can be exercised. Used to
 * navigate the UI to a page with playable tracks.
 */
async function findFolderWithTracks(request) {
  const categories = await (await request.get("/categories")).json();
  let frontier = categories.categories.map((folder) => folder.id);
  let anyFolder = null;
  while (frontier.length > 0) {
    const id = frontier.shift();
    const folder = await (await request.get(`/folders/${id}`)).json();
    if (folder.tracks.length >= 2) return folder;
    if (folder.tracks.length > 0 && !anyFolder) anyFolder = folder;
    frontier.push(...folder.folders.map((child) => child.id));
  }
  if (anyFolder) return anyFolder;
  throw new Error("no folder with tracks in the test collection");
}

/**
 * The row action buttons (add/next) are hover-revealed with CSS
 * `visibility: hidden`; hovering the row first is what makes them
 * clickable.
 */
async function addTrackRow(page, index = 0) {
  const row = page.locator(".track-list li").nth(index);
  await row.hover();
  await row.locator(".action-add").click();
}

function audioState(page) {
  return page.evaluate("(() => { const a = document.querySelector('audio'); return a ? { paused: a.paused, currentTime: a.currentTime } : null; })()");
}

test.describe("browser playback", () => {
  test("the default mode is browser playback", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#player-bar")).toHaveAttribute("data-mode", "browser");
    await expect(page.locator("#mode-browser")).toHaveClass(/active/);
    await expect(page.locator("#mode-jukebox")).not.toHaveClass(/active/);
    await expect(page.locator("#transport-play")).toBeVisible();
  });

  test("playing a track sounds it in the browser", async ({ page }) => {
    const folder = await findFolderWithTracks(page.request);
    const track = folder.tracks[0];

    await page.goto(`/browse/${folder.id}`);
    await page.locator(".track-title", { hasText: track.title }).first().click();

    // The bar shows the track, and the audio element is really playing:
    // its position advances.
    await expect(page.locator("#now-playing .np-title")).toHaveText(track.title);
    await expect
      .poll(async () =>
        page.evaluate("document.querySelector('audio')?.currentTime ?? -1"),
      )
      .toBeGreaterThan(0.5);
    await expect(page.locator("#transport-pause")).toBeVisible();
    // The drawer marks the current track as playing.
    await expect(page.locator("#queue-list li.playing .queue-title")).toHaveText(
      track.title,
    );
  });

  test("the track row play button plays the track now", async ({ page }) => {
    const folder = await findFolderWithTracks(page.request);
    const track = folder.tracks[0];

    await page.goto(`/browse/${folder.id}`);
    const row = page.locator(".track-list li").first();
    await row.hover();
    await row.locator(".action-play").click();

    // The bar shows the track and the audio element is really playing.
    await expect(page.locator("#now-playing .np-title")).toHaveText(track.title);
    await expect
      .poll(async () =>
        page.evaluate("document.querySelector('audio')?.currentTime ?? -1"),
      )
      .toBeGreaterThan(0.5);
    await expect(page.locator("#transport-pause")).toBeVisible();
  });

  test("add queues without playing; the play button starts the front", async ({
    page,
  }) => {
    const folder = await findFolderWithTracks(page.request);
    const first = folder.tracks[0];

    await page.goto(`/browse/${folder.id}`);
    await addTrackRow(page, 0);

    // Queued, not playing: the bar names the track but the transport shows
    // the play button, and the audio element is silent and idle.
    await expect(page.locator("#now-playing .np-title")).toHaveText(first.title);
    await expect(page.locator("#transport-play")).toBeVisible();
    await expect
      .poll(async () =>
        page.evaluate("document.querySelector('audio')?.currentTime ?? -1"),
      )
      .toBe(0);

    await page.locator("#transport-play").click();
    await expect(page.locator("#transport-pause")).toBeVisible();
    await expect
      .poll(async () =>
        page.evaluate("document.querySelector('audio')?.currentTime ?? -1"),
      )
      .toBeGreaterThan(0.5);
  });

  test("a finished track advances to the next one", async ({ page }) => {
    const folder = await findFolderWithTracks(page.request);
    const [first, second] = folder.tracks;
    test.skip(!second, "needs an album with two tracks");

    await page.goto(`/browse/${folder.id}`);
    // Queue both, then start the front from the transport.
    await addTrackRow(page, 0);
    await addTrackRow(page, 1);
    await page.locator("#transport-play").click();
    await expect(page.locator("#now-playing .np-title")).toHaveText(first.title);

    // Seek to the very end of the track; the ended event must advance the
    // session queue and start the next track.
    await page.evaluate(
      "(() => { const audio = document.querySelector('audio'); audio.currentTime = audio.duration - 0.2; })()",
    );
    await expect(page.locator("#now-playing .np-title")).toHaveText(second.title, {
      timeout: 10_000,
    });
    await expect
      .poll(async () =>
        page.evaluate("document.querySelector('audio')?.currentTime ?? -1"),
      )
      .toBeGreaterThan(0.2);
    // The drawer now highlights the second track.
    await expect(page.locator("#queue-list li.playing .queue-title")).toHaveText(
      second.title,
    );
  });

  test("skip advances to the next track", async ({ page }) => {
    const folder = await findFolderWithTracks(page.request);
    const [first, second] = folder.tracks;
    test.skip(!second, "needs an album with two tracks");

    await page.goto(`/browse/${folder.id}`);
    await addTrackRow(page, 0);
    await addTrackRow(page, 1);
    await page.locator("#transport-play").click();
    await expect(page.locator("#now-playing .np-title")).toHaveText(first.title);

    await page.locator("#transport-skip").click();
    await expect(page.locator("#now-playing .np-title")).toHaveText(second.title);
    await expect
      .poll(async () =>
        page.evaluate("document.querySelector('audio')?.currentTime ?? -1"),
      )
      .toBeGreaterThan(0.2);
  });

  test("clear stops the player and empties the queue", async ({ page }) => {
    const folder = await findFolderWithTracks(page.request);

    await page.goto(`/browse/${folder.id}`);
    await addTrackRow(page, 0);
    await page.locator("#transport-play").click();
    await expect
      .poll(async () =>
        page.evaluate("document.querySelector('audio')?.currentTime ?? -1"),
      )
      .toBeGreaterThan(0.2);

    // The drawer is off-canvas until it is opened.
    await page.locator("#drawer-toggle").click();
    await page.locator("#queue-clear").click();
    await expect(page.locator("#queue-list .queue-empty")).toHaveText(
      "The queue is empty",
    );
    await expect(page.locator("#now-playing")).toContainText("Nothing playing");
    await expect
      .poll(async () => page.evaluate("document.querySelector('audio')?.paused"))
      .toBe(true);
  });

  test("adding a track while playing keeps the bar showing pause", async ({
    page,
  }) => {
    const folder = await findFolderWithTracks(page.request);
    const [first, second] = folder.tracks;
    test.skip(!second, "needs an album with two tracks");

    await page.goto(`/browse/${folder.id}`);
    await addTrackRow(page, 0);
    await page.locator("#transport-play").click();
    await expect(page.locator("#transport-pause")).toBeVisible();

    // A queue add swaps the bar out-of-band; the re-rendered transport must
    // still match the audio element, which is still sounding the front track.
    await addTrackRow(page, 1);
    await expect(page.locator("#now-playing .np-title")).toHaveText(first.title);
    await expect(page.locator("#transport-pause")).toBeVisible();
    await expect(page.locator("#transport-play")).toBeHidden();
    expect((await audioState(page)).paused).toBe(false);
  });

  test("the volume slider drives the browser player", async ({ page }) => {
    const folder = await findFolderWithTracks(page.request);
    test.skip(folder.tracks.length < 2, "needs an album with two tracks");

    await page.goto(`/browse/${folder.id}`);
    await addTrackRow(page, 0);
    await page.locator("#transport-play").click();
    await expect(page.locator("#transport-pause")).toBeVisible();

    await page.locator("#volume").fill("42");
    await expect
      .poll(async () => page.evaluate("document.querySelector('audio')?.volume"))
      .toBeCloseTo(0.42);

    // A bar swap re-renders the slider at its default; it must be pointed
    // back at the volume the audio element is really using.
    await addTrackRow(page, 1);
    await expect(page.locator("#volume")).toHaveValue("42");
    expect(await page.evaluate("document.querySelector('audio')?.volume")).toBeCloseTo(
      0.42,
    );
  });

  test("toggling modes switches the bar and stops the other player", async ({
    page,
  }) => {
    const folder = await findFolderWithTracks(page.request);

    await page.goto(`/browse/${folder.id}`);
    await addTrackRow(page, 0);
    await page.locator("#transport-play").click();
    await expect(page.locator("#transport-pause")).toBeVisible();

    // Switch to jukebox: the bar becomes the server-driven flavor and the
    // browser player stands down.
    await page.locator("#mode-jukebox").click();
    await expect(page.locator("#player-bar")).toHaveAttribute("data-mode", "jukebox");
    await expect(page.locator("#mode-jukebox")).toHaveClass(/active/);
    await expect
      .poll(async () => page.evaluate("document.querySelector('audio')?.paused"))
      .toBe(true);

    // Switch back: the session queue is still there, loaded and paused.
    await page.locator("#mode-browser").click();
    await expect(page.locator("#player-bar")).toHaveAttribute("data-mode", "browser");
    await expect(page.locator("#mode-browser")).toHaveClass(/active/);
    await expect(page.locator("#transport-play")).toBeVisible();
    await expect(page.locator("#now-playing .np-title")).toHaveText(
      folder.tracks[0].title,
    );
  });
});
