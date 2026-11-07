/**
 * View models for the web UI templates. These exist to keep the templates
 * dumb: strings to render, booleans to branch on, no logic beyond loops.
 */
use std::path::{Path, PathBuf};

use askama::Template;

use std::time::Duration;

use crate::api::types::kind_name;
use crate::app_state::AppState;
use crate::filemanager::collection_builder::DEFAULT_CATEGORY;
use crate::filemanager::model::{Folder, Kind};
use crate::filemanager::utils::{
    generate_album_id, generate_artist_id, generate_category_id, generate_disc_id,
};
use crate::player::PlaybackStatus;

/**
 * The shared chrome of every page: title, category tabs, the queue drawer's
 * contents, and the player bar. `oob` marks the queue list and player bar as
 * out-of-band swap targets for action responses, so the same templates render
 * both inline (a full page) and as OOB updates (an action response).
 */
pub struct Shell {
    pub title: String,
    pub categories: Vec<CategoryTab>,
    /// ID of the category tab to highlight; empty on the home page.
    pub active_category: String,
    pub queue: QueueView,
    pub player_bar: PlayerBarView,
    pub oob: bool,
}

/**
 * One consistent snapshot of the player: its playback status and the current
 * volume. Built once per render and shared by every view that needs player
 * state, so a page render costs a fixed pair of requests to the player
 * thread instead of one per component.
 */
pub struct PlayerSnapshot {
    pub status: Option<PlaybackStatus>,
    pub volume: u8,
}

/**
 * Snapshot the player for the views. The status can fail when the player
 * thread is wedged: the views then render as if nothing is playing, which is
 * the least-wrong thing to show. The volume always answers.
 */
pub async fn player_snapshot(state: &AppState) -> PlayerSnapshot {
    let status = state.player.status().await.ok();
    let volume = state
        .player
        .request_async(|player| player.get_volume())
        .await
        .unwrap_or(100);
    PlayerSnapshot { status, volume }
}

impl Shell {
    /**
     * Build the shell for a full page render: category tabs from the
     * collection, plus the queue drawer and player bar from one player
     * snapshot. Async because the player state lives on the Player thread.
     */
    pub async fn for_page(state: &AppState, title: &str, active_category: &str) -> Shell {
        let snapshot = player_snapshot(state).await;
        Shell::from_snapshot(state, title, active_category, &snapshot, false)
    }

    /**
     * Build the shell for an action response: only the live chrome (queue
     * list, player bar) matters, the rest of the page is never swapped, and
     * `oob` tells htmx to replace those regions out-of-band.
     */
    pub async fn for_action(state: &AppState) -> Shell {
        let snapshot = player_snapshot(state).await;
        Shell::from_snapshot(state, "", "", &snapshot, true)
    }

    fn from_snapshot(
        state: &AppState,
        title: &str,
        active_category: &str,
        snapshot: &PlayerSnapshot,
        oob: bool,
    ) -> Shell {
        Shell {
            title: title.to_string(),
            categories: state
                .collection
                .values()
                .map(|folder| CategoryTab {
                    id: folder.id.clone(),
                    name: folder.name.clone(),
                })
                .collect(),
            active_category: active_category.to_string(),
            queue: QueueView::from_snapshot(state, snapshot),
            player_bar: PlayerBarView::from_snapshot(state, snapshot),
            oob,
        }
    }

    /**
     * Build the shell for a direct swap of the queue list (not out-of-band).
     * Only the queue is rendered from this shell, so the player bar is left
     * empty.
     */
    pub fn for_fragment(queue: QueueView) -> Shell {
        Shell {
            title: String::new(),
            categories: Vec::new(),
            active_category: String::new(),
            queue,
            player_bar: PlayerBarView::default(),
            oob: false,
        }
    }
}

pub struct CategoryTab {
    pub id: String,
    pub name: String,
}

pub struct QueueView {
    pub tracks: Vec<QueueTrackRow>,
    pub empty: bool,
}

pub struct QueueTrackRow {
    /// Position in the queue, 0-based; 0 is the track playing right now
    /// (or the next one when idle). The drawer renders 1-based.
    pub index: usize,
    pub title: String,
    pub artist: String,
    pub playing: bool,
}

impl QueueView {
    /**
     * Snapshot the queue for display. The front of the queue is the track
     * playing right now when the player is active; when idle it is the next
     * track to play, so nothing is highlighted.
     */
    pub async fn build(data: &AppState) -> QueueView {
        let snapshot = player_snapshot(data).await;
        QueueView::from_snapshot(data, &snapshot)
    }

