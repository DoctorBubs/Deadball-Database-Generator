
mod b_traits;
mod era;
mod league;
mod lineup_score;
mod pd;
mod pitcher_rank_info;
mod player;
mod player_quality;
mod team;
mod traits;
mod main_menu;

use crate::era::Era;
use crate::league::create_new_league;
use crate::league::AddTeamError;
use crate::league::League;
use crate::pd::PD;
use crate::player::Player;
use crate::player::PlayerGender;
use crate::player_quality::BatterQuality;
use crate::player_quality::PitcherQuality;
use crate::player_quality::PlayerQuality;

use crate::team::Team;
use crate::traits::Contact;
use crate::traits::Defense;
use crate::traits::Power;
use crate::traits::Speed;
use crate::traits::Toughness;
use crate::validator::MaxLengthValidator;
use crate::validator::MinLengthValidator;

use crate::main_menu::MenuInput;
use crate::main_menu::run_main_menu;
use inquire::*;
use league::load_league;
use rand::rngs::ThreadRng;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

use std::fmt;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;



fn select_era() -> Era {
    let options: Vec<Era> = vec![Era::Ancient, Era::Modern];
    let ans: Result<Era, InquireError> =
        Select::new("Select the era for the league", options).prompt();
    match ans {
        Ok(era) => era,
        Err(_) => panic!("Error selecting era"),
    }
}

fn select_gender() -> PlayerGender {
    let options: Vec<PlayerGender> =
        vec![PlayerGender::Male, PlayerGender::Female, PlayerGender::Coed];
    let ans: Result<PlayerGender, InquireError> =
        Select::new("Select the league gender,", options).prompt();
    match ans {
        Ok(gender) => gender,
        Err(_) => panic!("Error selecting gender"),
    }
}

//#[tailcall]
fn add_new_team(
    league: &mut League,
    thread: &mut ThreadRng,
    conn: &mut Connection,
    league_id: i64,
    first_team: bool,
) -> std::io::Result<()> {
    let result: std::io::Result<()>;
    // If this is the first team generated for the league, we display a different prompt to the user.
    let mut prompt_string = match first_team {
        true => "Enter the name of the first team",
        false => "Enter the name of the new team",
    };
    let current_teams = &league.teams;
    if !current_teams.is_empty() {
        let names = current_teams.iter().fold(String::new(), |acc, team| {
            let new_string = format!("\n{} {}", team.abrv, team.name);
            acc + &new_string
        });
        println!(
            "This league currently includes the following teams:{}\n",
            names
        );
    };
    let abrv_min_validator = MinLengthValidator::new(2);
    let abrv_max_validator = MaxLengthValidator::new(4);
    let name_validator = MinLengthValidator::new(3);
    // Each team must have a unique name and abbreviation, we loop until we receive one.
    loop {
        let name_input = Text::new(prompt_string)
            .with_validator(name_validator.clone())
            .prompt();

        let team_name = match name_input {
            Ok(name) => name.trim().to_string(),
            Err(_) => panic!("Error creating team name."),
        };

        let abrv_input = Text::new("Please enter an abbreviation for the new team.")
            .with_validator(abrv_min_validator.clone())
            .with_validator(abrv_max_validator.clone())
            .with_default(&team_name[0..=1].to_string().to_uppercase())
            .prompt();

        let abrv = match abrv_input {
            Ok(input) => input.trim().to_string(),
            Err(_) => panic!("Error creating team abbreviation."),
        };
        /*  The league takes the name and abreviation we just created. If there is already a team in the league with that name or abbreviation,  it returns an error.
        Otherwise, the league generates a new team, and then returns the team as a string wrapped in an Ok, which we use to save the team as a file on the disk.*/
        match league.new_team(&abrv, &team_name, thread, league_id, conn) {
            Err(message) => {
                match message {
                    AddTeamError::AbrvTaken => println!(
                        "This league already has a team with that abbreviation, please try again."
                    ),
                    AddTeamError::NameTaken => {
                        println!("This league already has a team with that name, please try again.")
                    }
                    AddTeamError::DatabaseError => {
                        println!("Error adding team to the data base, please try again.");
                        return Ok(());
                    }
                };
                //println!("Error {:?}",message);
                prompt_string = "Enter a unique team name.";
            }
            // If the league returns OK, we take the string, and write it to a new file in the leauge folder
            Ok(()) => {
               

                result = add_team_check(league, conn, thread, league_id);
                break;
            }
        };

       
    }

    result
}
// After creating a new team, we ask the user if they would like to create another team.
fn add_team_check(
    league: &mut League,
    conn: &mut Connection,
    thread: &mut ThreadRng,
    league_id: i64,
) -> std::io::Result<()> {
    let ans = Confirm::new("Would you like to create another team?")
        .with_default(true)
        .prompt();

    match ans {
        // If the user selects true, the user adds another team, however we note that this is not the first team created for the league.
        Ok(true) => add_new_team(league, thread, conn, league_id, false)?,
        //If not, we save the leauge and hten exit.
        Ok(false) => save_league(league,conn,thread)?,
        Err(_) => {
            panic!("Error on add team prompt");
        }
    };

    Ok(())
}

