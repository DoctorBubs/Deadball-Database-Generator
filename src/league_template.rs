use core::panicking::panic_nounwind;
use std::thread::Thread;

use name_maker::Gender;
use rand::rngs::ThreadRng;
use rusqlite::Connection;
use chrono;
use crate::{era::Era, league::{AddTeamError, League}, player::PlayerGender, team::{self, Team}};

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


fn new_league_from_template(conn: &mut Connection, thread: &mut ThreadRng, template: &LeagueTemplate) -> Result<(),rusqlite::Error>{
    let date_string = chrono::offset::Local::now().to_string();
    let league_name = format!("{}_{}",template.name,date_string);
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

    for team_template in template.teams_templates.iter(){
        let team_add = new_league.new_team(&team_template.abrv,&team_template.name,thread,league_id, conn);
        match team_add {
            Err(message) => panic!("{:?}",message),
            _ => {}
        }
    }

    Ok(())
}