    fn from_snapshot(data: &AppState, snapshot: &PlayerSnapshot) -> QueueView {
        let playing = snapshot
            .status
            .as_ref()
            .map(|status| status.active)
            .unwrap_or(false);

        let queue = data.queue.read().unwrap();
        let tracks: Vec<QueueTrackRow> = queue
            .tracks
            .iter()
            .enumerate()
            .map(|(index, track)| QueueTrackRow {
                index,
                title: track.name.clone(),
                artist: track.artist.clone().unwrap_or_default(),
                playing: playing && index == 0,
            })
            .collect();
        let empty = tracks.is_empty();
        QueueView { tracks, empty }
    }
}

/**
 * The bottom player bar: what is playing, the transport state, progress and
 * volume. Rendered as part of every page (it sits in the shell, outside the
 * swapped content), as an out-of-band swap in every action response, and -
 * just the progress readout - polled every second.
 */
#[derive(Debug, Clone, Default)]
pub struct PlayerBarView {
    /// The track playing right now; empty strings when idle.
    pub track_title: String,
    pub track_artist: String,
    /// A track's playback lifecycle is open (playing or paused). When idle
    /// the bar shows nothing instead.
    pub active: bool,
    pub paused: bool,
    /// Elapsed time in the current track, rendered as `M:SS`; empty when
    /// unknown or idle.
    pub elapsed: String,
    /// Total duration of the current track, rendered as `M:SS`; empty when
    /// unknown or idle.
    pub duration: String,
    /// Whether the progress readout has anything to show. There is a
    /// progress element in the bar at all times (the poll swaps it), but it
    /// is empty while idle.
    pub show_progress: bool,
    pub volume: u8,
}

impl PlayerBarView {
    /**
     * The bar's view from a player snapshot: the current track comes from
     * the front of the queue (the player's own idea of "the track" is just
     * a file path), the progress and paused state from the status.
     */
    pub fn from_snapshot(data: &AppState, snapshot: &PlayerSnapshot) -> PlayerBarView {
        let status = snapshot.status.as_ref();
        let active = status.map(|status| status.active).unwrap_or(false);

        let track = if active {
            data.queue
                .read()
                .ok()
                .and_then(|queue| queue.tracks.front().cloned())
        } else {
            None
        };

        let (elapsed, duration, show_progress) = match status {
            Some(status) if status.active => (
                status.position.as_ref().map(mmss).unwrap_or_default(),
                status.duration.as_ref().map(mmss).unwrap_or_default(),
                status.duration.is_some(),
            ),
            _ => (String::new(), String::new(), false),
        };

        PlayerBarView {
            track_title: track.as_ref().map(|t| t.name.clone()).unwrap_or_default(),
            track_artist: track
                .as_ref()
                .and_then(|t| t.artist.clone())
                .unwrap_or_default(),
            active,
            paused: status.map(|status| status.paused).unwrap_or(false),
            elapsed,
            duration,
            show_progress,
            volume: snapshot.volume,
        }
    }
}

/**
 * Render a duration as `M:SS`, the way players show it (no leading zero on
 * the minutes, tracks longer than an hour get more minutes).
 */
fn mmss(duration: &Duration) -> String {
    let total_seconds = duration.as_secs();
    format!("{}:{:02}", total_seconds / 60, total_seconds % 60)
}

/**
 * A browsable folder page: breadcrumbs, heading, the folders one level
 * below, and the tracks directly inside.
 */
pub struct FolderPageView {
    pub crumbs: Vec<Crumb>,
    pub heading: String,
    pub kind: String,
    pub sub_kind_label: String,
    pub folders: Vec<FolderLink>,
    pub folders_empty: bool,
    pub tracks: Vec<TrackRow>,
    pub tracks_empty: bool,
    /// Category tab that should be highlighted on this page.
    pub active_category: String,
}

impl FolderPageView {
    pub fn from_folder(folder: &Folder) -> FolderPageView {
        FolderPageView {
            crumbs: breadcrumbs(folder),
            heading: folder.name.clone(),
            kind: kind_name(folder.kind).to_string(),
            sub_kind_label: sub_kind_label(folder.kind).to_string(),
            folders: folder
                .folders
                .values()
                .map(|child| FolderLink {
                    id: child.id.clone(),
                    name: child.name.clone(),
                    kind: kind_name(child.kind).to_string(),
                })
                .collect(),
            folders_empty: folder.folders.is_empty(),
            tracks: folder
                .tracks
                .iter()
                .map(|track| TrackRow {
                    id: track.id.clone(),
                    title: track.name.clone(),
                    artist: track.artist.clone().unwrap_or_default(),
                })
                .collect(),
            tracks_empty: folder.tracks.is_empty(),
            active_category: active_category(folder),
        }
    }
}

