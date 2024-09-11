use core::fmt;

use crate::{
    era::Era,
    inquire_check,
    league::{check_name_hash, save_league, EditLeagueError, League},
    player::PlayerGender,
};

use rand::rngs::ThreadRng;
use rusqlite::Connection;
/// Used to generate a team based off a template
struct TeamTemplate {
    name: String,
    abrv: String,
}
/// Used to generate a League based off a template chosen by user.
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
        write!(
            f,
            "[ name:{}, era:{}, gender:{}]",
            self.name, self.era, self.gender
        )
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
/// Takes a league template, and creates a league based off the template that is added to the database.
pub fn new_league_from_template(
    conn: &mut Connection,
    thread: &mut ThreadRng,
    template: &LeagueTemplate,
) -> Result<(), rusqlite::Error> {
    // First, we query to see what league has the largest id.
    let mut max_id_stmt = conn.prepare("SELECT COUNT(leagues.league_id) FROM leagues")?;
    let max_id_iter = max_id_stmt.query_map([], |row| row.get(0))?;
    // We then put the max id in a vector
    let mut max_id_vec: Vec<i64> = Vec::new();
    for value in max_id_iter {
        let sql_result = value?;
        max_id_vec.push(sql_result);
    }
    // To determine the league name, we check if there are any id's.
    let league_name = match max_id_vec.is_empty() {
        false => {
            // If there already leagues in the database, first we retrieve the id number.
            let mut id_num = max_id_vec[0];
            let result;
            // We then query for a vector of league names
            let name_hash = check_name_hash(conn)?;
            loop {
                let potential_name = format!("{}_{}", template.name, id_num + 1);
                match name_hash.get(&potential_name).is_none() {
                    //If there is already a league saved with the file name, we add 1 to id_num
                    false => id_num += 1,
                    //Otherwise, we
                    true => {
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
    // And we create a new entry in the sql database.
    let _league_entry = conn.execute(
        "INSERT INTO leagues(league_name,era,gender) VALUES(?1, ?2, ?3)",
        [&league_name, &era_json, &gender_json],
    )?;

    // Via last_insert_rowid, we get the SQl id for the new league, as the teams we generate will need it.
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
        if let Err(message) = team_add {
            panic!("{:?}", message)
        };
    }
    save_league(&new_league);
    Ok(())
}

pub fn load_new_template(
    conn: &mut Connection,
    thread: &mut ThreadRng,
) -> Result<(), EditLeagueError> {
    let options = load_league_templates();

    let template_choice = inquire::Select::new("Please choose a league_template", options).prompt();

    match template_choice {
        Ok(template) => match new_league_from_template(conn, thread, &template) {
            Ok(()) => Ok(()),
            Err(message) => Err(EditLeagueError::DatabaseError(message)),
        },
        Err(message) => inquire_check(message),
    }
}
