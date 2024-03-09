use std::path::Path;

use inquire::validator::MinLengthValidator;
use rusqlite::Connection;

use crate::add_new_team;
use crate::get_league_name;
use crate::select_era;
use crate::select_gender;
use crate::team::load_team;
use crate::Deserialize;
use crate::Era;
use crate::LeagueWrapper;
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
        println!("New team id = {}", team_id);
        match team_enter_result {
            Ok(_) => (),
            Err(_message) => return Err(AddTeamError::DatabaseError),
        };
        let save_team_result = new_team.save_players_sql(conn, team_id);
        match save_team_result {
            Err(_) => return Err(AddTeamError::DatabaseError),
            _ => (),
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

// Creates a new leagues, and saves the league in the database
pub fn create_new_league(thread: &mut ThreadRng, conn: &mut Connection) -> std::io::Result<()> {
    //let league_name: String;
    //let mut _folder_path: &Path;
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


    // We then serialize the era and jender to json.
    let era_json = serde_json::to_string(&era).unwrap();

    let gender_json = serde_json::to_string(&gender).unwrap();
    // And we create a new entry in the sql databse.
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
    // Via last_inster_rowid, we get the SQl id for the new league, as the teams we generate will need it.
    let league_id = conn.last_insert_rowid();
    // We then create a leage struct in rust.
    let mut new_league = League::new(&league_name, gender, era);
    println!("{} created", &league_name);
    //And then prompt the user to create the first team for the league.
    add_new_team(&mut new_league, thread, conn, league_id, true)
}

#[derive(Debug)]
pub struct TeamWrapper {
    pub team_id: i64,
    pub team: Team,
}

pub fn load_league(
    thread: &mut ThreadRng,
    conn: &mut Connection,
    wrapper: LeagueWrapper,
) -> Result<(), rusqlite::Error> {
    let LeagueWrapper {
        league_id,
        mut league,
    } = wrapper;
    
    let era = league.era;
    /*let stmt_string = format!(
        "SELECT id,abrv,team_name,team_score,wins,losses FROM teams WHERE league_id = ?1",
        league_id
    );*/

    /*let stmt_string = format!(
        "SELECT id,abrv,team_name,team_score,wins,losses FROM teams WHERE league_id = ?1",

    );*/

    // We select the teams from the database that matchch the league id
    let mut stmt = conn.prepare(
        "SELECT id,abrv,team_name,team_score,wins,losses 
        FROM teams 
        WHERE league_id = ?1",
    )?;

    let league_iter = stmt.query_map([league_id], |row| {
        Ok(TeamWrapper {
            team_id: row.get(0)?,
            team: Team {
                abrv: row.get(1)?,
                name: row.get(2)?,
                team_score: row.get(3)?,
                wins: row.get(4)?,
                losses: row.get(5)?,
                lineup: Vec::new(),
                bench: Vec::new(),
                starting_pitching: Vec::new(),
                bullpen: match era {
                    Era::Ancient => None,
                    Era::Modern => Some(Vec::new()),
                },
            },
        })
    })?;

    let wrappers: Vec<TeamWrapper> = league_iter.map(|x| x.unwrap()).collect();
    // We drop stmt so we can borrw conn later.
    drop(stmt);
    for wrapper in wrappers {
        // We load the team from the database in the form of a Rust struct.
        let loaded_team = load_team(conn, wrapper)?;

        //println!("team {} loaded", loaded_team.name);
        //println!("{}", loaded_team);
        // And add the team to the team vector for hte league.
        league.teams.push(loaded_team)
    }
    println!("Leauge{} loaded", league.name);
    // Now that we have loaded the existing league from the database, it is time to generate a new team.
    match add_new_team(&mut league, thread, conn, league_id, true) {
        Ok(_) => Ok(()),
        Err(_) => {
            println!("Error adding a new team, please try again");
            Ok(())
        }
    }

    //todo!();
    /*  let mut league: League;
    let league_info =
        fs::read_to_string(path.join("league_info.txt")).expect("league_info file is missing");
    let league_result = serde_json::from_str(&league_info);

    match league_result {
        Ok(loaded_league) => league = loaded_league,
        Err(_) => {
            println!("League Info file is corrupted!");
            let ans = Confirm::new("Would you like to load a different league?")
                .with_default(true)
                .prompt();
            match ans {
                Ok(true) => return league_check(conn, thread),
                Ok(false) => return Ok(()),
                Err(_) => panic!("Error after league is corrupted"),
            }
        }
    };

    println!("League Loaded");
    add_team_check(&mut league, path, thread)*/
}
