
use crate::Deserialize;
use crate::Era;
use crate::PlayerGender;
use crate::Serialize;
use crate::Team;
use crate::ThreadRng;
use crate::Rc;
// A league containts a vector of teams, but also keeps track of the gender and era enums. A league can create team, an also ensure that
// each team follows the gender and era rules.
#[derive(Serialize, Deserialize)]
pub struct League {
    name: String,

    teams: Vec<Team>,
    gender: PlayerGender,
    era: Era,
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

    pub fn new_team(&mut self, name: &String, thread: &mut ThreadRng) -> Team {
        Team::new(name, self.gender, self.era, thread)
    }

    pub fn add_team(&mut self, team: Team) {
        self.teams.push(team)
    }
}
