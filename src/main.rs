mod b_traits;
mod edit_league_error;
mod era;
mod league;
mod league_template;
mod lineup_score;
mod main_menu;
mod minor_leaguer;
mod note;
mod pd;
mod pennantgen;
mod pitcher_rank_info;
mod player;
mod player_error;
mod player_quality;
mod player_serde;
mod sched_view;
mod schedule;
mod team;
mod tier;
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
use edit_league_error::EditLeagueError;
use inquire::Confirm;
use inquire::InquireError;
use league::league_check;
use rand::rngs::ThreadRng;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Checks an inquire error to see if it is the result of the user cancelling. If not, returns the Error wrapped in an EditLeagueError
/// This is usefull, as it allows the user to esc from a prompt without causing the program to crash
pub fn inquire_check(err: InquireError) -> Result<(), EditLeagueError> {
    match err {
        inquire::InquireError::OperationCanceled => Ok(()),
        _ => Err(EditLeagueError::Inquire(err)),
    }
}

// Takes a vec of type e,  returns a hash map of each value with a result of true.
pub fn vec_to_hash<E: std::hash::Hash + std::cmp::Eq>(vec: &[E]) -> HashMap<&E, bool> {
    let mut result = HashMap::new();
    for value in vec.iter() {
        result.insert(value, true);
    }
    result
}
fn main() -> Result<(), ()> {
    // First, we load the databsae, or create one if it doesn't exist.
    let default_path = "deadball.db";
    let conn_load = load_database(default_path);
    let mut conn = match conn_load {
        Ok(connection) => connection,
        Err(_) => {
            println!(
                "Unable to create database under {}, please check if the folder is read only.",
                default_path
            );
            return Ok(());
        }
    };
    //Next we generate a thread for the random numbers we will need to generate.
    let mut r_thread = rand::thread_rng();

    println!("Welcome to the Deadball league generator!");
    println!("This tool is based off the Deadball tabletop game by W.M. Akers.");
    // We go to the main menu.
    let mut user_input = run_main_menu(&mut conn, &mut r_thread);
    loop {
        // We take the result of what the user choose. If there is an error, we break
        match user_input {
            Ok(_) => {}
            Err(ref message) => {
                match message {
                    EditLeagueError::DatabaseError(output) => {
                        println!("Something went wrong, please check that this folder is not read only.\nIt is also possible that {} is corrupted, please check that as well\nThe Error message was {}",default_path,output);
                    }
                    EditLeagueError::PennantError(message) => {
                        println!("{}", message)
                    }
                    EditLeagueError::Inquire(_) | EditLeagueError::SerdeError(_) => {
                        let library_type;
                        let output_string = match message {
                            EditLeagueError::SerdeError(output) => {
                                library_type = "serde";
                                output.to_string()
                            }
                            EditLeagueError::Inquire(output) => {
                                library_type = "inquire";
                                output.to_string()
                            }
                            // We break, as there are only 2 options that can lead to this path
                            _ => unreachable!(),
                        };
                        println!("Something went wrong with the {} library, please make sure that all dependencies have been installed correctly",library_type);
                        println!("The error message was: {}", output_string)
                    }
                    _ => println!("Something went wrong, please restart the program and try again"),
                }
            }
        }
        //If there was no error, we ask the user if they would like to return to the main menu.
        let ans = Confirm::new("Would you like to return to the main menu?")
            .with_default(true)
            .prompt();
        match ans {
            // If the user selects yes, we go back to the main menu, otherwise we break the loop
            Ok(true) => user_input = run_main_menu(&mut conn, &mut r_thread),
            _ => break,
        };
    }
    Ok(())
}

