use chrono::{DateTime, Utc};

#[derive(Clone)]
pub enum FetchStatus {
    NotFetched,
    Fetching,              // Not tracking how much fetched
    FetchingProgress(u16), // Tracking number of bytes fetched
    FetchedAt(DateTime<Utc>),
}
