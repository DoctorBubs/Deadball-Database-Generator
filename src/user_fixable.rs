use rand::seq;
use rusqlite::Connection;
use serde::Deserialize;

use crate::{era::Era, pd::PD};



pub trait UserFixable{
    fn fix_via_prompt<T:UserFixable, U: UserFixable>(conn: &mut Connection, era: Era, player_id: i64, player_name: &str,pd: Option<PD>, input_error: serde_json::Error) -> Option<Box< T>>;
}

pub fn handle_fix_prompt<'a, T: UserFixable + Deserialize<'a>>(input:&'a str, conn: &mut Connection, era: Era, player_id: i64, player_name: &str, pd: Option<PD>) -> Result<T, serde_json::Error>{
    let first_attempt = serde_json::from_str(input);
    match first_attempt{
        Ok(value) => Ok(value),
        Err(input_error) => {
            let user_fix = T::fix_via_prompt(conn, era, player_id, player_name,pd, input_error);
            match user_fix{
                Some(second_value) => Ok(*second_value),
                None => Err(input_error)
            }
        }
    }

}