fn load_database(path: &str) -> Result<Connection, rusqlite::Error> {
    // We look for the database, and create a new one if it doesn't exist. If no database exists and there we are unable to create a new database in the folder, the function returns an error
    let conn = Connection::open(path)?;
    // We create the league table in the database. Each league has an ID and a unique name. Each league also has an era and gender, which are used in creating teams and players withing the league.
    conn.execute(
        "create table if not exists leagues (
             league_id INTEGER PRIMARY KEY,
             league_name TEXT NOT NULL UNIQUE,
             era TEXT NOT NULL,
             gender TEXT NOT NULL,
             league_note TEXT
         )",
        (),
    )?;
    /*  Leagues have a one to many relationship to teams. Each team has its own id, as well as a foreign key that references the league id.
    Each team also has a name, and an abbreviation of their name. For example, if  you wanted to create a team named after the Los Angeles Dodgers, the abbreviation would be LAD.
    If a team is generated via the program, the program will not let you have multiple teams in the same league with the same name and/or abbreviation.
    Teams also have a team score, which is a number that summarizes how good a team is based off the quality of their players, as well as current wins and losses*/
    conn.execute(
        "create table if not exists teams (
             team_id INTEGER PRIMARY KEY,
             league_id INTEGER NOT NULL,
             team_name TEXT NOT NULL,
             abrv TEXT NOT NULL,
             wins INTEGER DEFAULT 0,
             losses INTEGER DEFAULT 0,
             team_note TEXT,
             FOREIGN KEY(league_id) REFERENCES leagues(league_id)
         )",
        (),
    )?;
    /*  The last table to create is the players tables. Teams have a one to many relationship with players, with each player belonging to one team
        Each player has a unique id, and a foreign key team_id which references the id of the team the player belongs to


    */
    conn.execute(
        "create table if not exists players(
             player_id INTEGER PRIMARY KEY,
             team_id INTEGER NOT NULL,
             player_name TEXT NOT NULL, -- Player's Name
             age INTEGER NOT NULL, --Players Age
             pos TEXT NOT NULL, --Player's position, E.G Shortstop, Right Fielder, Pitcher, etc.
             hand TEXT NOT NULL, --Notes if a player bats left handed, right handed, or if the player is not a pitcher, bats as a switch hitter.
             bt INTEGER NOT NULL, -- Players batting target, which is an approximation of a players batting average.
             obt_mod INTEGER NOT NULL, --OBT Modifier, which is used to calculate a players on base target by addition to a player batter target
             obt INTEGER NOT NULL, -- On base Target, indicates how often a player get's on base. Correlates to a player on base percentage in real life.
             PD TEXT , -- If a player is a pitcher, they are assigned a pitch die, which represents the stand rpg die, E.G. d12, d4. Pitch die can be negative.
             pd_int INTEGER , /*If a player has a pitch die, PD_int represents the outcome of a pitch die roll that is the farthest away from 0.
             For example, if a pitcher has a pd of d12, their pd_int would be 12, while a -d4 would be -4.
            */
             pitcher_trait TEXT , -- Traits for pitchers
             team_spot TEXT NOT NULL, -- Represents where a player is on a team. E.G are they starting lineup or in the bullpen.
             contact TEXT ,
             defense TEXT,
             power TEXT ,
             speed TEXT ,
             toughness TEXT,
             trade_value INTEGER NOT NULL,
             player_note TEXT,
             FOREIGN KEY(team_id) REFERENCES teams(team_id)
         )",
        (),
    )?;
    // We also generate a season table.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS seasons(

        season_id INTEGER PRIMARY KEY,
        league_id INTEGER NOT NULL,
        champion_id INTEGER,
        FOREIGN KEY(league_id) REFERENCES leagues(league_id),
        FOREIGN KEY(champion_id) REFERENCES teams(team_id))",
        (),
    )?;
    // As well as a table of rounds.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rounds(
        round_id INTEGER PRIMARY KEY,
        season_id INTEGER NOT NULL,
        FOREIGN KEY (season_id) REFERENCES seasons(season_id)
    )",
        (),
    )?;
    // Ans series that are part of a round.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS series(
    series_id INTEGER PRIMARY KEY,
    round_id INTEGER NOT NULL,
    home_team_id INTEGER NOT NULL,
    away_team_id INTEGER NOT NULL,
    FOREIGN KEY (round_id) REFERENCES rounds(round_id),
    FOREIGN KEY (home_team_id) REFERENCES teams(team_id),
    FOREIGN KEY (away_team_id) REFERENCES teams(team_id)
    )",
        (),
    )?;
    // And games that are part of a series.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS games(
        game_id INTEGER PRIMARY KEY,
        series_id INTEGER NOT NULL,
        home_score INTEGER DEFAULT 0,
        away_score INTEGER DEFAULT 0,
        game_not TEXT,
        FOREIGN KEY (series_id) REFERENCES series(series_id)
    )",
        (),
    )?;

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS pennants(
            pennant_id INTEGER PRIMARY KEY,
            league_id INTEGER NOT NULL,
            FOREIGN KEY (league_id) REFERENCES leagues(league_id)
        
        )
    ",
        (),
    )?;

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS pennants_standings(
        standings_id INTEGER PRIMARY KEY,
        pennant_id INTEGER NOT NULL,
        team_id INTEGER NOT NULL,
        wins INTEGER NOT NULL,
        losses INTEGER NOT NULL,
        FOREIGN KEY (pennant_id) REFERENCES pennants(pennant_id),
        FOREIGN KEY(team_id) REFERENCES teams(team_id)
        )
    
    ",
        (),
    )?;
    /*conn.execute("CREATE TABLE IF NOT EXISTS team_seasons(
    team_season_id INTEGER PRIMARY KEY,st
    league_season_id INTEGER,
    team_id INTEGER,
    wins INTEGER DEFAULT 0,
    losses INTEGER DEFAULT 0,


    ", params) */

    // If no errors occurred, the database is returned.
    Ok(conn)
}

