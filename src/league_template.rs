use core::fmt;
use std::{cmp::max, thread::Thread};

use crate::{
    era::Era,
    inquire_check,
    league::{check_name_vec, save_league, AddTeamError, League},
    player::PlayerGender,
    team::{self, Team},
};
use chrono;
use name_maker::Gender;
use rand::rngs::ThreadRng;
use rusqlite::Connection;

struct TeamTemplate {
    name: String,
    abrv: String,
}

pub struct LeagueTemplate {
    name: String,
    era: Era,
    gender: PlayerGender,
    series_per_matchup: i32,
    game_per_series: i32,
    teams_templates: Vec<TeamTemplate>,
}

impl fmt::Display for LeagueTemplate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
pub fn load_league_templates() -> Vec<LeagueTemplate> {
    vec![LeagueTemplate {
        name: "PCL".to_string(),
        era: Era::Modern,
        gender: PlayerGender::Male,
        series_per_matchup: 10,
        game_per_series: 3,
        teams_templates: vec![
            TeamTemplate {
                name: "Los Angeles Angels".to_string(),
                abrv: "LA".to_string(),
            },
            TeamTemplate {
                name: "Hollywood Stars".to_string(),
                abrv: "HWYD".to_string(),
            },
            TeamTemplate {
                name: "San Diego Mission Bells".to_string(),
                abrv: "SD".to_string(),
            },
            TeamTemplate {
                name: "Sacramento Solons".to_string(),
                abrv: "SAC".to_string(),
            },
            TeamTemplate {
                name: "Oakland Oaks".to_string(),
                abrv: "OAK".to_string(),
            },
            TeamTemplate {
                name: "San Francisco Seals".to_string(),
                abrv: "SF".to_string(),
            },
            TeamTemplate {
                name: "Portland Beavers".to_string(),
                abrv: "POR".to_string(),
            },
            TeamTemplate {
                name: "Seattle Rainiers".to_string(),
                abrv: "SEA".to_string(),
            },
        ],
    }]
}

fn new_league_from_template(
    conn: &mut Connection,
    thread: &mut ThreadRng,
    template: &LeagueTemplate,
) -> Result<(), rusqlite::Error> {
    // let date_string = chrono::offset::Local::now().to_string();
    //let league_name = format!("{}_{}",template.name,date_string);
    
    // First, we query to see what league has the largest id.
    let mut max_id_stmt = conn.prepare("SELECT MAX(leagues.league_id) FROM leagues")?;
    let max_id_iter = max_id_stmt.query_map([], |row| Ok(row.get(0)?))?;
    // We then put the max id in a vector
    let mut max_id_vec: Vec<i64> = Vec::new();
    for value in max_id_iter {
        let sql_result = value?;
        max_id_vec.push(sql_result);
    }
    // To determine the league name, we check ifg htere are any id's
    let league_name = match max_id_vec.is_empty() {
        false => {
            // If there alreeady leagues i nteh database, first we retireve the id number.
            let mut id_num = max_id_vec[0];
            let result;
            // We then query for a vector of leauge names
            let name_vec = check_name_vec(conn)?;
            loop {
                let potential_name = format!("{}_{}", template.name, id_num + 1);
                match name_vec.contains(&potential_name) {
                    true => id_num += 1,
                    false => {
                        result = potential_name;
                        break;
                    }
                }
            }
            result
        }
        true => format!("{}_1", template.name),
    };
    println!("League_name = {}", league_name);
    drop(max_id_stmt);
    let era_json = serde_json::to_string(&template.era).unwrap();

    let gender_json = serde_json::to_string(&template.gender).unwrap();
    // And we create a new entry in the sql databse.
    let league_entry = conn.execute(
        "INSERT INTO leagues(league_name,era,gender) VALUES(?1, ?2, ?3)",
        [&league_name, &era_json, &gender_json],
    )?;

    // Via last_inster_rowid, we get the SQl id for the new league, as the teams we generate will need it.
    let league_id = conn.last_insert_rowid();

    let mut new_league = League::new(&league_name, template.gender, template.era, league_id);
    println!("{} created", &league_name);

    for team_template in template.teams_templates.iter() {
        let team_add = new_league.new_team(
            &team_template.abrv,
            &team_template.name,
            thread,
            league_id,
            conn,
        );
        match team_add {
            Err(message) => panic!("{:?}", message),
            _ => {}
        }
    }
    save_league(&new_league, conn, thread).unwrap();
    Ok(())
}

pub fn load_new_template(
    conn: &mut Connection,
    thread: &mut ThreadRng,
) -> Result<(), rusqlite::Error> {
    let options = load_league_templates();

    let template_choice = inquire::Select::new("Please choose a league_template", options).prompt();

    match template_choice {
        Ok(template) => new_league_from_template(conn, thread, &template),
        Err(message) => inquire_check(message),
    }
}