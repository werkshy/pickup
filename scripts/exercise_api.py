#!/usr/bin/env python3
"""
Exercise a running Pickup server: browse the collection, play tracks,
change the volume, queue and skip tracks, replace the queue, and stop.
Lets each track play for a few seconds, so it takes about half a minute.

Usage:
    python3 scripts/exercise_api.py [base_url]      (default: http://localhost:9090)

Start the server first (from the rs directory):
    cargo run -- serve
"""

import json
import sys
import time
import urllib.error
import urllib.request

BASE = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:9090"

# How long to let a track actually play before the next action.
PLAY_SECS = 7


def listen():
    """Let the current track play for a few seconds."""
    print(f"  ...listening for {PLAY_SECS}s")
    time.sleep(PLAY_SECS)


def req(method, path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    headers = {"Content-Type": "application/json"} if body is not None else {}
    request = urllib.request.Request(BASE + path, method=method, data=data, headers=headers)
    try:
        with urllib.request.urlopen(request) as resp:
            return resp.status, json.load(resp)
    except urllib.error.HTTPError as error:
        return error.code, json.load(error)
    except urllib.error.URLError as error:
        raise SystemExit(f"cannot reach {BASE}: {error}\n(is the server running? cargo run -- serve)")


def step(label):
    print(f"\n== {label}")


def find_track(title, folder_id=None):
    """Find a track by (partial) title, anywhere in the collection."""
    if folder_id is None:
        for category in req("GET", "/categories")[1]["categories"]:
            hit = find_track(title, category["id"])
            if hit:
                return hit
        return None
    _, folder = req("GET", f"/folders/{folder_id}")
    for track in folder.get("tracks", []):
        if title.lower() in track["title"].lower():
            return track
    for sub in folder.get("folders", []):
        hit = find_track(title, sub["id"])
        if hit:
            return hit
    return None


def find_folder(kind, name, folder_id=None):
    """Find a folder by kind and (partial) name, anywhere in the collection."""
    if folder_id is None:
        for category in req("GET", "/categories")[1]["categories"]:
            hit = find_folder(kind, name, category["id"])
            if hit:
                return hit
        return None
    _, folder = req("GET", f"/folders/{folder_id}")
    for sub in folder.get("folders", []):
        if sub["kind"] == kind and name.lower() in sub["name"].lower():
            return sub
        hit = find_folder(kind, name, sub["id"])
        if hit:
            return hit
    return None


step("server info")
print(req("GET", "/")[1])

step("list categories")
for category in req("GET", "/categories")[1]["categories"]:
    print(f"  {category['id']}  {category['name']}")

step("find some tracks")
first = find_track("Finally See The Light")
second = find_track("Neptunian Princess")
assert first and second, "expected the two Bryan Teoh tracks in the collection"
print(f"  first:  {first['id']}  {first['title']}")
print(f"  second: {second['id']}  {second['title']}")

step(f"play '{first['title']}'")
print(" ", req("POST", f"/play/{first['id']}")[1])
print(" ", req("GET", "/status")[1])
listen()

step("set volume to 60")
print(" ", req("POST", "/volume/60")[1])
print("  read back:", req("GET", "/volume")[1])

step(f"queue '{second['title']}' as next")
print(" ", req("POST", "/queue/add", {"id": second["id"], "position": "next"})[1])
print("  queue:", [t["title"] for t in req("GET", "/queue")[1]["tracks"]])

step("skip to the next track")
print(" ", req("POST", "/skip")[1])
queue = req("GET", "/queue")[1]["tracks"]
print("  now playing:", queue[0]["title"] if queue else "(nothing)")
print(" ", req("GET", "/status")[1])
listen()

step("replace the queue with the Free album and play it")
album = find_folder("album", "Free")
assert album, "expected a 'Free' album in the collection"
_, after_replace = req("POST", "/queue/add", {"id": album["id"], "position": "replace", "play_now": True})
print("  queue:", [t["title"] for t in after_replace["tracks"]])
print(" ", req("GET", "/status")[1])
listen()

step("skip again (album continues)")
print(" ", req("POST", "/skip")[1])
queue = req("GET", "/queue")[1]["tracks"]
print("  now playing:", queue[0]["title"] if queue else "(nothing)")
listen()

step("stop")
print(" ", req("POST", "/stop")[1])
print(" ", req("GET", "/status")[1])

print("\nDone.")
