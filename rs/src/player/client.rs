use std::error::Error;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use actix_web::web;

use crate::player::commands::QueryCommand;
use crate::player::{Command, Player};

/**
 * Default timeout for waiting on the Player thread to answer a query.
 * Generous on purpose: a query is also serialised behind any earlier commands
 * on the queue (e.g. a play() whose file open is slow). It exists to prevent
 * an infinite hang on a stuck Player thread, not to be tight.
 * Override with PlayerClient::with_timeout().
 */
const DEFAULT_RECV_TIMEOUT: Duration = Duration::from_secs(5);

/**
 * Why a request to the Player failed.
 */
#[derive(Debug)]
pub enum PlayerError {
    /// The Player thread is gone; no reply can ever arrive.
    Dead,
    /// The Player thread is alive but did not answer within the client's
    /// timeout.
    Timeout,
    /// The async request was aborted before the Player answered (the actix
    /// `web::block` call was cancelled or failed).
    Aborted,
    /// The Player thread processed the command, but the action itself failed
    /// (e.g. the file to play could not be opened or decoded).
    Failed(Box<dyn Error + Send + Sync>),
}

/**
 * Equality compares the error message of Failed; good enough for test
 * assertions (the underlying errors are not comparable).
 */
impl PartialEq for PlayerError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Failed(lhs), Self::Failed(rhs)) => lhs.to_string() == rhs.to_string(),
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

/**
 * A clonable handle to the Player thread. Every action (play, stop, set
 * volume) runs a closure on the Player thread and waits for the result, so
 * callers always know whether the action was delivered and answered.
 *
 * If the Player thread dies, there is no recovery in-process: every request
 * fails with PlayerError::Dead (surfaced as 503 by the API) until the server
 * is restarted.
 */
#[derive(Clone)]
pub struct PlayerClient {
    player_sender: Sender<Box<dyn Command>>,
    timeout: Duration,
}

impl PlayerClient {
    pub fn new(player_sender: Sender<Box<dyn Command>>) -> Self {
        Self {
            player_sender,
            timeout: DEFAULT_RECV_TIMEOUT,
        }
    }

    /// Set a custom timeout for queries. Overriding is mostly useful in
    /// tests; the default is generous on purpose.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /**
     * Plays a file. Resolves once the Player thread has processed the
     * command: Ok if playback was started, Err(PlayerError::Failed) if the
     * Player rejected it (e.g. the file is missing or undecodable), or the
     * usual delivery errors if the command never reached the Player.
     */
    pub async fn play(&self, file: String) -> Result<(), PlayerError> {
        self.request_async(move |player| player.play(file))
            .await
            // map file error to PlayerError or return the Ok()
            .and_then(|result| result.map_err(PlayerError::Failed))
    }

    /**
     * Stops playback. Resolves once the Player thread has processed the
     * command.
     */
    pub async fn stop(&self) -> Result<(), PlayerError> {
        self.request_async(|player| player.stop()).await
    }

    /**
     * Sets the volume and returns the resulting volume. The set and the read
     * happen inside a single command on the Player thread's queue, so the
     * returned value is the volume *after* this call (serialised after any
     * earlier commands, and after the set itself).
     */
    pub async fn set_volume(&self, volume: u8) -> Result<u8, PlayerError> {
        self.request_async(move |player| {
            player.set_volume(volume);
            player.get_volume()
        })
        .await
    }

    /**
     * Runs `query` on the Player thread and blocks the calling thread until
     * the result arrives or the client's timeout elapses. Blocking: only
     * call from sync contexts; async code must use request_async, which runs
     * this on the actix blocking pool.
     */
    fn request<T: Send + 'static>(
        &self,
        query: impl FnOnce(&mut Player) -> T + Send + 'static,
    ) -> Result<T, PlayerError> {
        let (reply_tx, reply_rx) = channel();
        let command = QueryCommand::new(query, reply_tx);

        // No receiver: the Player thread is gone before the query was sent.
        if self.player_sender.send(Box::new(command)).is_err() {
            return Err(PlayerError::Dead);
        }

        wait_for_reply(reply_rx, self.timeout)
    }

    /**
     * Helper to run queries (commands that return a response) from the Actix
     * async runtime by using `web::block()` to run sync code on the Actix
     * blocking pool.
     * Must be awaited inside the actix runtime.
     * Usage:
     *     player
     *       .request_async(|player| player.get_volume())
     *       .await
     */
    pub async fn request_async<T: Send + 'static>(
        &self,
        query: impl FnOnce(&mut Player) -> T + Send + 'static,
    ) -> Result<T, PlayerError> {
        // clone so the blocking closure owns the sender
        let client = self.clone();
        web::block(move || client.request(query))
            .await
            .unwrap_or_else(|_| Err(PlayerError::Aborted))
    }
}

