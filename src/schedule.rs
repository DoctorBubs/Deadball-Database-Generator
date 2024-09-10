use std::collections::HashMap;

use crate::inquire_check;
use crate::league::{save_league, League};
use crate::team::Team;
use itertools::Itertools;
//use serde::ser;
use inquire::{CustomType, InquireError};
use rand::prelude::IteratorRandom;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rusqlite::Connection;
//use inquire::validator::{I, Validation};
//Ok, to generate a whole season, we will start with the smaller elements and build up from there.

//First we have the game struct, which represents an individual game that is scheduled
#[derive(Clone, Debug)]
pub struct Game {
    home_team_id: i64,
    away_team_id: i64,
}

fn new_game(home_team_id: i64, away_team_id: i64) -> Game {
    Game {
        home_team_id,
        away_team_id,
    }
}

// Next we have a series, which is a small collection of games.
#[derive(Clone, Debug)]
pub struct Series {
    games: Vec<Game>,
    home_team_id: i64,
    away_team_id: i64,
}

impl Series {
    /// When given a vector of ints, this function returns true if the series home team and away team id are no in the vector.
    fn is_valid(&self, forbidden_numbers: &HashMap<i64, bool>) -> bool {
        (forbidden_numbers.get(&self.home_team_id).is_none())
            & (forbidden_numbers.get(&self.away_team_id).is_none())
    }
}
fn new_series(home_team_id: i64, away_team_id: i64, series_length: i32) -> Series {
    Series {
        games: vec![new_game(home_team_id, away_team_id); series_length as usize],
        home_team_id,
        away_team_id,
    }
}

#[derive(Clone, Debug)]
pub struct Round {
    series: Vec<Series>,
}

/// SeriesListing is a struct that contains a refrence to a series, as well as an index field that marks under what position the series is in a vector.
#[derive(Debug)]
struct SeriesListing<'a> {
    series: &'a Series,
    index: usize,
}

// Generates a schedule based off a vec of series
pub fn new_round_generator(mut all_series: Vec<Series>, _series_per_round: i32) -> Vec<Round> {
    let mut result = vec![];

    // We loop untill the length of all series is 0
    while !all_series.is_empty() {
        // We create a vector for the new round.
        let mut new_round_vec = vec![];
        //We create a vector of forbidden team id's that have already been used.
        let mut forbidden_numbers = HashMap::new();

        //First, we choose a random series from all series
        let (i, _) = all_series
            .iter()
            .enumerate()
            .choose(&mut thread_rng())
            .unwrap();
        // And rempove it from alll series.
        let first_series = all_series.remove(i);
        // And add the series home team and away team id to the forbidden numbers.
        forbidden_numbers.insert(first_series.home_team_id, true);
        forbidden_numbers.insert(first_series.away_team_id, true);
        // we then add the series to the round vec
        new_round_vec.push(first_series);

        //We loops from 1 to matchup_per_round. This previously checked the length of the new round, however this casued bugs that loopingl ike this might fix
        loop {
            // If we havce enough series in the round, we brea

            let filtered_series_listing: Vec<SeriesListing> = all_series
                .iter()
                // We set the iter to enumerate, as we need the index to generate a SeriesListing.
                .enumerate()
                //From the clone we map to create a series listing. The listing contains an index, which represents the index of the series in all series.
                .map(|(index, series)| SeriesListing { series, index })
                // We then filter for only series listing that series is valid based off the current forbidden numbers
                .filter(|x| x.series.is_valid(&forbidden_numbers))
                // And we collect the new vector.
                .collect();

            //If there are no valid series listing, we panic.
            if filtered_series_listing.is_empty() {
                println!("Round Created with {} series", new_round_vec.len());
                break;
            }
            // We choose a series listing from filtered series listing
            let (_i, current_series_listing) = filtered_series_listing
                .iter()
                .enumerate()
                .choose(&mut thread_rng())
                .unwrap();

            // We get the current series from all series based off the current_series_listing_index.
            let current_series = all_series.remove(current_series_listing.index);

            // We add the series home and away team ids to the forbidden numbers.
            forbidden_numbers.insert(current_series.home_team_id, true);
            forbidden_numbers.insert(current_series.away_team_id, true);
            // And we add the current series to the round
            new_round_vec.push(current_series);
        }
        // Once the vector is full, we use it to create a new round.
        let new_round = Round {
            series: new_round_vec,
        };
        // We then add the round to the result.
        result.push(new_round);
    }

    result
}

