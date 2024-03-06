use crate::Deserialize;
use crate::Era;
use crate::PlayerGender;

use crate::Serialize;
use crate::Team;
use crate::ThreadRng;
use crate::player_quality::BatterQuality;
// A league containts a vector of teams, but also keeps track of the gender and era enums. A league can create team, an also ensure that
// each team follows the gender and era rules.
#[derive(Serialize, Deserialize)]
pub struct League {
    name: String,
    teams: Vec<Team>,
    gender: PlayerGender,
    era: Era,
    //bench_quality:BatterQuality
}
#[derive(Debug)]
pub enum AddTeamError {
    AbrvTaken,
    NameTaken,
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
    ) -> Result<String, AddTeamError> {
        for team in &self.teams {
            if new_abrv == &team.abrv {
                return Err(AddTeamError::AbrvTaken);
            } else if new_name == &team.name {
                return Err(AddTeamError::NameTaken);
            };
        }
        let new_team = Team::new(new_abrv, new_name, self.gender, self.era, thread);
        let new_team_string = new_team.to_string();
        self.teams.push(new_team);
        Ok(new_team_string)
    }

    /* pub fn add_team(&mut self, team: Team) {
        self.teams.push(team)
    }*/
}
