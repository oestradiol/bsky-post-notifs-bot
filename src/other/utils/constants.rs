// The bot will retry in incrementing intervals of 1s, up to INCREMENTS_LIMIT, for a maximum of MINUTES_LIMIT.
pub static INCREMENTS_LIMIT: u64 = 60; // 60 Seconds
static MINUTES_LIMIT: u64 = 30; // 30 Minutes
pub static MAX_FAILURES: u64 = {
  let seconds_spent_incrementing = INCREMENTS_LIMIT * (INCREMENTS_LIMIT + 1) / 2;
  let minutes_as_secs = MINUTES_LIMIT * 60;
  if minutes_as_secs < seconds_spent_incrementing {
    INCREMENTS_LIMIT
  } else {
    INCREMENTS_LIMIT + (minutes_as_secs - seconds_spent_incrementing) / INCREMENTS_LIMIT
  }
};
