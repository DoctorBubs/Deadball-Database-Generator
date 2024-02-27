use crate::Era;
use crate::Player;
use crate::PlayerGender;
use crate::PlayerQuality;
use crate::Team;
use crate::ThreadRng;

use crate::Deserialize;
use crate::Serialize;


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
            gender: gender,
            era: era,
        }
    }

    pub fn new_team(&mut self,name: &String, thread: &mut ThreadRng) -> Team{
        Team::new(name,self.gender, self.era, thread)
        
    }

    pub fn add_team(&mut self, team: Team){
    
        self.teams.push(team)
    }

    fn new_player(
        &self,
        pos: String,
        quality: impl PlayerQuality,
        mut thread: &mut ThreadRng,
        era: Era,
    ) -> Player {
        Player::new(pos, self.gender, quality, &mut thread, era)
    }

    pub fn to_string(&self) -> String{
    
        let header_string = format!("Name:{} Era:{} Gender:{} \n",self.name,self.era.to_string(),self.gender.to_string());
        let team_string = self.teams.iter().map(|team| team.to_string()).reduce(|acc,e| format!("{}\n{}",acc,e)).unwrap_or_else(||"No Teams Created".to_string());
        format!("{}{}",header_string, team_string)
    
    }
}
