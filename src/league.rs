use core::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use inquire::validator::MinLengthValidator;
use inquire::Confirm;
use inquire::InquireError;
use inquire::Select;
use inquire::Text;
use rusqlite::Connection;

use crate::era::select_era;
use crate::main_menu::run_main_menu;
use crate::main_menu::MenuInput;

use crate::player::select_gender;
use crate::team::add_new_team;
use crate::team::load_team;
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
//Possible Errors that oculd arrise from adding a team to a league
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
    /*  Take a new abbrevaiton and name for a team, a thread for random number, a league id and connection to the the database.
        If there are no teams in the league that have the same name or abbreviation,  we attempt to add the team to the league. If it is succesfull, an Ok is returned
    */
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
        // We create a new team
        let new_team = Team::new(new_abrv, new_name, self.gender, self.era, thread);
        // We get the team score for hte new team.
        let new_team_score = new_team.team_score.to_string();
        // We enter the team into the database.
        let team_enter_result = conn.execute(
            "INSERT INTO teams(team_name,abrv, league_id, team_score) VALUES(?1,?2, ?3,?4)",
            [
                &new_name,
                &new_abrv,
                &league_id.to_string(),
                &new_team_score,
            ],
        );
        // We save the team ID, so that we we generate the new players they can be saved in the databse with the league id as the foreign key.
        let team_id = conn.last_insert_rowid();

        match team_enter_result {
            Ok(_) => (),
            Err(_message) => return Err(AddTeamError::DatabaseError),
        };
        //If all has gone well, we save the players that have been generated into the database
        new_team.save_players_sql(conn, team_id).unwrap();
        // And we inster the team struct into the league's team vector.
        self.teams.push(new_team);
        Ok(())
        
    }

  
}

fn check_name_vec(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT league_name FROM leagues")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut names = Vec::new();
    for name_result in rows {
        names.push(name_result?);
    }

    Ok(names)
}

// Creates a new leagues, and saves the league in the database
pub fn create_new_league(thread: &mut ThreadRng, conn: &mut Connection) -> std::io::Result<()> {
    let _validator = MinLengthValidator::new(3);

    let league_name: String;

    let taken_names = check_name_vec(conn).unwrap();

    loop {
        if !taken_names.is_empty() {
            println!("The following league names have already been taken:");
            for name in &taken_names {
                println!("{}", name)
            }
        };

        let potential_name = Text::new("Please enter a name for the new league")
            .prompt()
            .unwrap();

        if !taken_names.contains(&potential_name) {
            league_name = potential_name;
            break;
        }
    }

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

    if league_entry.is_err() {
        println!("Error creating a new league in the database.");
        return Ok(());
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
//Teamwrapper contains a team id from the database, as well as the team struct that correspond ot the id.
pub struct TeamWrapper {
    pub team_id: i64,
    pub team: Team,
}

pub fn load_league(
    thread: &mut ThreadRng,
    conn: &mut Connection,
    wrapper: LeagueWrapper,
) -> Result<(), rusqlite::Error> {
    // We destructure the LeagueWrapper
    let LeagueWrapper {
        league_id,
        mut league,
    } = wrapper;
    
    let era = league.era;
    // We query the database to select all teams in the database that belong to the league via the league_id car
    let mut stmt = conn.prepare(
        "SELECT id,abrv,team_name,team_score,wins,losses 
        FROM teams 
        WHERE league_id = ?1",
    )?;
    
    let team_iter = stmt.query_map([league_id], |row| {
        // For each team that matchers, we create a new TeamWrapper that is wrapped in an Ok.  
        Ok(TeamWrapper {
            // We set the team id field to the team id from the database
            team_id: row.get(0)?,
            // We use the remaing rows to deseriale the team
            team: Team {
                // We fill out the rest of the fields in the team struct from the database entry.
                abrv: row.get(1)?,
                name: row.get(2)?,
                team_score: row.get(3)?,
                wins: row.get(4)?,
                losses: row.get(5)?,
                // We create a vector for each player pool that a team has.
                lineup: Vec::new(),
                bench: Vec::new(),
                starting_pitching: Vec::new(),
                //Ancient Era teams do not have a bullpen, while Modern Era teams do.
                bullpen: match era {
                    Era::Ancient => None,
                    Era::Modern => Some(Vec::new()),
                },
            },
        })
    })?;
    // We make a vector of TeamWrapper that are no longer in an ok.
    let wrappers: Vec<TeamWrapper> = team_iter.map(|x| x.unwrap()).collect();
    // We drop stmt so we can borrw conn later.
    drop(stmt);
    // We then loa
    for wrapper in wrappers {
        // We load the team from the database in the form of a Rust struct.
        let loaded_team = load_team(conn, wrapper)?;

        // And add the team to the league's teams vector.
        league.teams.push(loaded_team)
    }
    
    // Now that we have loaded the existing league from the database, it is time to generate a new team.
    add_new_team(&mut league, thread, conn, league_id, true).unwrap();
    Ok(())
}

/*  The League Wrapper struct is used when the program checks to see what leagues are saved in the database.

 It contains the ID which the leagues is saved in the database, as well a desrtialzied League struct from the database
*/

pub struct LeagueWrapper {
    league_id: i64,
    league: League,
}

// We implement display for LeagueWrapper, as we will need to see print a list of all leeagues to the console when a user wants to open an existing leaghue
impl fmt::Display for LeagueWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}", self.league_id, self.league.name)
    }
}

