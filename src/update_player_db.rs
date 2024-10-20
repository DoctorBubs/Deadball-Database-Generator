use rusqlite::Connection;
use serde::Serialize;

fn update_player_column(
    conn: &mut Connection,
    column_name: String,
    player_id: i64,
    input_value: serde_json::Value,
) -> Result<usize, rusqlite::Error> {
    let sql_string = format!(
        "UPDATE players SET {} = ?1 WHERE player_id = ?2",
        column_name
    );
    conn.execute(
        &sql_string,
        [input_value, serde_json::to_value(player_id).unwrap()],
    )
}

/// Signifiys that a struct is part of a Playerstruct, and has a value in the database.
pub trait UpdatePlayerDb {
    /// Returns a str that signifys what columnn in the player table a value is assigned to.
    fn get_column_name(&self) -> &str;
    /// Takes a player id, and updates the players entry in the database with the value provided.
    fn update_player_db(
        &self,
        conn: &mut Connection,
        player_id: i64,
    ) -> Result<usize, rusqlite::Error>
    where
        Self: Serialize,
    {
        // First we get the colum name we will be updating.
        let column_name = self.get_column_name().to_string();
        // And we use that to create a SQL query that we can execute.
        update_player_column(
            conn,
            column_name,
            player_id,
            serde_json::to_value(self).unwrap(),
        )
    }
}

pub fn update_player_db_option<T: UpdatePlayerDb + Serialize + Default>(
    field_opt: Option<T>,
    conn: &mut Connection,
    player_id: i64,
) -> Result<usize, rusqlite::Error> {
    let column_name = match &field_opt {
        Some(value) => value.get_column_name().to_string(),
        None => {
            let default = T::default();
            default.get_column_name().to_string()
        }
    };
    let input_value = serde_json::to_value(&field_opt).unwrap();
    update_player_column(conn, column_name, player_id, input_value)
}
