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
//mod futures;
use crate::era::Era;
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
use chrono::prelude::*;
use inquire::*;
use rand::rngs::ThreadRng;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::SystemTime;

/*fn trimed_capital_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let result = input.trim().to_uppercase();
    result
} */

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

// To create a new league
fn create_new_league(thread: &mut ThreadRng, conn: &mut Connection) -> std::io::Result<()> {
    let mut league_name: String;
    let mut folder_path: &Path;
    let validator = MinLengthValidator::new(3);

    let league_input = Text::new("Enter the name for the new league")
        .with_validator(validator.clone())
        .prompt();
    league_name = match league_input {
        Ok(input) => input.trim().to_string(),
        Err(_) => panic!("Error creating a league name!"),
    };

    // We have the user select the era for the league.
    let era = select_era();
    // As well as the gender of the players for the league.
    let gender = select_gender();

    // We then create a league struct.

    let league_entry = conn.execute(
        "INSERT INTO leagues(league_name,era,gender) VALUES(?1, ?2, ?3)",
        [&league_name, &era.to_string(), &gender.to_string()],
    );

    match league_entry {
        Err(message) => panic!("{message}"),
        Ok(_) => (),
    };
    // Via last_inster_rowid, we get the SQl id for the new league
    let league_id = conn.last_insert_rowid();
    let mut new_league = League::new(&league_name, gender, era);
    println!("{} created", &league_name);
    //And then prompt the user to create the first team for the league.
    add_new_team(&mut new_league, thread, conn, league_id, true)
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
            Err(_) => panic!("Error creating team name"),
        };

        let abrv_input = Text::new("Please enter an abbreviation for the new team")
            .with_validator(abrv_min_validator.clone())
            .with_validator(abrv_max_validator.clone())
            .with_default(&team_name[0..=1].to_string().to_uppercase())
            .prompt();

        let abrv = match abrv_input {
            Ok(input) => input.trim().to_string(),
            Err(_) => panic!("Error creating team abrv"),
        };
        /*  The league takes the name and abreviation we just created. If there is already a team in the league with that name or abbreviation,  it returns an error.
        Otherwise, the league generates a new team, and then returns the team as a string wrapped in an Ok, which we use to save the team as a file on the disk.*/
        match league.new_team(&abrv, &team_name, thread, league_id, conn) {
            Err(message) => {
                match message {
                    AddTeamError::AbrvTaken => println!(
                        "This league already has a team with that abbreviation, please try again"
                    ),
                    AddTeamError::NameTaken => {
                        println!("This league already has a team with that name, please try again")
                    }
                    AddTeamError::DatabaseError => {
                        panic!("Error adding team to the data base")
                    }
                };
                //println!("Error {:?}",message);
                prompt_string = "Enter a unique team name";
            }
            // If the league returns OK, we take the string, and write it to a new file in the leauge folder
            Ok(team_string) => {
                /*let team_path = path.join(format!("{}.txt", team_name));
                let mut team_info = File::create(team_path)?;
                team_info.write_all(team_string.as_bytes())?;*/

                result = add_team_check(league, conn, thread, league_id);
                break;
            }
        };

        /* let team_path = path.join(format!("{}.txt", team_name));
        match team_path.exists() {
            true => {
                println!("A team with that name already exists for this leauge");
                prompt_string = "Enter a unique team name";
            }

            false => {
                let team = league.new_team(&team_name, thread);
                let mut team_info = File::create(team_path)?;
                team_info.write_all(team.to_string().as_bytes())?;
                league.add_team(team);
                result = add_team_check(league, path, thread);
                break;
            }
        }*/
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
        Ok(true) => add_new_team(league, thread, conn, league_id, false),
        //If not, we save the leauge and hten exit.
        Ok(false) => save_league(league),
        Err(_) => panic!("Error on add team prompt"),
    }
}

