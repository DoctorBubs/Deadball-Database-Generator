mod b_traits;
mod era;
mod league;
mod lineup_score;
mod main_menu;
mod pd;
mod pitcher_rank_info;
mod player;
mod player_quality;
mod schedule;
mod team;
mod traits;
use crate::era::Era;
use crate::main_menu::run_main_menu;
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
use inquire::Confirm;
use league::league_check;
use rand::rngs::ThreadRng;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

fn main() -> std::io::Result<()> {
    // First, we load the databse, or create one if it doesn't exist.
    let mut conn = load_database().unwrap();
    //Next we generate a thread for the random numbers we will need to generate.
    let mut r_thread = rand::thread_rng();

    println!("Welcome to the Deadball league generator!");
    println!("This tool is based off the Deadball tabletop game by W.M. Akers.");
    // We then go to the main menu.
    let mut user_input = run_main_menu(&mut conn, &mut r_thread);
    loop {
        match user_input {
            Err(_) => break,
            _ => {}
        };
        //We then prompt the user if they would like to return to the main menu
        let ans = Confirm::new("Would you like to return to the main menu?")
            .with_default(true)
            .prompt();
        match ans {
            Ok(true) => user_input = run_main_menu(&mut conn, &mut r_thread),
            _ => break,
        };
    }
    user_input
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
             defense TEXT,
             power TEXT ,
             speed TEXT ,
             toughness TEXT,
             trade_value INTEGER NOT NULL,
             FOREIGN KEY(team_id) REFERENCES teams(id)
         )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS seasons(

        season_id INTEGER PRIMARY KEY,
        league_id INTEGER NOT NULL,
        champion_id INTEGER,
        FOREIGN KEY(league_id) REFERENCES leagues(id),
        FOREIGN KEY(champion_id) REFERENCES teams(id))",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS rounds(
        round_id INTEGER PRIMARY KEY,
        season_id INTEGER NOT NULL,
        FOREIGN KEY (season_id) REFERENCES seasons(season_id)
    )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS games(
        game_id INTEGER PRIMARY KEY,
        round_id INTEGER NOT NULL,
        home_team_id INTEGER NOT NULL,
        home_score INTEGER DEFAULT 0,
        away_team_id INTEGER NOT NULL,
        away_score INTEGER DEFAULT 0,
        FOREIGN KEY (round_id) REFERENCES rounds(round_id),
        FOREIGN KEY (home_team_id) REFERENCES teams(id),
        FOREIGN KEY (away_team_id) REFERENCES teams(id)
    )",
        (),
    )?;

    /*conn.execute("CREATE TABLE IF NOT EXISTS team_seasons(
    team_season_id INTEGER PRIMARY KEY,
    league_season_id INTEGER,
    team_id INTEGER,
    wins INTEGER DEFAULT 0,
    losses INTEGER DEFAULT 0,


    ", params) */

    // If no errors occured, the database is returned.
    Ok(conn)
}
