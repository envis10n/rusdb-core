use crate::RusDbConnection;

#[derive(Clone)]
pub struct RusDatabase {
    conn: RusDbConnection,
}

impl RusDatabase {
    pub async fn connect(dst: &'static str) -> Self {
        Self {
            conn: RusDbConnection::connect(dst).await,
        }
    }
}