// Take's a league and a path, serializes the league to a json object, which is saved under path.league_info.text
fn save_league(league: &League) -> std::io::Result<()> {
    let current_time = Utc::now().to_string();
    println!("{}", &current_time);
    let flder_path_string = format!("{}", &league.name);
    let folder_path = Path::new(&flder_path_string);
    fs::create_dir_all(folder_path)?;

    for team in &league.teams {
        let file_path = folder_path.join(format!("{}.txt", team.name).as_str());
        let mut file = File::create(file_path)?;
        file.write_all(&team.to_string().as_bytes())?;
    }
    Ok(())
}

fn get_league_name() -> String {
    let validator = MinLengthValidator::new(3);
    let name_input = Text::new("Enter the name of the league you would like to add a team to.")
        .with_validator(validator)
        .prompt();

    match name_input {
        Ok(input) => input.trim().to_string(),
        Err(_) => panic!("Error creating league name"),
    }
}

// if a user wants to add a new team to an existing league, we check to see if we can find the league folder.

struct LeagueWrapper {
    id: i64,
    league: League,
}

fn league_check(conn: &mut Connection, thread: &mut ThreadRng) -> std::io::Result<()> {
    let mut stmt = conn.prepare("SELECT * from leagues").unwrap();
    let league_iter = stmt
        .query_map([], |row| {
            Ok(LeagueWrapper {
                id: row.get(0)?,
                league: League {
                    name: row.get(1)?,
                    era: Era::from_string(row.get(2)?),
                    gender: PlayerGender::from_string(row.get(3)?),
                    teams: Vec::new(),
                },
            })
        })
        .unwrap();

    let mut options = Vec::new();

    for wrapper in league_iter {
        options.push(wrapper.unwrap())
    }

    if options.len() == 0 {
        print!("No Leagues created yet!");
        Ok(())
    } else {
        todo!()
    }
}

fn load_league(thread: &mut ThreadRng, conn: &mut Connection) -> std::io::Result<()> {
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
    todo!()
}

//#[tailcall]
fn main() -> std::io::Result<()> {
    let mut conn = load_database().unwrap();

    let mut r_thread = rand::thread_rng();
    fs::create_dir_all("leagues")?;
    println!("Welcome to the Deadball Team generator");

    let starting_options: Vec<&str> = vec![
        "Create a new league.",
        "Add a new team to an existing league.",
    ];

    let starting_choice: Result<&str, InquireError> =
        Select::new("What would you like to do?", starting_options).prompt();

    match starting_choice {
        Ok(choice) => match choice {
            "Create a new league." => create_new_league(&mut r_thread, &mut conn),
            "Add a new team to an existing league." => league_check(&mut conn, &mut r_thread),
            _ => {
                println!("Error with starting choice");
                Ok(())
            }
        },

        Err(_) => {
            println!("Error matching first choice");
            Ok(())
        }
    }
}

fn load_database() -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open("league.db")?;

    conn.execute(
        "create table if not exists leagues (
             id integer primary key,
             league_name text not null,
             era text not null,
             gender text not null
         )",
        (),
    )?;

    conn.execute(
        "create table if not exists teams (
             id integer primary key,
             league_id integer not null,
             team_name text not null,
             abrv text not null,
             team_score int not null,
             wins int DEFAULT 0,
             losses integer DEFAULT 0
         )",
        (),
    )?;

    conn.execute(
        "create table if not exists players(
             id integer primary key,
             team_id integer not null,
             player_name text not null,
             age integer not null,
             pos integer not null,
             hand integer not null,
             bt integer not null,
             obt_mod integer not null,
             obt integer not null,
             PD text not null,
             pitcher_trait text not null,
             team_spot text not null,
             contact text not null,
             defense text not null,
             power text not null,
             speed text not null,
             toughness text not null
         )",
        (),
    )?;

    /*let batting_traits_gen = conn.execute(
        "create table if not exists batter_traits(
             id integer primary key,
             player_id integer not null,
             contact text not null,
             defense text not null,
             power text not null,
             speed text not null,
             toughness text not null
         )",
        (),
    );*/

    Ok(conn)
}