/**
 * Wait for a reply, distinguishing a dead Player thread (channel disconnected
 * immediately) from a stuck one (no reply within the given timeout). The
 * timeout is logged so the two cases are distinguishable in logs.
 */
fn wait_for_reply<T>(reply_rx: Receiver<T>, timeout: Duration) -> Result<T, PlayerError> {
    match reply_rx.recv_timeout(timeout) {
        Ok(value) => Ok(value),
        Err(RecvTimeoutError::Disconnected) => Err(PlayerError::Dead),
        Err(RecvTimeoutError::Timeout) => {
            log::error!(
                "Timed out after {:?} waiting for a reply from the Player thread",
                timeout
            );
            Err(PlayerError::Timeout)
        }
    }
}

// In-module tests of the private sync `request()` method. Tests of public API are in tests/.
#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_request_runs_on_player_thread() {
        let client = crate::spawn_player();
        let caller = thread::current().id();

        assert_ne!(client.request(|_| thread::current().id()), Ok(caller));
    }

    #[actix_web::test]
    async fn test_request_round_trips_volume() {
        let client = crate::spawn_player();

        assert_eq!(client.set_volume(42).await, Ok(42));

        assert_eq!(client.request(|p| p.get_volume()), Ok(42));
    }

    #[actix_web::test]
    async fn test_request_returns_composite_values() {
        let client = crate::spawn_player();

        assert_eq!(client.set_volume(63).await, Ok(63));

        let result = client.request(|p| (p.get_volume(), p.status()));
        assert_eq!(result, Ok((63, 0)));
    }

    #[actix_web::test]
    async fn test_request_returns_dead_when_player_is_gone() {
        let (tx, rx) = mpsc::channel();
        drop(rx); // player thread has exited

        let client = PlayerClient::new(tx);
        assert_eq!(
            client.play(String::from("some/track.wav")).await,
            Err(PlayerError::Dead)
        );
        assert_eq!(client.stop().await, Err(PlayerError::Dead));
        assert_eq!(client.set_volume(10).await, Err(PlayerError::Dead));

        assert_eq!(client.request(|p| p.get_volume()), Err(PlayerError::Dead));
    }

    /// A query that outlives the client's receive timeout must yield an error
    /// instead of blocking forever. Uses a short test-only timeout (the
    /// spawn_player client has the 5s default), so this stays fast.
    #[test]
    fn test_request_times_out_on_stuck_player_thread() {
        let (tx, rx) = mpsc::channel::<Box<dyn Command>>();
        thread::spawn(move || {
            let mut player = Player::new();
            while let Ok(command) = rx.recv() {
                player.command(command);
            }
        });
        let client = PlayerClient::new(tx).with_timeout(Duration::from_millis(50));

        let (started_tx, started_rx) = mpsc::channel();

        let result = client.request(move |_| {
            let _ = started_tx.send(()); // let the test confirm the player thread ran the closure
            thread::sleep(Duration::from_secs(1));
            1u8
        });

        started_rx.recv().unwrap(); // player thread actually ran the closure
        assert_eq!(result, Err(PlayerError::Timeout));
    }

    /// A missing file must surface as PlayerError::Failed (with the underlying
    /// file error), not panic the Player thread.
    #[actix_web::test]
    async fn test_play_with_missing_file_fails_and_keeps_player_alive() {
        let client = crate::spawn_player();

        assert!(matches!(
            client
                .play(String::from("definitely/missing/file.wav"))
                .await,
            Err(PlayerError::Failed(_))
        ));

        // the Player thread is still alive and answering
        assert_eq!(client.set_volume(42).await, Ok(42));
        assert_eq!(client.request(|p| p.get_volume()), Ok(42));
    }
}
