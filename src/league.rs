use core::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use inquire::validator::MinLengthValidator;
use inquire::InquireError;
use inquire::Select;
use inquire::Text;

use rusqlite::Connection;

use crate::era::select_era;
use crate::inquire_check;
use crate::main_menu::EditLeagueInput;
use crate::main_menu::LoadLeagueInput;
use crate::player::select_gender;
//use crate::sched_view::view_schedule;

use crate::sched_view::view_schedule;
use crate::schedule::save_schedule_sql;
use crate::team::add_new_team;
use crate::team::load_team;
use crate::vec_to_hash;
use crate::Deserialize;
use crate::Era;

use crate::PlayerGender;

use crate::Serialize;
use crate::Team;
use crate::ThreadRng;

//use crate::schedule::*;
use std::collections::HashMap;

// A league containts a vector of teams, but also keeps track of the gender and era enums. A league can create team, an also ensure that
// each team follows the gender and era rules.
#[derive(Serialize, Deserialize)]
pub struct League {
    pub name: String,
    pub teams: Vec<Team>,
    pub gender: PlayerGender,
    pub era: Era,
    pub league_id: i64, //bench_quality:BatterQuality,
}

pub struct LeagueWrapper {
    pub league_id: i64,
    pub league: League,
}

// We implement display for LeagueWrapper, as we will need to see print a list of all leeagues to the console when a user wants to open an existing leaghue
impl fmt::Display for LeagueWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}", self.league_id, self.league.name)
    }
}

#[derive(Debug)]
//Possible Errors that oculd arrise from adding a team to a league
pub enum AddTeamError {
    AbrvTaken,
    NameTaken,
    DatabaseError,
}

struct StandingWrapper {
    name: String,
    team_score: i32,
    wins: i32,
    losses: i32,
    games_behind: i32,
}
impl League {
    pub fn new(name: &String, gender: PlayerGender, era: Era, league_id: i64) -> League {
        League {
            name: name.to_string(),
            teams: Vec::new(),
            gender,
            era,
            league_id,
        }
    }

    /*pub fn get_new_schedule(&self, series_length: i32,series_per_matchup:i32) ->  Result<Vec<Round>, ScheduleGenError>{
        new_schedule(&self.teams,  series_length, series_per_matchup)

    }*/
    pub fn display_standings(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = conn.prepare(
            "
         WITH batter_scores AS(
            SELECT 
                teams.team_id AS team_id,
                SUM(players.bt) AS bt_score
            FROM
                players
            INNER JOIN 
                teams ON teams.team_id = players.team_id
            WHERE
                teams.league_id = ?1
                AND players.PD IS NULL
            GROUP BY teams.team_id
        ),
        
        pitcher_scores AS(
            SELECT 
                teams.team_id as team_id,
                SUM(players.pd_int) * 7 as pd_score
                
            FROM
                players
            INNER JOIN 
                teams ON teams.team_id = players.team_id
            WHERE
                teams.league_id = ?1
                AND
                players.PD IS NOT NULL
            GROUP BY teams.team_id
        
        ),

        team_scores AS(
            SELECT
                teams.team_id AS team_id,
                (batter_scores.bt_score + pitcher_scores.pd_score) / 10 AS team_score
                FROM 
                    teams
                INNER JOIN
                    batter_scores ON batter_scores.team_id = teams.team_id
                INNER JOIN pitcher_scores ON pitcher_scores.team_id = teams.team_id
               
        ),
        
        first_place AS(
            SELECT 
                leagues.league_id AS league_id, 
                MAX(teams.wins) AS wins
            FROM 
                leagues
            INNER JOIN 
                teams ON teams.league_id = leagues.league_id
            WHERE 
                leagues.league_id = ?1
            GROUP BY 
                leagues.league_id
        )
        
        SELECT 
            teams.team_name, 
            team_scores.team_score,
            teams.wins, 
            teams.losses,
            (first_place.wins - teams.wins) AS games_behind 
        FROM 
            teams
        INNER JOIN 
            first_place ON first_place.league_id = teams.league_id
        INNER JOIN
            team_scores ON teams.team_id = team_scores.team_id
        WHERE 
            teams.league_id = ?1
        ORDER BY 
            games_behind ASC, team_scores.team_score DESC;
        ",
        )?;

        let rows = stmt.query_map([self.league_id], |row| {
            Ok({
                StandingWrapper {
                    name: row.get(0)?,
                    team_score: row.get(1)?,
                    wins: row.get(2)?,
                    losses: row.get(3)?,
                    games_behind: row.get(4)?,
                }
            })
        })?;

        for row in rows {
            let standing = row.unwrap();
            println!(
                "{} {} {} {} {}",
                standing.name,
                standing.team_score,
                standing.wins,
                standing.losses,
                standing.games_behind
            )
        }

        Ok(())
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
        let mut new_team = Team::new(new_abrv, new_name, self.gender, self.era, thread);
        // We get the team score for hte new team.
        // We enter the team into the database.
        let team_enter_result = conn.execute(
            "INSERT INTO teams(team_name,abrv, league_id) VALUES(?1,?2, ?3)",
            [&new_name, &new_abrv, &league_id.to_string()],
        );
        // We save the team ID, so that we we generate the new players they can be saved in the databse with the league id as the foreign key.
        let new_team_id = conn.last_insert_rowid();
        //new_team.team_id = team_id as i32;
        match team_enter_result {
            Ok(_) => (),
            Err(_message) => return Err(AddTeamError::DatabaseError),
        };
        //If all has gone well, we save the players that have been generated into the database
        new_team.save_players_sql(conn, new_team_id).unwrap();
        // And we inster the team struct into the league's team vector.
        self.teams.push(new_team);
        Ok(())
    }