pub struct Crumb {
    pub id: String,
    pub name: String,
}

pub struct FolderLink {
    pub id: String,
    pub name: String,
    pub kind: String,
}

pub struct TrackRow {
    pub id: String,
    pub title: String,
    pub artist: String,
}

/**
 * Breadcrumbs from the top of the collection down to `folder`. Each crumb
 * links to the folder of that level.
 *
 * The chain is derived from the folder's path, which mirrors how the
 * collection is built: a leading `_Category` directory at the top (the
 * implicit default category contributes no path component), then artist,
 * album, disc, one level each. The IDs are computed from the path prefixes
 * exactly like the builder does, so they always match the folders it
 * created.
 */
fn breadcrumbs(folder: &Folder) -> Vec<Crumb> {
    let components: Vec<String> = folder
        .path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect();

    let mut crumbs: Vec<Crumb> = Vec::new();
    let mut depth = 0;
    if folder.kind != Kind::Category && components[0].starts_with('_') {
        crumbs.push(Crumb {
            id: generate_category_id(Path::new(&components[0])),
            name: category_display(&components[0]),
        });
        depth = 1;
    }

    if matches!(folder.kind, Kind::Artist | Kind::Album | Kind::Disc) {
        crumbs.push(Crumb {
            id: generate_artist_id(&path_to(&components[..depth + 1])),
            name: components[depth].clone(),
        });
        depth += 1;
    }
    if matches!(folder.kind, Kind::Album | Kind::Disc) {
        crumbs.push(Crumb {
            id: generate_album_id(&path_to(&components[..depth + 1])),
            name: components[depth].clone(),
        });
        depth += 1;
    }
    if folder.kind == Kind::Disc {
        crumbs.push(Crumb {
            id: generate_disc_id(&path_to(&components[..depth + 1])),
            name: components[depth].clone(),
        });
    }

    // The display name of the folder itself is the tree's (e.g. a category
    // name without its underscore), so correct the last crumb.
    if let Some(last) = crumbs.last_mut() {
        last.name = folder.name.clone();
    }
    crumbs
}

fn path_to(components: &[String]) -> PathBuf {
    components.iter().collect()
}

fn category_display(directory: &str) -> String {
    directory.strip_prefix('_').unwrap_or(directory).to_string()
}

/**
 * The category tab to highlight while browsing a folder: the folder itself
 * for a category; otherwise the category its path hangs off (the explicit
 * `_Category` directory, or the implicit default).
 */
fn active_category(folder: &Folder) -> String {
    if folder.kind == Kind::Category {
        return folder.id.clone();
    }
    let first = folder
        .path
        .components()
        .next()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .unwrap_or_default();
    if first.starts_with('_') {
        generate_category_id(Path::new(&first))
    } else {
        generate_category_id(Path::new(DEFAULT_CATEGORY))
    }
}

/**
 * What the folders one level below a folder of this kind are called.
 */
fn sub_kind_label(kind: Kind) -> &'static str {
    match kind {
        Kind::Category => "Artists",
        Kind::Artist => "Albums",
        Kind::Album => "Discs",
        Kind::Disc | Kind::Track => "",
    }
}

// ---- templates ----

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub shell: Shell,
}

#[derive(Template)]
#[template(path = "pages/folder.html")]
pub struct FolderTemplate {
    pub shell: Shell,
    pub page: FolderPageView,
}

/**
 * The error page for browser navigation that lands somewhere with no
 * folder: rendered with the full shell so it works both as a direct page
 * load and as a boosted swap.
 */
#[derive(Template)]
#[template(path = "pages/not_found.html")]
pub struct NotFoundTemplate {
    pub shell: Shell,
    pub path: String,
}

/**
 * The queue drawer's list on its own, for action responses (out-of-band
 * swap) and for direct swaps into the drawer.
 */
#[derive(Template)]
#[template(path = "fragments/queue.html")]
pub struct QueueTemplate {
    pub shell: Shell,
}

/**
 * An action response: the player bar and the queue drawer together, both as
 * out-of-band swaps. Every transport/queue action re-renders both, so the
 * live chrome never goes stale after a click.
 */
#[derive(Template)]
#[template(path = "fragments/action.html")]
pub struct ActionTemplate {
    pub shell: Shell,
}

/**
 * The progress readout on its own: the one thing the one-second poll swaps.
 * Small on purpose - the poll response is a few dozen bytes.
 */
#[derive(Template)]
#[template(path = "fragments/progress.html")]
pub struct ProgressTemplate {
    pub bar: PlayerBarView,
}