//This function queries the database for all leagues. If there are no leagues in the database, the user is prompted to create one.
pub fn league_check(
    conn: &mut Connection,
    thread: &mut ThreadRng,
    input: MenuInput,
) -> Result<(), rusqlite::Error> {
    // We query the database to get all the leagues that already exist.
    let mut stmt = conn.prepare("SELECT * from leagues").unwrap();
    // We wrap the rows into a LeagueWrapper that is part of a Rust Ok.
    let league_iter = stmt
        .query_map([], |row| {
            Ok(LeagueWrapper {
                league_id: row.get(0)?,
                league: League {
                    name: row.get(1)?,
                    era: serde_json::from_value(row.get(2)?).unwrap(),

                    //
                    gender: serde_json::from_value(row.get(3)?).unwrap(),

                    //PlayerGender::from_string(row.get(3)?),
                    teams: Vec::new(),
                },
            })
        })
        .unwrap();

    let mut options = Vec::new();
    // We unwrap the results in leauge iter, and push it to the options vec
    for wrapper in league_iter {
        options.push(wrapper.unwrap())
    }
    // We drop the stmt so we can borrow conn later.
    drop(stmt);
    // If there are no leagues in the database, tthe user is prompted to create a league
    if options.is_empty() {
        println!("No Leagues created yet! Let's create a new league to get started.");
        create_new_league(thread, conn).unwrap();
        Ok(())
    } else {
        //Otherwise, the user is shown a list of all leagues that currently exist, and is prompted to select one.
        let ans: Result<LeagueWrapper, InquireError> =
            Select::new("Select an existing league", options).prompt();
        match ans {
            Ok(select) => match input {
                //If the users decided they wanted to create a new team earlierm they are taken to the prompt to create a new team
                MenuInput::CreateNewTeam => {
                    load_league(thread, conn, select)?;
                    Ok(())
                }
                //Otherwise, the league is saved to the users disk.
                MenuInput::RefreshLeague => {
                    println!("Refreshing league.");
                    save_league(&select.league, conn, thread).unwrap();
                    Ok(())
                }
                _ => panic!("Invalid Menu Input:{:?}", input),
            },
            Err(_) => {
                println!("Error selecting a new league");
                Ok(())
            }
        }
    }
}

// Once a league is saved, we save a copy of the league data in a folder.
pub fn save_league(
    league: &League,
    conn: &mut Connection,
    thread: &mut ThreadRng,
) -> std::io::Result<()> {
    println!();
    let flder_path_string = league.name.to_string();
    let folder_path = Path::new(&flder_path_string);
    fs::create_dir_all(folder_path)?;

    for team in &league.teams {
        let file_path = folder_path.join(format!("{}.txt", team.name).as_str());

        let mut file = File::create(file_path)?;
        file.write_all(team.to_string().as_bytes())?;
    }
    println!("League saved succesfully.");
    //We then prompt the user if they would like to return to the main menu
    let ans = Confirm::new("Would you like to return to the main menu?")
        .with_default(true)
        .prompt();
    match ans {
        Ok(true) => run_main_menu(conn, thread),
        _ => Ok(()),
    }
}
