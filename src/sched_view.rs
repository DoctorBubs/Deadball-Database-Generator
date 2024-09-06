use core::fmt;

use inquire::{InquireError, Select};
use rusqlite::Connection;

use crate::{era, inquire_check, league::League};

fn get_season_vec(league: &League, conn: &Connection) -> Result<Vec<i64>, rusqlite::Error> {
    let mut seasons_stmt =
        conn.prepare("SELECT seasons.season_id FROM seasons WHERE seasons.league_id = ?1 ")?;
    let season_iter = seasons_stmt.query_map([league.league_id], |row| row.get(0))?;

    let mut result_vec = Vec::new();
    for num in season_iter {
        result_vec.push(num?)
    }

    Ok(result_vec)
}

fn get_round_vec(conn: &Connection, season_id: i64) -> Result<Vec<i64>, rusqlite::Error> {
    let mut rounds_stmt =
        conn.prepare("SELECT rounds.round_id FROM rounds WHERE rounds.season_id = ?1")?;
    let round_iter = rounds_stmt.query_map([season_id], |row| row.get(0))?;

    let mut result_vec = Vec::new();

    for num in round_iter {
        result_vec.push(num?)
    }

    Ok(result_vec)
}
#[derive(Debug)]
pub struct SeriesWrapper {
    series_id: i64,
    home_team_name: String,
    home_team_id: i64,
    away_team_name: String,
    away_team_id: i64,
}

impl fmt::Display for SeriesWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {}", self.away_team_name, self.home_team_name)
    }
}

fn get_series_vec(conn: &Connection, round_id: i64) -> Result<Vec<SeriesWrapper>, rusqlite::Error> {
    let mut series_stmt = conn.prepare(
        "
        WITH home_teams AS(
            SELECT 
                teams.team_name AS team_name,
                teams.team_id AS team_id ,
                series.series_id AS series_id
            FROM
                series
            INNER JOIN 
                teams on series.home_team_id = teams.team_id
            WHERE
                series.round_id = ?1        
        )

        SELECT
            series.series_id,
            home_teams.team_name,
            home_teams.team_id,
            teams.team_name,
            teams.team_id
        FROM 
            series
        INNER JOIN 
            home_teams ON home_teams.series_id = series.series_id
        INNER JOIN
            teams on series.away_team_id = teams.team_id
        WHERE
            series.round_id = ?1

    
    
    ",
    )?;

    let series_iter = series_stmt.query_map([round_id], |row| {
        Ok(SeriesWrapper {
            series_id: row.get(0)?,
            home_team_name: row.get(1)?,
            home_team_id: row.get(2)?,
            away_team_name: row.get(3)?,
            away_team_id: row.get(4)?,
        })
    })?;

    let mut result_vec = Vec::new();

    for s_wrapper in series_iter {
        result_vec.push(s_wrapper?)
    }

    Ok(result_vec)
}

fn get_game_vec(conn: &Connection, wrapper: &SeriesWrapper) -> Result<Vec<i64>, rusqlite::Error> {
    let mut games_stmt =
        conn.prepare("SELECT games.game_id FROM games WHERE games.series_id = ?1")?;
    let games_iter = games_stmt.query_map([wrapper.series_id], |row| row.get(0))?;
    let mut result_vec = Vec::new();
    for num in games_iter {
        result_vec.push(num?)
    }

    Ok(result_vec)
}

struct RoundChoiceListing {
    index: usize,
    value: i64,
}

impl fmt::Display for RoundChoiceListing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.index + 1)
    }
}



pub fn view_schedule(league: &League, conn: &Connection) -> Result<(), rusqlite::Error> {
    let sched_vec = get_season_vec(league, conn)?;
    if sched_vec.is_empty() {
        println!("No schedule generated");
        return Ok(());
    }
    let season_choice = Select::new("Choose a season to view.", sched_vec)
        .prompt();
    let season_id;
    match season_choice{
        Ok(num) => season_id = num,
        Err(message) => return inquire_check(message)
    }
    //println!("{:?}",sched_vec?);
    let round_vec = get_round_vec(conn, season_id)?
        .into_iter()
        .enumerate()
        .map(|(index, value)| RoundChoiceListing { index, value })
        .collect();

    //println!("{:?}",round_vec);
    let round_choice = Select::new("Choose a round to view.", round_vec)
        .prompt();

    let round_id = match round_choice{
        Ok(listing) => listing.value,
        Err(message) => return inquire_check(message)
    };
    let series_vec = get_series_vec(conn, round_id)?;
    //println!("{:?}",series_vec);
    let series_choice = Select::new("Choose a series from the round", series_vec)
        .prompt();
    let choosed_wrapper = match series_choice{
        Ok(wrapper) => wrapper,
        Err(message) => return inquire_check(message)
    };

    let game_vec = get_game_vec(conn, &choosed_wrapper)?;
    println!("{:?}", game_vec);
    Ok(())
}
