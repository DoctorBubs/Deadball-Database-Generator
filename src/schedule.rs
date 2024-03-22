use crate::league::{self, save_league, League};
use crate::team::Team;
//use std::collections::HashMap;
use itertools::Itertools;
//use serde::ser;
use inquire::CustomType;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rusqlite::Connection;
//Ok, to generate a whole season, we will start with hte smaller elemnts and build up from there.

//First we have the game strruct, which represents an idvidual game that is scheduled
#[derive(Clone)]
pub struct Game {
    home_team_id: i32,
    away_team_id: i32,
}

// NExt we have a series, which is a small collection of games.
pub struct Series {
    home_team_id: i32,
    away_team_id: i32,
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
        home_team_id,
        away_team_id,
        games: vec![new_game(home_team_id, away_team_id); series_length as usize],
    }
}
/*
fn home_team_matchups(
    home_team_id: i32,
    away_team_id: i32,
    series_length: i32,
    series_per_matchup: i32,
) -> Vec<Series> {
    vec![
        new_series(home_team_id, away_team_id, series_length),
        series_per_matchup,
    ]
}
// And next we have the home schedule, which is a collection ofd series where a team is schedueld to play at home.
struct HomeSchedule {
    team_id: i32,
    matchups: HashMap<i32, Vec<Series>>,
}

impl HomeSchedule {
    pub fn available_matchups(&self) -> Vec<i32> {
        self.matchups.iter()
            .filter(|(key, value)| value.len() > 0)
            .map(|(key, value)| *key)
            .collect()
    }

    pub fn take_series(
        &mut self,
        posible_opponents: &Vec<i32>,
        taken_opponents: &Vec<i32>,
    ) -> Result<(i32,Series), ()> {
        let valid_series: Vec<i32> = self.matchups
            .iter()
            .filter(|(key, value)| {
                posible_opponents.contains(key) & !taken_opponents.contains(key) & (value.len() > 0)
            })
            .map(|(key,value)| *key)
            .collect();
        match valid_series.len() {
            0 => Err(()),
            _ =>{
                let(key) = valid_series[0];//shuffle valid key

                let mut list = self.matchups.get_mut(&key).unwrap();
                let last = list.pop().unwrap();
                Ok((key, last))
            }
        }
    }
}
fn new_home_season(
    team_id: i32,
    all_team_ids: &Vec<i32>,
    series_length: i32,
    series_per_matchup: i32,
) -> HomeSchedule {
    HomeSchedule {
        team_id,
        matchups: all_team_ids
            .iter()
            .filter(|x| x != team_id)
            .map(|x| {
                (
                    x,
                    home_team_matchups(team_id, x, series_length, series_per_matchup),
                )
            })
            .fold(HashMap::new(), |mut acc, e| {
                acc.insert(e.0, e.1);
                acc
            })
            .collect(),
    }
}

pub fn get_home_schedules(
    ids: &Vec<i32>,
    series_length: i32,
    series_per_matchup: i32,
) -> Vec<HomeSchedule> {
    ids.iter()
        .map(|x| new_home_season(x, &ids, series_length, series_per_matchup))
        .collect()
}
*/
pub struct Round {
    home_team_ids: Vec<i32>,
    away_team_ids: Vec<i32>,
    series: Vec<Series>,
}
/*
fn unique_series_filter(series_vec: Vec<Series>,number_of_teams; ) -> bool {
    let home_team_ids = series_vec.iter().map(|series| series.home_team_id).collect();
    let away_team_ids = series_vec.iter().map(|series| series.away_team_id).collect();
    for h_id in home_team_ids{
        if away_team_ids.contains(h_id){
            return false
        };

        for other_id in home_team_ids

    };
}

pub fn new_round(
    home_schedules: Vec<HomeSchedule>,
    half_teams: i32,
    round_id: i32,
) -> Result<Round, ()> {
    let valid_matchups: Vec<HomeSchedule> = home_schedules
        .filter(|hs| hs.available_matchups > 0)
        .collect();
    //we need to shuffle valid_matchups here
    let home_team_schedules = valid_matchups[0..=half_teams];
    let home_teams_ids = home_team_schedules
        .iter()
        .map(|sched| sched.team_id)
        .collect();
    let away_teams_ids = ids.iter.filter(|id| !home_teams_ids.contains(id)).collect();

    let taken_away_team_ids = Vec::new();
    series_vec = Vec::new();
    for schedule in home_teams.schedule {
        match schedule.take_series(&away_teams_ids, &taken_away_team_ids) {
            Ok(series) => {series_vec.push(series)
            taken_away_team_ids.push(ser)},
            Err(()) => return Err(()),
        }
    }

    Ok(Round {
        round_id,
        home_team_ids,
        away_team_ids,
        series: series_vec,
    })
}

fn compare_vec_hash(vec: Vec<i32>, map: HashMap<i32,i32>) -> bool{
    for num in vec{
        match map.get(&num){
            Some(_) => return false,
            None => ()
        }
    ;}
    true
}
*/
// The next 2 fn are the ones that work.
pub fn round_from_vec(vec: Vec<i32>, series_length: i32) -> Round {
    let half_point = vec.len() / 2;
    let home_team_ids = vec[..(half_point - 1)].to_vec();
    let away_team_ids = vec[half_point..].to_vec();
    let mut away_team_clone = away_team_ids.clone();
    let series = home_team_ids
        .iter()
        .map(|home_team_id| {
            let away_team_id = away_team_clone.pop().unwrap();
            new_series(*home_team_id, away_team_id, series_length)
        })
        .collect();
    Round {
        home_team_ids,
        away_team_ids,
        series,
    }
}