// Once a league is saved, we save a copy of the league data in a folder.
fn save_league(league: &League, conn: &mut Connection, thread: &mut ThreadRng) -> std::io::Result<()> {
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
    match ans{
        Ok(true) => run_main_menu(conn, thread),
        _ => Ok(())
    }
    
}

/*  The League Wrapper struct is used when the program checks to see what leagues are saved in the database.

 It contains the ID which the leagues is saved in the database, as well a desrtialzied League struct from the database
*/

struct LeagueWrapper {
    league_id: i64,
    league: League,
}

// We implement display for LeagueWrapper, as we will need to see print a list of all leeagues to the console when a user wants to open an existing leaghue
impl fmt::Display for LeagueWrapper {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}", self.league_id, self.league.name)
    }
}

fn league_check(conn: &mut Connection, thread: &mut ThreadRng, input: MenuInput) -> Result<(), rusqlite::Error> {
    // We query the database to get all the leagues that already exist.
    let mut stmt = conn.prepare("SELECT * from leagues").unwrap();
    // We wrap the rows into a LeagueWrapper that is part of a Rust Ok.
    let league_iter = stmt
        .query_map([], |row| {
            Ok(LeagueWrapper {
                league_id: row.get(0)?,
                league: League {
                    name: row.get(1)?,
                    era: serde_json::from_value(row.get(2)?).unwrap() ,
                    
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
            Ok(select) => {
                match input{
                    MenuInput::CreateNewTeam => {
                        load_league(thread, conn, select)?;
                        Ok(())
                    },
                    MenuInput::RefreshLeague =>{
                        println!("Refreshing league.");
                        save_league(&select.league,conn,thread).unwrap();
                        Ok(())
                    },
                    _ => panic!("Invalid Menu Input:{:?}",input)
                }
                
            }
            Err(_) => {
                println!("Error selecting a new league");
                Ok(())
            }
        }
    }
}




fn main() -> std::io::Result<()> {
    let mut conn = load_database().unwrap();
    let mut r_thread = rand::thread_rng();
    

    println!("Welcome to the Deadball Team generator!");

    run_main_menu(&mut conn,&mut r_thread)
    
    
}

fn load_database() -> Result<Connection, rusqlite::Error> {
    // We look for the database, and create a new one if it doesn't exist. If no database exists and there we are unable to create a new database in the folder, the function returns an error
    let conn = Connection::open("deadball.db")?;
    // We create the league table in the database. Eeach league has an ID and a unique name. Each league also has an era and gender, which are used in creating teams and players withing the league.
    conn.execute(
        "create table if not exists leagues (
             id INTEGER PRIMARY KEY,
             league_name TEXT NOT NULL UNIQUE,
             era TEXT NOT NULL,
             gender TEXT NOT NULL
         )",
        (),
    )?;
    /*  Leagues have a one to many relationship to teams. Each team has its own id, as well as a foreign key that references the league id.
        Each team also has a name, and an abbreviation of their name. For example, if  you wanted to create a team named after the Los Angeles Dodgers, the abreviation would be LAD.
        If a team is generated via the program, the program will not let you have multiple teams in the same league with the same name and/or abbreviation.
        Teams also have a team score, which is a number that summarizes how good a team is based off the quality of their players, as well as current wins and losses*/
    conn.execute(
        "create table if not exists teams (
             id INTEGER PRIMARY KEY,
             league_id INTEGER NOT NULL,
             team_name TEXT NOT NULL,
             abrv TEXT NOT NULL,
             team_score INTEGER NOT NULL,
             wins INTEGER DEFAULT 0,
             losses INTEGER DEFAULT 0,
             FOREIGN KEY(league_id) REFERENCES leagues(id)
         )",
        (),
    )?;
    /*  The last table to create is the players tables. Teams have a one to many relationship with players, with each player beloning to one team
        Each player has a unique id, and a forein key team_id which refrenceses the id of the team the player belongs to

        In Deadball, players have the chance to gain traits in the following categories: contact, defense, power,speed, and toughness, with each trait indicating if a player is average, above average, or below average in the categorie.
        In Rust, a players traits are stored in a struct name Batter Traits, with each trait beingr represented in an enum. The contents of the struct are serialized onto the players table, however the way enums are serialized is confusing to look at insided the database.
        Thus, we spilt each trait into 2 rows on the palyer table. The first column is named after the trait itself. If hte trait for hte player is average, then the value will be NULL, otherwise the value will be the players trait in a straightforward text format.
        The next column is [name of trait]_enum, and that contains data that can be converted to the correct enum when the row is deseiralized
    */
    conn.execute(
        "create table if not exists players(
             id INTEGER PRIMARY KEY,
             team_id INTEGER NOT NULL,
             player_name TEXT NOT NULL, -- Player's Name
             age INTEGER NOT NULL, --Players Age
             pos TEXT NOT NULL, --Player's position, E.G Shortstop, Right Fielder, Pitcher, etc.
             hand TEXT NOT NULL, --Notes if a player bats left handed, right handed, or if the player is not a pitcher, bats as a switch hitter.
             bt INTEGER NOT NULL, -- Players batting target, which is an apporximation of a players batting average.
             obt_mod INTEGER NOT NULL, --OBT Modifier, which is used to calculate a players on base target by addition to a player batter target
             obt INTEGER NOT NULL, -- On base Target, indicates how often a player get's on base. Correlates to a player on base percentage in real life.
             PD TEXT , -- If a player is a pitcher, they are assigned a pitch die, which represents the stand rpg die, E.G. d12, d4. Pitch die can be negative.
             pd_int INTEGER , /*If a player has a ptich die, PD_int represents the outcome of a pitch die roll that is the fartherst away from 0.
             For example, if a pitcher has a pd of d12, their pd_int would be 12, while a -d4 would be -4.
            */
             pitcher_trait TEXT , --Pitchers
             team_spot TEXT NOT NULL, -- Repreresents a player's rolke 
             contact TEXT ,
             contact_enum TEXT NOT NULL,
             defense TEXT,
             defense_enum TEXT NOT NULL,
             power TEXT ,
             power_enum TEXT NOT NULL,
             speed TEXT ,
             speed_enum TEXT NOT NULL,
             toughness TEXT,
             toughness_enum TEXT NOT NULL,
             FOREIGN KEY(team_id) REFERENCES teams(id)
         )",
        (),
    )?;

    // If no errors occured, the database is returned.
    Ok(conn)
}
