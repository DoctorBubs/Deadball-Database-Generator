use std::path::Path;

use inquire::validator::MinLengthValidator;
use rusqlite::Connection;

use crate::add_new_team;
use crate::get_league_name;
use crate::select_era;
use crate::select_gender;
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
        let team_id = conn.last_insert_rowid();
        println!("New team id = {}",team_id);
        match team_enter_result {
            Ok(_) => (),
            Err(_message) => return Err(AddTeamError::DatabaseError),
        };
        let save_team_result = new_team.save_players_sql(conn, team_id);
        match save_team_result {
            _ => (),
            Err(_) => return Err(AddTeamError::DatabaseError),
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


// To create a new league
fn create_new_league(thread: &mut ThreadRng, conn: &mut Connection) -> std::io::Result<()> {
    //let league_name: String;
    let mut _folder_path: &Path;
    let _validator = MinLengthValidator::new(3);
    /*
    let league_input = Text::new("Enter the name for the new league")
        .with_validator(validator.clone())
        .prompt();
    league_name = match league_input {
        Ok(input) => input.trim().to_string(),
        Err(_) => panic!("Error creating a league name!"),
    };
    */

    let league_name_attempt = get_league_name();
    let league_name = match league_name_attempt {
        Ok(name) => name,
        Err(_) => {
            println!("Error getting league name");
            return Ok(());
        }
    };

    // We have the user select the era for the league.
    let era = select_era();
    // As well as the gender of the players for the league.
    let gender = select_gender();

    // We then create a league struct.

    let era_json = serde_json::to_string(&era).unwrap();

    let gender_json = serde_json::to_string(&gender).unwrap();
    let league_entry = conn.execute(
        "INSERT INTO leagues(league_name,era,gender) VALUES(?1, ?2, ?3)",
        [&league_name, &era_json, &gender_json],
    );

    match league_entry {
        Err(_) => {
            println!("Error creating a new league in the database");
            return Ok(());
        }
        Ok(_) => (),
    };
    // Via last_inster_rowid, we get the SQl id for the new league
    let league_id = conn.last_insert_rowid();
    let mut new_league = League::new(&league_name, gender, era);
    println!("{} created", &league_name);
    //And then prompt the user to create the first team for the league.
    add_new_team(&mut new_league, thread, conn, league_id, true)
}