    pub fn new_team_hash(&self) -> HashMap<i64, &Team> {
        let mut result = HashMap::new();
        for team in self.teams.iter() {
            result.insert(team.team_id, team);
        }
        result
    }
}

pub fn check_name_vec(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT league_name FROM leagues")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut names = Vec::new();
    for name_result in rows {
        names.push(name_result?);
    }

    Ok(names)
}
// Returns a hash
pub fn check_name_hash(conn: &Connection) -> Result<HashMap<String, bool>, rusqlite::Error> {
    let names_vec = check_name_vec(conn)?;
    let mut result = HashMap::new();
    for name in names_vec.into_iter() {
        result.insert(name, true);
    }

    Ok(result)
}

// Creates a new leagues, and saves the league in the database
pub fn create_new_league(
    thread: &mut ThreadRng,
    conn: &mut Connection,
) -> Result<(), rusqlite::Error> {
    let _validator = MinLengthValidator::new(3);

    let league_name: String;

    let taken_names = check_name_vec(conn)?;
    // let mut taken_hash = HashMap::new();
    let taken_hash = vec_to_hash(&taken_names);
    //for name in taken_names.iter() {
    //taken_hash.insert(name, true);
    // }
    loop {
        if !taken_names.is_empty() {
            println!("The following league names have already been taken:");
            for name in &taken_names {
                println!("{}", name)
            }
        };

        let potential_name_choice = Text::new("Please enter a name for the new league").prompt();
        let potential_name = match potential_name_choice {
            Ok(input) => input,
            Err(message) => return inquire_check(message),
        };
        if taken_hash.get(&potential_name).is_none() {
            league_name = potential_name;
            break;
        } else {
            println!("Name already taken.")
        }
    }

    // We have the user select the era for the league.
    let era_choice = select_era();
    let era = match era_choice {
        Ok(input) => input,
        Err(message) => return inquire_check(message),
    };
    // As well as the gender of the players for the league.
    let gender_choice = select_gender();
    let gender = match gender_choice {
        Ok(input) => input,
        Err(message) => return inquire_check(message),
    };
    // We then create a league struct.

    // We then serialize the era and jender to json.
    let era_json = serde_json::to_string(&era).unwrap();

    let gender_json = serde_json::to_string(&gender).unwrap();
    // And we create a new entry in the sql databse.
    let _league_entry = conn.execute(
        "INSERT INTO leagues(league_name,era,gender) VALUES(?1, ?2, ?3)",
        [&league_name, &era_json, &gender_json],
    )?;

    // if league_entry.is_err() {
    // println!("Error creating a new league in the database.");
    // return Ok(());
    //};
    // Via last_inster_rowid, we get the SQl id for the new league, as the teams we generate will need it.
    let league_id = conn.last_insert_rowid();
    // We then create a leage struct in rust.
    let mut new_league = League::new(&league_name, gender, era, league_id);
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

/// Loads teams from SQL database and adds to league struct.
pub fn load_teams_from_sql(
    league_id: i64,
    league: &mut League,
    conn: &mut Connection,
) -> Result<(), rusqlite::Error> {
    let era = league.era;
    // We query the database to select all teams in the database that belong to the league via the league_id car
    let mut stmt = conn.prepare(
        "SELECT team_id,abrv,team_name,wins,losses 
        FROM teams 
        WHERE league_id = ?1",
    )?;

    let team_iter: Vec<Team> = stmt
        .query_map([league_id], |row| {
            // For each team that matchers, we create a new TeamWrapper that is wrapped in an Ok.
            Ok(
                // We use the remaining rows to deseirialize the team
                Team {
                    // We fill out the rest of the fields in the team struct from the database entry.
                    team_id: row.get(0)?,
                    abrv: row.get(1)?,
                    name: row.get(2)?,
                    wins: row.get(3)?,
                    losses: row.get(4)?,
                    // We create a vector for each player pool that a team has.
                    lineup: Vec::new(),
                    bench: Vec::new(),
                    starting_pitching: Vec::new(),
                    //Ancient Era teams do not have a bullpen, while Modern Era teams do.
                    bullpen: match era {
                        Era::Ancient => None,
                        Era::Modern => Some(Vec::new()),
                    },
                    team_score: 0,
                },
            )
        })?
        .map(|x| x.unwrap())
        .collect();

    // We drop stmt so we can borrw conn later.
    drop(stmt);
    // We then loa
    for team in team_iter {
        // We load the team from the database in the form of a Rust struct.
        let loaded_team = load_team(conn, team)?;

        // And add the team to the league's teams vector.
        league.teams.push(loaded_team)
    }

    Ok(())
}

pub fn load_league(
    thread: &mut ThreadRng,
    conn: &mut Connection,
    wrapper: LeagueWrapper,
    edit_input: EditLeagueInput,
) -> Result<(), rusqlite::Error> {
    // We destructure the LeagueWrapper
    let LeagueWrapper {
        league_id,
        mut league,
    } = wrapper;

    league.display_standings(conn)?;
    load_teams_from_sql(league_id, &mut league, conn)?;

    // Now that we have loaded the existing league from the database, it is time to generate a new team or create a new schedule based off the input
    match edit_input {
        EditLeagueInput::CreateNewTeam => {
            add_new_team(&mut league, thread, conn, league_id, true).unwrap()
        } 
        EditLeagueInput::CreateSchedule => {
             match league.teams.len() % 2 == 0 {
                true => save_schedule_sql(conn, &league, thread).unwrap(),
             false => {
               println!("League must have an even number of teams");
            save_league(&league, conn, thread).unwrap();
           }
          };
          }
    };
    Ok(())
}

/*  The League Wrapper struct is used when the program checks to see what leagues are saved in the database.

 It contains the ID which the leagues is saved in the database, as well a deserialzied League struct from the database
*/
pub fn get_all_leagues_from_db(conn: &mut Connection) -> Vec<LeagueWrapper> {
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
                    league_id: row.get(0)?,

                    //PlayerGender::from_string(row.get(3)?),
                    teams: Vec::new(),
                },
            })
        })
        .unwrap();

    let mut result = Vec::new();
    // We unwrap the results in leauge iter, and push it to the options vec
    for wrapper in league_iter {
        result.push(wrapper.unwrap());
    }
    result
}

