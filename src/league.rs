use std::ops::Add;

use rusqlite::Connection;


use crate::Deserialize;
use crate::Era;
use crate::PlayerGender;


use crate::Serialize;
use crate::Team;
use crate::ThreadRng;

// A league containts a vector of teams, but also keeps track of the gender and era enums. A league can create team, an also ensure that
// each team follows the gender and era rules.
#[derive(Serialize, Deserialize)]
pub struct League {
    pub name: String,
    pub teams: Vec<Team>,
    pub gender: PlayerGender,
    pub era: Era,
    //bench_quality:BatterQuality
}
#[derive(Debug)]
pub enum AddTeamError {
    AbrvTaken,
    NameTaken,
    DatabaseError,
}

impl League {
    pub fn new(name: &String, gender: PlayerGender, era: Era) -> League {
        League {
            name: name.to_string(),
            teams: Vec::new(),
            gender,
            era,
        }
    }

    pub fn new_team(
        &mut self,
        new_abrv: &String,
        new_name: &String,
        thread: &mut ThreadRng,
        league_id: i64,
        conn: &mut Connection,
    ) -> Result<(), AddTeamError> {
        for team in &self.teams {
            if new_abrv == &team.abrv {
                return Err(AddTeamError::AbrvTaken);
            } else if new_name == &team.name {
                return Err(AddTeamError::NameTaken);
            };
        }

        let team_id = conn.last_insert_rowid();
        let new_team = Team::new(new_abrv, new_name, self.gender, self.era, thread);
        let new_team_score = new_team.team_score.to_string();
        let team_enter_result = conn.execute(
            "INSERT INTO teams(team_name,abrv, league_id, team_score) VALUES(?1,?2, ?3,?4)",
            [
                &new_name,
                &new_abrv,
                &league_id.to_string(),
                &new_team_score,
            ],
        );

        match team_enter_result {
            Ok(_) => (),
            Err(message) => return AddTeamError::DatabaseError
        };
        let save_team_result = new_team.save_players_sql(conn, team_id);
        match save_team_result{
            (_) => (),
            Err(_) => return  AddTeamError::DatabaseError
        };
       // let new_team_string = new_team.to_string();
        self.teams.push(new_team);
        Ok(())
        //Ok(new_team_string)
    }

    /* pub fn add_team(&mut self, team: Team) {
        self.teams.push(team)
    }*/
}
