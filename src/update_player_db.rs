use rusqlite::Connection;
use serde::Serialize;

pub trait UpdatePlayerDb {
    fn get_column_name(&self) -> &str;
    fn update_player_db(
        &self,
        conn: &mut Connection,
        player_id: i64,
    ) -> Result<usize, rusqlite::Error>
    where
        Self: Serialize,
    {
        let column_name = self.get_column_name().to_string();
        let sql_string = format!("UPDATE players SET {} = ?1 WHERE player_id = ?2",column_name);
        conn.execute(
            &sql_string,
            [
                serde_json::to_string(self).unwrap(),
                player_id.to_string(),
            ],
        )
    }
}
