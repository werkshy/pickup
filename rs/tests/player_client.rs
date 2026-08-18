//! Integration tests for PlayerClient's public async API. The sync `request`
//! path is private and is covered by unit tests inside src/player/client.rs.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use pickup::{
    player::{Command, Player, PlayerClient, PlayerError},
    spawn_player,
};

#[actix_web::test]
async fn test_request_async_runs_on_player_thread() {
    let client = spawn_player();
    let caller = thread::current().id();

    assert_ne!(
        client.request_async(|_| thread::current().id()).await,
        Ok(caller)
    );
}

#[actix_web::test]
async fn test_request_async_returns_value() {
    let client = spawn_player();

    assert_eq!(client.set_volume(55).await, Ok(55));

    assert_eq!(client.request_async(|p| p.get_volume()).await, Ok(55));
}

#[actix_web::test]
async fn test_request_async_returns_dead_when_player_is_gone() {
    let (tx, rx) = mpsc::channel();
    drop(rx); // player thread has exited

    let client = PlayerClient::new(tx);

    assert_eq!(
        client.play(String::from("some/track.wav")).await,
        Err(PlayerError::Dead)
    );
    assert_eq!(client.stop().await, Err(PlayerError::Dead));
    assert_eq!(client.set_volume(10).await, Err(PlayerError::Dead));

    assert_eq!(
        client.request_async(|p| p.get_volume()).await,
        Err(PlayerError::Dead)
    );
}

/// A query that outlives the client's receive timeout must yield an error
/// instead of hanging forever. Uses a short test-only timeout (the
/// spawn_player client has the 5s default), so this stays fast.
#[actix_web::test]
async fn test_request_async_times_out_on_stuck_player_thread() {
    let (tx, rx) = mpsc::channel::<Box<dyn Command>>();
    thread::spawn(move || {
        let mut player = Player::new();
        while let Ok(command) = rx.recv() {
            player.command(command);
        }
    });
    let client = PlayerClient::new(tx).with_timeout(Duration::from_millis(50));

    let result = client
        .request_async(move |_| {
            thread::sleep(Duration::from_secs(1));
            1u8
        })
        .await;

    assert_eq!(result, Err(PlayerError::Timeout));
}

/// A missing file must surface as PlayerError::Failed, and the player must
/// keep working afterwards.
#[actix_web::test]
async fn test_play_with_missing_file_fails_and_keeps_player_alive() {
    let client = spawn_player();

    assert!(matches!(
        client
            .play(String::from("definitely/missing/file.wav"))
            .await,
        Err(PlayerError::Failed(_))
    ));

    assert_eq!(client.set_volume(42).await, Ok(42));
    assert_eq!(client.request_async(|p| p.get_volume()).await, Ok(42));
}