pub fn new_schedule(teams: &[Team], series_length: i32, series_per_matchup: i32) -> Vec<Round> {
    let ids: Vec<i64> = teams.iter().map(|team| team.team_id).collect();
    let series_per_round = (teams.len() / 2) as i32;
    let home_series = series_per_matchup / 2;
    println!("home series = {}", home_series);

    let all_series: Vec<Series> = ids
        .into_iter()
        // We create a permutation of home team and away id.
        .permutations(2)
        .map(|vec| new_series(vec[0], vec[1], series_length))
        .collect();
    for series in all_series.iter() {
        println!("{} @ {}", series.home_team_id, series.away_team_id)
    }
    // println!("home_matchups_created = {}",home_matchups_created );
    //println!("Total Series Created = {}",all_series.len());

    let mut result: Vec<Round> = Vec::new();
    for _ in 0..(series_per_matchup / 2) {
        let series_clone = all_series.clone();
        let mut rounds = new_round_generator(series_clone, series_per_round);

        rounds.shuffle(&mut thread_rng());
        result.append(&mut rounds);
    }
    println!("Total Rounds generated = {}", result.len());
    result
}

// Asks the user for a number. It ensures the number is positive, and if force_even is true ensures the number is even
fn get_valid_number(message: &str, force_even: bool) -> Result<i32, InquireError> {
    //
    loop {
        let input = CustomType::<i32>::new(message)
            .with_error_message("Please type a valid number")
            .prompt()?;
        match (!force_even | (input % 2 == 0)) & (input > 0) {
            true => return Ok(input),
            false => {
                let error_message = match force_even {
                    true => "\nInput must be an even positive whole number.",
                    false => "\nInput must be a positive whole number.",
                };
                println!("{}", error_message);
            }
        }
    }
}

pub fn schedule_from_input(league: &League) -> Result<Vec<Round>, InquireError> {
    let series_number = get_valid_number(
        "Please enter how many series should be played between each team.",
        true,
    )?;
    println!();
    let series_length = get_valid_number(
        "Please enter how many games should be played in each series.",
        false,
    )?;
    let teams = &league.teams;
    Ok(new_schedule(teams, series_length, series_number))
}

pub fn schedule_to_sql(
    conn: &mut Connection,
    league: &League,
    sched: Vec<Round>,
) -> Result<(), rusqlite::Error> {
    let league_id = league.league_id;
    conn.execute("INSERT INTO seasons(league_id) VALUES(?1)", [league_id])
        .unwrap();
    let season_id = conn.last_insert_rowid();
    for round in sched {
        conn.execute("INSERT INTO rounds(season_id) VALUES(?1)", [season_id])
            .unwrap();
        let round_id = conn.last_insert_rowid();
        for series in round.series {
            let home_id = series.home_team_id;
            let away_id = series.away_team_id;
            conn.execute(
                "INSERT INTO series(round_id,home_team_id,away_team_id) VALUES(?1, ?2, ?3)",
                [round_id, home_id, away_id],
            )
            .unwrap();
            let series_id = conn.last_insert_rowid();
            for _game in series.games {
                conn.execute("INSERT INTO games(series_id) VALUES(?1)", [series_id])
                    .unwrap();
            }
        }
    }

    Ok(())
}

pub fn save_schedule_sql(
    conn: &mut Connection,
    league: &League,
    thread: &mut ThreadRng,
) -> Result<(), rusqlite::Error> {
    let sched_input = schedule_from_input(league);
    let sched = match sched_input {
        Ok(rounds) => rounds,
        Err(message) => return inquire_check(message),
    };
    schedule_to_sql(conn, league, sched)?;
    save_league(league, conn, thread).unwrap();
    Ok(())
}