pub enum ScheduleGenError {
    UnevenNumberOfTeams(i32),
    UnevenNumberOfGames(i32),
}

pub fn new_schedule(teams: &Vec<Team>, series_length: i32, series_per_matchup: i32) -> Vec<Round> {
    let ids: Vec<i32> = teams.iter().map(|team| team.team_id).collect();
    //let total_game_per_team = (series_length * series_per_matchup * ((ids.len() - 1) as i32)) * 2;

    let team_size = ids.len();
    let team_num = ids.len() as i32;

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
            false => (),
        }
    }
}

pub fn schedule_from_input(league: &League) -> Vec<Round> {
    let series_per_matchup = get_valid_series_number() / 2;
    let series_length =
        CustomType::<i32>::new("Please enter how many games should be played in each series.")
            .with_error_message("Please type a valid number")
            .prompt()
            .unwrap();
    let teams = &league.teams;
    new_schedule(teams, series_length, series_per_matchup)
}

pub fn save_schedule_sql(conn: &mut Connection, league: &League, thread: &mut ThreadRng) {
    let sched = schedule_from_input(league);
    let league_id = league.league_id;
    conn.execute("INSERT INTO seasons(league_id) VALUES(?1)", [league_id])
        .unwrap();
    let season_id = conn.last_insert_rowid();
    for round in sched {
        conn.execute("INSERT INTO rounds(season_id) VALUES(?1)", [season_id]).unwrap();
        let round_id = conn.last_insert_rowid();
        for series in round.series {
            for game in series.games {
                let home_id = game.home_team_id as i64;
                let away_id = game.away_team_id as i64;
                conn.execute(
                    "INSERT INTO games(round_id,home_team_id,away_team_id) VALUES(?1, ?2, ?3)",
                    [round_id, home_id, away_id],
                ).unwrap();
            }
        }
    }
    save_league(league, conn, thread).unwrap()
}
/*
pub fn new_schedule(teams: &Vec<Team>, series_length: i32, series_per_season: i32) -> Vec<Round> {
    let ids: Vec<i32> = teams.iter().map(|team| team.team_id).collect();
    let all_series: Vec<Series> = ids.iter().permutations(2).map(|vc| new_series(*vc[0], *vc[1], series_length))
    .collect();
    //let all_rounds = all_series.iter().permutations(teams.len() / 2);


    for _ in 1..=series_per_season{

        let map = HashMap::new();
        //shuffle vec
        let first = all_series.pop.unwrap();
        let base = vec![first.home_team_id, first.away_team_id];
        for key in base{
            map.insert(key,1)
        };
    }
   /*  let series_per_matchup = series_per_season / 2;
    let home_schedules = get_home_schedules(ids, series_length, series_per_matchup);
    let half_teams = teams.len() / 2;
    for i in 0..=series_per_season {
        let valid_matchups: Vec<HomeSchedule> = home_schedules
            .filter(|hs| hs.available_matchups > 0)
            .collect();
        //shuffle valid_matchups;
        let home_teams = valid_matchups[0..=half_teams];
        let home_teams_ids = home_teams.iter().map(|sched| sched.team_id).collect();
        let away_teams_ids = ids.iter.filter(|id| !home_teams_ids.contains(id)).collect();

        let taken_away_team_ids = Vec::new();
    }

    */
    /*let first = home_schedules[0];
    let first_matchups = first.available_matchups(); */
}
// psuedocode, take an array of numbers, use a permutation with the whole vec.
// then we can split hte vec in halves, with the first half being the home team, and and second being away tem

*/
