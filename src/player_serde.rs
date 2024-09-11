
use crate::Serialize;
use std::collections::HashMap;
/* 
":team_id": &team_id,
":player_name":&self.name,
":age":&self.age.to_string(),
":pos":&self.pos,
":hand":serde_json::to_value(&self.hand).unwrap(),
":bt":&self.bt.to_string(),
":obt_mod":&self.obt_mod.to_string(),
":obt":&self.obt.to_string(),
":pd":serde_json::to_value(self.pd).unwrap(),
":pd_int": serde_json::to_value(pd_int).unwrap(),
":pitcher_trait": serde_json::to_value(self.pitcher_trait).unwrap(),
":team_spot":serde_json::to_string(&team_spot).unwrap(),
":contact": serde_json::to_value(contact_option).unwrap(),
":defense":serde_json::to_value(defense_option).unwrap(),
":power": serde_json::to_value(power_option).unwrap(),
":speed": serde_json::to_value(speed_option).unwrap(),
":toughness": serde_json::to_value(toughness_option).unwrap(),
":trade_value": self.trade_value
*/


pub struct PlayerSerde{

    team_id: i64,
    player_name: String,
    age: i32,
    pos: String,
    hand: serde_json::Value,
    bt: String,
    obt_mod: String,
    obt: String,
    pd: serde_json::Value,


}