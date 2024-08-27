use crate::league::{save_league, League};
use crate::team::Team;
//use std::collections::HashMap;
use itertools::Itertools;
//use serde::ser;
use inquire::CustomType;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rusqlite::Connection;
//Ok, to generate a whole season, we will start with the smaller elements and build up from there.

//First we have the game struct, which represents an individual game that is scheduled
#[derive(Clone, Debug)]
pub struct Game {
    home_team_id: i32,
    away_team_id: i32,
}

// NExt we have a series, which is a small collection of games.
#[derive(Clone, Debug)]
pub struct Series {
    games: Vec<Game>,
}

fn new_game(home_team_id: i32, away_team_id: i32) -> Game {
    Game {
        home_team_id,
        away_team_id,
    }
}

fn new_series(home_team_id: i32, away_team_id: i32, series_length: i32) -> Series {
    /*  let mut result = Vec::new();
    for _ in 1..=length{
        result.push(new_game(home_team_id, away_team_id))
    };

    result */
    Series {
        games: vec![new_game(home_team_id, away_team_id); series_length as usize],
    }
}

#[derive(Clone, Debug)]
pub struct Round {
    series: Vec<Series>,
}

pub fn round_from_vec(vec: Vec<i32>, series_length: i32) -> Round {
    let half_point = vec.len() / 2;
    let home_team_ids = vec[..half_point].to_vec();
    let away_team_ids = vec[half_point..].to_vec();
    let mut away_team_clone = away_team_ids.clone();
    let series = home_team_ids
        .iter()
        .map(|home_team_id| {
            let away_team_id = away_team_clone.pop().unwrap();
            new_series(*home_team_id, away_team_id, series_length)
        })
        .collect();
    Round { series }
}

pub fn new_schedule(teams: &Vec<Team>, series_length: i32) -> Vec<Round> {
    let ids: Vec<i32> = teams.iter().map(|team| team.team_id).collect();
    //let total_game_per_team = (series_length * series_per_matchup * ((ids.len() - 1) as i32)) * 2;
    //println!("{:?}",ids);
    let team_size = ids.len();

    let mut rounds: Vec<Round> = ids
        .into_iter()
        .permutations(team_size)
        .fold(Vec::new(), |mut acc, e| {
            for _ in [..series_length] {
                acc.push(e.clone())
            }
            acc
        })
        .into_iter()
        .map(|vec| round_from_vec(vec, series_length))
        .collect();
    rounds.shuffle(&mut thread_rng());
    rounds
}

fn get_valid_series_number() -> i32 {
    loop {
        let input = CustomType::<i32>::new(
            "Please enter how many series between each team should be scheduled.",
        )
        .with_error_message("Please type a valid number")
        .prompt()
        .unwrap();
        match input % 2 == 0 {
            true => return input,
            false => {
                println!("\nThe number of series must be even.");
                ()
            }
        }
    }
}

pub fn schedule_from_input(league: &League) -> Vec<Round> {
    let series_length =
        CustomType::<i32>::new("Please enter how many games should be played in each series.")
            .with_error_message("Please type a valid number")
            .prompt()
            .unwrap();
    let teams = &league.teams;
    new_schedule(teams, series_length)
}

pub fn save_schedule_sql(conn: &mut Connection, league: &League, thread: &mut ThreadRng) {
    let sched = schedule_from_input(league);
    let league_id = league.league_id;
    conn.execute("INSERT INTO seasons(league_id) VALUES(?1)", [league_id])
        .unwrap();
    let season_id = conn.last_insert_rowid();
    for round in sched {
        //println!("{:?}", round);
        conn.execute("INSERT INTO rounds(season_id) VALUES(?1)", [season_id])
            .unwrap();
        let round_id = conn.last_insert_rowid();
        for series in round.series {
            //println!("{:?}", series);
            for game in series.games {
                // println!("{:?}",game);
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
