use std::sync::mpsc::Sender;

use crate::player::{Command, Player};

/**
 * Generic replacement for response-carrying commands. The closure runs on the
 * Player thread and its result is sent back over a short-lived channel.
 */
pub struct QueryCommand<F, T> {
    reply: Sender<T>,
    // Option because a FnOnce closure can't be invoked through &mut self
    query: Option<F>,
}

impl<F, T> QueryCommand<F, T>
where
    F: FnOnce(&mut Player) -> T + Send + 'static,
    T: Send + 'static,
{
    pub fn new(query: F, reply: Sender<T>) -> Self {
        Self {
            reply,
            query: Some(query),
        }
    }
}

impl<F, T> Command for QueryCommand<F, T>
where
    F: FnOnce(&mut Player) -> T + Send + 'static,
    T: Send + 'static,
{
    fn action(&mut self, player: &mut Player) {
        // Sanity check defensiveness that this isn't a double invocation. A
        // second invocation would never send a reply, leaving the caller
        // waiting until its receive timeout. Fail loudly in debug/test builds.
        debug_assert!(
            self.query.is_some(),
            "QueryCommand::action must only be invoked once"
        );

        if let Some(query) = self.query.take() {
            let value = query(player);
            let _ = self.reply.send(value);
        }
        // self.query is now None (it was taken).
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    /// Only compiled in when debug_assertions are on (debug and test builds);
    /// in release builds the second action() call is a silent no-op.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "QueryCommand::action must only be invoked once")]
    fn test_action_panics_if_invoked_twice() {
        let (reply_tx, reply_rx) = mpsc::channel();
        let mut command = QueryCommand::new(|_: &mut Player| 1u8, reply_tx);
        let mut player = Player::new();

        command.action(&mut player); // first invocation is fine
        assert_eq!(reply_rx.recv().ok(), Some(1));

        command.action(&mut player); // second invocation must be loud
    }
}
