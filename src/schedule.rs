use crate::league::{save_league, League};
use crate::team::Team;
use itertools::Itertools;
//use serde::ser;
use inquire::CustomType;
use rand::prelude::IteratorRandom;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rusqlite::Connection;
use inquire::validator::{I, Validation};
//Ok, to generate a whole season, we will start with the smaller elements and build up from there.

//First we have the game struct, which represents an individual game that is scheduled
#[derive(Clone, Debug)]
pub struct Game {
    home_team_id: i32,
    away_team_id: i32,
}

fn new_game(home_team_id: i32, away_team_id: i32) -> Game {
    Game {
        home_team_id,
        away_team_id,
    }
}

// Next we have a series, which is a small collection of games.
#[derive(Clone, Debug)]
pub struct Series {
    games: Vec<Game>,
    home_team_id: i32,
    away_team_id: i32,
}

impl Series {
    fn is_valid(&self, forbidden_numbers: &Vec<i32>) -> bool {
        if forbidden_numbers.contains(&self.home_team_id)
            | forbidden_numbers.contains(&self.away_team_id)
        {
            return true;
        } else {
            return false;
        }
    }
}

fn new_series(home_team_id: i32, away_team_id: i32, series_length: i32) -> Series {
    Series {
        games: vec![new_game(home_team_id, away_team_id); series_length as usize],
        home_team_id: home_team_id,
        away_team_id: away_team_id,
    }
}

#[derive(Clone, Debug)]
pub struct Round {
    series: Vec<Series>,
}

#[derive(Debug)]
struct SeriesListing {
    series: Series,
    index: usize,
}

// Generates a schedule based off a vec of series
pub fn new_round_generator(mut all_series: Vec<Series>, matchups_per_round: usize) -> Vec<Round> {
    let mut result = vec![];

    // We loop untill the length of all series is 0
    while all_series.len() > 0 {
        // We create a vector for the new round.
        let mut new_round_vec = vec![];
        //We create a vector of forbidden team id's that have already been used.
        let mut forbidden_numbers = vec![];
        // We clone all_series.
        let series_clone = all_series.clone();
        // And choose a random series from the vector.
        let (i, first_series) = series_clone
            .iter()
            .enumerate()
            .choose(&mut thread_rng())
            .unwrap();
        // We remove the index of the series.
        all_series.remove(i);
        // And add the sereis home team and away team id to the forbidden numbers.
        forbidden_numbers.push(first_series.home_team_id);
        forbidden_numbers.push(first_series.away_team_id);
        // we then add the series to the round vec
        new_round_vec.push(first_series.clone());
        // We then loop untill the lenght of the new round vec is equal to matchups per round
        while new_round_vec.len() < matchups_per_round {
            // We create an index variable.
            let mut index = 0;
            // We clone series again.
            let series_clone = all_series.clone();
            let filtered_series_listing: Vec<SeriesListing> = series_clone
                .iter()
                //From the clone we map to create a series listing. The listing contains an index, which represents the index of the series in all series
                .map(|x| {
                    let result = SeriesListing {
                        series: x.clone(),
                        index,
                    };
                    index += 1;
                    result
                })
                // We then filter for only series listing that series is valid based off the current forbidden numbers
                .filter(|x| x.series.is_valid(&forbidden_numbers))
                // And we collect the new vector.
                .collect();
            // We select a random series listing  and it's index
            let (_i, current_series_listing) = filtered_series_listing
                .iter()
                .enumerate()
                .choose(&mut thread_rng())
                .unwrap();
            // We take the series listig index
            let current_index = current_series_listing.index;
            // And clone the series we need.
            let current_series = series_clone[current_index].clone();
            // We remove the series from all_series via the index
            all_series.remove(current_index);
            // We add the series home and away team ids to the forbidden numbers.
            forbidden_numbers.push(current_series.home_team_id);
            forbidden_numbers.push(current_series.away_team_id);
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

pub fn new_schedule(teams: &Vec<Team>, series_length: i32, series_per_matchup: i32) -> Vec<Round> {
    let ids: Vec<i32> = teams.iter().map(|team| team.team_id).collect();

    let team_size = ids.len();

    let all_series: Vec<Series> = ids
        .into_iter()
        // We create a permutation of home team and away id.
        .permutations(2)
        .fold(Vec::new(), |mut acc, e| {
            for _ in 0..series_per_matchup / 2 {
                let gen_series = new_series(e[0], e[1], series_length);
                acc.push(gen_series)
            }
            acc
        });
    let mut rounds = new_round_generator(all_series, team_size / 2);
    rounds.shuffle(&mut thread_rng());
    rounds
}


// Determines how 
fn get_valid_series_number() -> i32 {
    //
    loop {
        let input = CustomType::<i32>::new(
            "Please enter how many series between each team should be scheduled.",
        )
        .with_error_message("Please type a valid number")
        .prompt()
        .unwrap();
        match (input % 2 == 0) & (input > 0) {
            true => return input,
            false => {
                println!("Input must be an even positive whole number.");
                ()
            }
        }
    }
}

pub fn schedule_from_input(league: &League) -> Vec<Round> {
    let series_number = get_valid_series_number();
    let series_length =
        CustomType::<i32>::new("Please enter how many games should be played in each series.")
            .with_error_message("Please type a valid number")
            .prompt()
            .unwrap();
    let teams = &league.teams;
    new_schedule(teams, series_length, series_number)
}

pub fn save_schedule_sql(conn: &mut Connection, league: &League, thread: &mut ThreadRng) {
    let sched = schedule_from_input(league);
    let league_id = league.league_id;
    conn.execute("INSERT INTO seasons(league_id) VALUES(?1)", [league_id])
        .unwrap();
    let season_id = conn.last_insert_rowid();
    for round in sched {
        conn.execute("INSERT INTO rounds(season_id) VALUES(?1)", [season_id])
            .unwrap();
        let round_id = conn.last_insert_rowid();
        for series in round.series {
            for game in series.games {
                let home_id = game.home_team_id as i64;
                let away_id = game.away_team_id as i64;
                conn.execute(
                    "INSERT INTO games(round_id,home_team_id,away_team_id) VALUES(?1, ?2, ?3)",
                    [round_id, home_id, away_id],
                )
                .unwrap();
            }
        }
    }
    save_league(league, conn, thread).unwrap()
}