// Tests if a player has a pd.
fn player_pd_test(player: &Player) -> bool {
    player.pd.is_some()
}

fn player_pool_test(input: &[Player], team_id: i64, for_pitchers: bool) {
    for player in input.iter() {
        assert_eq!(player.bt + player.obt_mod, player.obt);
        assert_eq!(player.team_id, team_id);
        assert_eq!(player_pd_test(player), for_pitchers);
        // Next, we check to ensure that there are no errors from loading the player from SQL.
        let pd_int = match player.pd {
            Some(die) => die.to_int(),
            None => 0,
        };
        let player_err = player.get_player_error(pd_int);
        assert!(player_err.is_none());
    }
}
/* Used for testing schedules. Commented for now untill we have a better solution for scheduling.
//fn how_many_rounds(number_of_teams: i32, series_per_matchup: i32) -> i32 {
   // (number_of_teams - 1) * series_per_matchup
}*/

#[cfg(test)]
mod tests {

    use b_traits::BTraits;
    use league::{get_all_leagues_from_db, load_teams_from_sql, BatterPosType};
    use league_template::{load_league_templates, new_league_from_template};

    /// Used to test Leagues in database.
    struct LeagueListing {
        name: String,
        id: i64,
    }
    use super::*;

    #[test]
    fn generate_db() {
        // WARNING: This will automatically fail if there is a test.db in the folder, as well as if there are folders named PCL_1,PCL_2,or PCL_3.

        // We create a test database
        let mut test_conn = load_database("test.db").unwrap();
        let mut r_thread = rand::thread_rng();
        // And get a league template.
        let templates = load_league_templates();
        let first = &templates[0];
        // We use the template to create a new league in the database multiple times
        for _ in 1..=3 {
            new_league_from_template(&mut test_conn, &mut r_thread, &first).unwrap();
        }
        // We query for the league ids and names.
        let mut league_stmt = test_conn
            .prepare(
                "
            SELECT 
                leagues.league_name, leagues.league_id
            FROM 
                leagues
            ORDER BY 
                leagues.league_id ASC;
        
        ",
            )
            .unwrap();

        // And use the data to create a vector of league listing.
        let league_iter = league_stmt
            .query_map([], |row| {
                Ok(LeagueListing {
                    name: row.get(0).unwrap(),
                    id: row.get(1).unwrap(),
                })
            })
            .unwrap();
        let mut test_vec = Vec::new();
        for listing in league_iter {
            test_vec.push(listing.unwrap())
        }
        // We test to see if the database contains the number of leagues we expect
        let test_vec_length = test_vec.len();
        assert_eq!(test_vec_length, 3);
        // As well as if they fit the template name pattern we expect as wel
        for i in 1..=3 {
            let test_string = format!("PCL_{}", i);
            let current_listing = &test_vec[i - 1];
            assert_eq!(current_listing.name, test_string)
        }
        // We drop the stmt so we can use conn later.
        drop(league_stmt);
        // We load all the leagues in the database as LeagueWrappers.
        let mut all_league_wrappers = get_all_leagues_from_db(&mut test_conn).unwrap();
        // And check to make sure the number is what we are expecting
        assert_eq!(all_league_wrappers.len(), 3);

        // We select the first league in the vector.
        let mut current_league = &mut all_league_wrappers.remove(0).league;
        // And check that it's name and league_id are what we expect.
        assert_eq!(current_league.name, "PCL_1");
        assert_eq!(current_league.league_id, 1);
        // Next, we load the the teams in the league from the database which are inserted into the league struct.
        load_teams_from_sql(
            current_league.league_id,
            &mut current_league,
            &mut test_conn,
        )
        .unwrap();
        // We test to make sure the length of league.teams is what we expect.
        assert_eq!(current_league.teams.len(), 8);
        // Next, we select the
        let first_team = current_league.teams.get(0).unwrap();
        let first_team_id = first_team.team_id;
        //Next we check the team's player pools to make sure they have all the players we expect.
        assert_eq!(first_team.lineup.len(), 8);
        // And then check that the player structs data matches what we expect.
        player_pool_test(&first_team.lineup, first_team_id, false);
        assert_eq!(first_team.bench.len(), 5);
        player_pool_test(&first_team.bench, first_team_id, false);
        assert_eq!(first_team.starting_pitching.len(), 5);
        player_pool_test(&first_team.starting_pitching, first_team_id, true);
        // For bullpen, there are multiple steps, as Ancient teams do not have a bullpen.
        match &first_team.bullpen {
            Some(pen) => {
                assert_eq!(pen.len(), 7);
                player_pool_test(pen, first_team_id, true);
            }
            None => match current_league.era {
                Era::Ancient => {}
                Era::Modern => panic!("Expected a bullpen for a modern team"),
            },
        }
        // Next, we check to make sure that a league will not allow the addition of a team with the same name as an existin team.
        let double_name_check = current_league.new_team(
            &"NY".to_string(),
            &"Los Angeles Angels".to_string(),
            &mut r_thread,
            1,
            &mut test_conn,
        );
        assert!(double_name_check.is_err());
        let double_abrv_check = current_league.new_team(
            &"LA".to_string(),
            &"Los Angeles Gladiators".to_string(),
            &mut r_thread,
            1,
            &mut test_conn,
        );
        assert!(double_abrv_check.is_err());
        println!("Now lets check the leaderboards");
        current_league
            .display_top_hitters(&mut test_conn, None)
            .unwrap();
        println!("Now the pitcher leaderboard");
        current_league.display_top_pitchers(&mut test_conn).unwrap();
        let power_check = serde_json::to_string(&Power::P2).unwrap();
        assert_eq!(power_check, "\"P++\"");
        let manual_power: Power = serde_json::from_str("\"P++\"").unwrap();
        let power = BTraits::from_string("P++").unwrap();
        assert_eq!(BTraits::from_string("P+++++").is_err(), true);
        BTraits::from_string("C+").unwrap();
        BTraits::from_string(" C+").unwrap();
        BTraits::from_string("S+").unwrap();
        assert_eq!(BTraits::from_string("P +").is_err(), true);
        BTraits::from_string("C+,P--").unwrap();
        current_league
            .create_pennant_race(&mut r_thread, &mut test_conn, 144)
            .unwrap();
        /*let series_per_matchup = 6;
        /let test_sched = new_schedule(&current_league.teams, 3, series_per_matchup);
        assert_eq!(
            test_sched.len() as i32,
            how_many_rounds(current_league.teams.len() as i32, series_per_matchup)
        );
        schedule_to_sql(&mut test_conn, &current_league, test_sched).unwrap();*/
    }
}