//This function queries the database for all leagues. If there are no leagues in the database, the user is prompted to create one.
pub fn league_check(
    conn: &mut Connection,
    thread: &mut ThreadRng,
    input: LoadLeagueInput,
) -> Result<(), rusqlite::Error> {
    // We query the database to get all the leagues that already exist.
    let options = get_all_leagues_from_db(conn);
    // If there are no leagues in the database, the user is prompted to create a league
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
                //If the users decided they wanted to create a new team earlier they are taken to the prompt to create a new team
                LoadLeagueInput::EditLeague(edit) => {
                    load_league(thread, conn, select, edit)?;
                    Ok(())
                }
                //Otherwise, the league is saved to the users disk.
                LoadLeagueInput::RefreshLeague => {
                    println!("Refreshing league.");
                    save_league(&select.league, conn, thread).unwrap();
                    Ok(())
                } 
                LoadLeagueInput::ViewSchedule => {
                    view_schedule(&select.league, conn)?;
                   Ok(())
                  }
            },
            Err(message) => inquire_check(message),
        }
    }
}

// Once a league is saved, we save a copy of the league data in a folder.
pub fn save_league(
    league: &League,
    _conn: &mut Connection,
    _thread: &mut ThreadRng,
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
    println!("League saved successfully.");
    Ok(())
}
