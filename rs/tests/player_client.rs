use std::sync::mpsc;
use std::thread;

use pickup::{player::PlayerClient, spawn_player};

#[test]
fn test_request_runs_on_player_thread() {
    let client = spawn_player();
    let caller = thread::current().id();

    assert_ne!(client.request(|_| thread::current().id()), Some(caller));
}

#[actix_web::test]
async fn test_request_round_trips_volume() {
    let client = spawn_player();

    client.set_volume(42).await;

    assert_eq!(client.request(|p| p.get_volume()), Some(42));
}

#[actix_web::test]
async fn test_request_returns_composite_values() {
    let client = spawn_player();

    client.set_volume(63).await;

    let result = client.request(|p| (p.get_volume(), p.status()));
    assert_eq!(result, Some((63, 0)));
}

#[actix_web::test]
async fn test_request_returns_none_when_player_is_gone() {
    let (tx, rx) = mpsc::channel();
    drop(rx); // player thread has exited

    let client = PlayerClient::new(tx);
    client.play(String::from("some/track.wav"));
    client.stop();
    client.set_volume(10).await;

    assert_eq!(client.request(|p| p.get_volume()), None);
}

#[actix_web::test]
async fn test_request_async_returns_value() {
    let client = spawn_player();

    client.set_volume(55).await;

    assert_eq!(client.request_async(|p| p.get_volume()).await, Some(55));
}

#[actix_web::test]
async fn test_request_async_returns_none_when_player_is_gone() {
    let (tx, rx) = mpsc::channel();
    drop(rx); // player thread has exited

    let client = PlayerClient::new(tx);

    assert_eq!(client.request_async(|p| p.get_volume()).await, None);
}
