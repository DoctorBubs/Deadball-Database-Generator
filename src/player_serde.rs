use rusqlite::{named_params, Connection};
#[derive(Debug)]
/// Used to convert Player struct into Serde values.
pub struct PlayerSerde<'a> {
    pub team_id: i64,
    pub player_name: &'a String,
    pub age: i32,
    pub pos: serde_json::Value,
    pub hand: serde_json::Value,
    pub bt: String,
    pub obt_mod: String,
    pub obt: String,
    pub pd: serde_json::Value,
    pub pd_int: serde_json::Value,
    pub pitcher_trait: serde_json::Value,
    pub team_spot: String,
    pub contact: serde_json::Value,
    pub defense: serde_json::Value,
    pub power: serde_json::Value,
    pub speed: serde_json::Value,
    pub toughness: serde_json::Value,
    pub trade_value: i32,
}

impl PlayerSerde<'_> {
    /// Saves struct so SQl via a conn, returns an Option containing the row id.
    pub fn save_to_sql(&self, conn: &mut Connection) -> Result<i64, rusqlite::Error> {
        conn.execute(
            "INSERT INTO players(
                team_id,player_name,age,pos,hand,
                bt,obt_mod,obt,
                pd,pd_int,pitcher_trait,team_spot,
                contact,defense,power,speed,toughness,trade_value) 
            VALUES(:team_id, 
                :player_name, 
                :age, 
                :pos, 
                :hand, 
                :bt, 
                :obt_mod, 
                :obt, 
                :pd,
                :pd_int, 
                :pitcher_trait, 
                :team_spot, 
                :contact,
                :defense,
                :power,
                :speed,
                :toughness,
                :trade_value
            )",
            named_params![
                ":team_id": self.team_id,
                ":player_name":self.player_name,
                ":age":&self.age.to_string(),
                ":pos":&self.pos,
                ":hand":self.hand,
                ":bt":&self.bt,
                ":obt_mod":&self.obt_mod,
                ":obt":&self.obt,
                ":pd":self.pd,
                ":pd_int": self.pd_int,
                ":pitcher_trait": self.pitcher_trait,
                ":team_spot":self.team_spot,
                ":contact": self.contact,
                ":defense":self.defense,
                ":power": self.power,
                ":speed": self.speed,
                ":toughness": self.toughness,
                ":trade_value": self.trade_value
            ],
        )?;

        let new_player_id = conn.last_insert_rowid();
        Ok(new_player_id)
    }
}
