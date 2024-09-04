use tracing::{event, Level};

pub fn act(user: &str, is_opt_out: bool) {
  // TODO: Should notify watchers, and remove user from WATCHED_USERS and then delete from DB if opt-out.
  event!(
    Level::INFO,
    "Now notifying watchers of {user} that they have opted out."
  );
}
