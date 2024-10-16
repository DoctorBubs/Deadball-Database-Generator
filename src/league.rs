use core::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;

use std::path::Path;

use crate::b_traits::BTraits;
use crate::edit_league_error::handle_sql_error;
use crate::edit_league_error::EditLeagueError;
use crate::era::select_era;
use crate::inquire_check;
use crate::main_menu::EditLeagueInput;
use crate::main_menu::LoadLeagueInput;
use crate::main_menu::RankingsChoice;
use crate::note::Notable;
use crate::note::Note;
use crate::pd::PD;
use crate::pennantgen::generate_pennant_standings;
use crate::pennantgen::PennantStanding;
use crate::player::select_gender;
use inquire::validator::MinLengthValidator;
use inquire::Confirm;
use inquire::CustomType;
use inquire::InquireError;
use inquire::Select;
use inquire::Text;
use itertools::Itertools;
use rand::prelude::*;
use rand::Error;
use rusqlite::Result;
use serde::de::value;

use crate::player::Player;

use crate::traits::Contact;
use crate::traits::Defense;
use crate::traits::PlayerTrait;
use crate::traits::Power;
use crate::traits::Speed;
use crate::traits::Toughness;
use rusqlite::Connection;
//use crate::sched_view::view_schedule;

use crate::sched_view::view_schedule;
use crate::schedule::save_schedule_sql;
use crate::team::add_new_team;
use crate::team::load_team;
use crate::vec_to_hash;
use crate::Deserialize;
use crate::Era;

use crate::PlayerGender;

use crate::Serialize;
use crate::Team;
use crate::ThreadRng;

//use crate::schedule::*;
use std::collections::HashMap;

/// Used when sorting players via a leaderboard.
#[derive(Debug, Copy, Clone)]
enum _PlayerSortBy<T: PlayerTrait> {
    Bt,
    Obt,
    Obt_Mod,
    Age,
    B_Trait(T),
}
// Used when filtering batters.
#[derive(Debug, Clone, Copy)]
pub enum BatterPosType {
    Catchers,
    Infielders,
    Outfielders,
    All,
}

impl BatterPosType {
    // Returns a string that is formatted like a touple of positions that match the batter type.
    fn get_tup_string(&self) -> &str {
        match self {
            Self::Catchers => "('C')",
            Self::Infielders => "('1B','2B','3B','SS','INF')",
            Self::Outfielders => "('LF','CF','RF','OF')",
            Self::All => "('C','1B','2B','3B','SS','INF','LF','CF','RF','OF')",
        }
    }
    // Returns a string that can be used in a SQL query to filter players that fit the type.
    pub fn get_sql_text(&self) -> String {
        match self {
            Self::All => "\n\t AND players.PD IS NULL\n".to_string(),
            _ => format!("\n\t AND players.pos IN {}\n", self.get_tup_string()),
        }
    }
}

impl fmt::Display for BatterPosType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Catchers => "Catchers",
            Self::Infielders => "Infielders",
            Self::Outfielders => "Outfielders",
            Self::All => "All",
        };
        write!(f, "{}", text)
    }
}

/// Used when adding players to leaderboards, this struct contains a player and the name of it's teams.
#[derive(Debug)]
struct PlayerRankWrapper {
    team_name: String,
    player: Player,
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct TeamPennantWrapper<'a> {
    id: i64,
    team_score: i32,
    weight: i32,
    name: &'a String,
}

// A league containts a vector of teams, but also keeps track of the gender and era enums. A league can create team, an also ensure that
// each team follows the gender and era rules.
#[derive(Debug, Serialize, Deserialize)]
pub struct League {
    pub name: String,
    pub teams: Vec<Team>,
    pub gender: PlayerGender,
    pub era: Era,
    pub league_id: i64, //bench_quality:BatterQuality,
    pub note: Note,
}

impl Notable for League {
    fn get_note(&self) -> &Note {
        &self.note
    }

    fn get_note_input_string(&self) -> String {
        format!("Please enter the note for {}", self.name)
    }
}

pub struct LeagueWrapper {
    pub league_id: i64,
    pub league: League,
}

// We implement display for LeagueWrapper, as we will need to see print a list of all leagues to the console when a user wants to open an existing leaghue
impl fmt::Display for LeagueWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}", self.league_id, self.league.name)
    }
}

impl fmt::Display for EditLeagueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Self::DatabaseError(message) => message.to_string(),
            Self::SerdeError(message) => message.to_string(),
            Self::Inquire(message) => message.to_string(),
            _ => "Invalid Team and/or Abbreviation".to_string(),
        };
        write!(f, "{}", text)
    }
}

struct StandingWrapper {
    name: String,
    team_score: i32,
    wins: i32,
    losses: i32,
    games_behind: i32,
}
impl League {
    pub fn new(name: &String, gender: PlayerGender, era: Era, league_id: i64) -> League {
        League {
            name: name.to_string(),
            teams: Vec::new(),
            gender,
            era,
            league_id,
            note: None,
        }
    }

    /*pub fn get_new_schedule(&self, series_length: i32,series_per_matchup:i32) ->  Result<Vec<Round>, ScheduleGenError>{
        new_schedule(&self.teams,  series_length, series_per_matchup)

    }*/
    pub fn display_standings(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = conn.prepare(
            "
         WITH batter_scores AS(
            SELECT 
                teams.team_id AS team_id,
                SUM(players.bt) AS bt_score
            FROM
                players
            INNER JOIN 
                teams ON teams.team_id = players.team_id
            WHERE
                teams.league_id = ?1
                AND players.PD IS NULL
            GROUP BY teams.team_id
        ),
        
        pitcher_scores AS(
            SELECT 
                teams.team_id as team_id,
                SUM(players.pd_int) * 7 as pd_score
                
            FROM
                players
            INNER JOIN 
                teams ON teams.team_id = players.team_id
            WHERE
                teams.league_id = ?1
                AND
                players.PD IS NOT NULL
            GROUP BY teams.team_id
        
        ),

        team_scores AS(
            SELECT
                teams.team_id AS team_id,
                (batter_scores.bt_score + pitcher_scores.pd_score) / 10 AS team_score
                FROM 
                    teams
                INNER JOIN
                    batter_scores ON batter_scores.team_id = teams.team_id
                INNER JOIN pitcher_scores ON pitcher_scores.team_id = teams.team_id
               
        ),
        
        first_place AS(
            SELECT 
                leagues.league_id AS league_id, 
                MAX(teams.wins) AS wins
            FROM 
                leagues
            INNER JOIN 
                teams ON teams.league_id = leagues.league_id
            WHERE 
                leagues.league_id = ?1
            GROUP BY 
                leagues.league_id
        )
        
        SELECT 
            teams.team_name, 
            team_scores.team_score,
            teams.wins, 
            teams.losses,
            (first_place.wins - teams.wins) AS games_behind 
        FROM 
            teams
        INNER JOIN 
            first_place ON first_place.league_id = teams.league_id
        INNER JOIN
            team_scores ON teams.team_id = team_scores.team_id
        WHERE 
            teams.league_id = ?1
        ORDER BY 
            games_behind ASC, team_scores.team_score DESC;
        ",
        )?;

        let rows = stmt
            .query_map([self.league_id], |row| {
                Ok({
                    StandingWrapper {
                        name: row.get(0)?,
                        team_score: row.get(1)?,
                        wins: row.get(2)?,
                        losses: row.get(3)?,
                        games_behind: row.get(4)?,
                    }
                })
            })?
            .filter_map(|x| x.ok());

        for standing in rows {
            println!(
                "{} {} {} {} {}",
                standing.name,
                standing.team_score,
                standing.wins,
                standing.losses,
                standing.games_behind
            )
        }

        Ok(())
    }

    /// Displays a leaderboard of the top 10 batters in the league.
    /// The query is structured so that batters with high on base targets, power, and the platoon
    /// advntage will be higher in the leaderboard
    pub fn display_top_hitters(
        &self,
        conn: &mut Connection,
        filter_choice: Option<BatterPosType>,
    ) -> Result<(), rusqlite::Error> {
        // We get a sql filter from the filter choice, we use BatterPosType::All if the option is none.
        let sql_filter = filter_choice.unwrap_or(BatterPosType::All).get_sql_text();

        let sql_input = format!(
            "
            SELECT
                teams.team_name,
                players.player_name,
                players.age,
                players.pos,
                players.hand,
                players.bt,
                players.obt_mod,
                players.obt,
                players.contact,
                players.defense,
                players.power,
                players.speed,
                players.toughness,
                -- Next we convert the players batter traits to numbers for use in sorting.
                CASE
                    WHEN players.power LIKE '%P++%' THEN 4
                    WHEN players.power LIKE '%P+%' THEN 2
                    WHEN players.power LIKE '%P-%' THEN -2
                    WHEN players.power LIKE '%P--%' THEN -4
                    ELSE 0
                END AS power_number,
                CASE
                    WHEN players.contact LIKE '%C+%' THEN 1
                    WHEN players.contact LIKE '%C-%' THEN 0
                    ELSE 0
                END AS contact_number,
                CASE 
                    WHEN players.speed LIKE '%S++%' THEN 2
                    WHEN players.speed  LIKE '%S+%' THEN 1
                    WHEN players.speed LIKE '%S-' THEN -1
                    ELSE 0
                END AS speed_number,
                CASE
                    WHEN players.defense LIKE '%D+%' Then 1
                    WHEN players.defense LIKE '%D-%' THEN -1
                    ELSE 0
                END AS defense_number,
                CASE
                    WHEN players.toughness LIKE '%T+%' THEN 1
                    ELSE 0
                END AS toughness_number,
                -- We also convert Hand to a number
                CASE
                    WHEN players.hand LIKE '%S%' THEN 5
                    WHEN players.hand LIKE '%L%' THEN 2
                    ELSE 0
                END AS hand_number
            FROM teams
            INNER JOIN 
                players
            ON
                players.team_id = teams.team_id
            WHERE
                -- We have filter text selected earlier in the function.
                teams.league_id = ?1{}
            ORDER BY
                /*To Sort, we add the players obt + power_number + hand number
                 The idea that is that players with high obts, powers and platoon
                advantage will be high up in the query
                If there is a tie, we sort by the individual field used.
                We are using the assumption that OBT is more important then power, ann
                 power is more important than the platoon advantage. */
                players.obt + power_number + hand_number DESC,
                hand_number DESC,
                players.obt DESC, 
                power_number DESC,
                /*If there is still a tie, we sort by the other rows in the query
                 First we sort by how good at contact the player is. */
                contact_number DESC,
                players.bt DESC, 
                --Then we check speed and defense, with defense being more important.
                defense_number DESC,
                speed_number DESC,
                -- Finally, we sort by if the players is resistant to injury, and then finally age.
                toughness_number DESC,
                players.age ASC
                -- And we limit the players in the query to 10.
          LIMIT 10;
        ",
            sql_filter
        );
        // And we prepare the statement.
        let mut stmt = conn.prepare(&sql_input)?;
        let player_iter = stmt
            .query_map([self.league_id], |row| {
                Ok(
                    // We Save the team name in the PlayerRankWrapper
                    PlayerRankWrapper {
                        team_name: row.get(0)?,
                        player: Player {
                            // And we fill the fields in the player struct with values from the rows.
                            name: row.get(1)?,
                            age: row.get(2)?,
                            pos: row.get(3)?,
                            hand: serde_json::from_value(row.get(4)?).unwrap(),
                            bt: row.get(5)?,
                            obt_mod: row.get(6)?,
                            obt: row.get(7)?,
                            b_traits: BTraits {
                                contact: serde_json::from_value(row.get(8)?).unwrap_or(Contact::C0),
                                defense: serde_json::from_value(row.get(9)?).unwrap_or(Defense::D0),
                                power: serde_json::from_value(row.get(10)?).unwrap_or(Power::P0),
                                speed: serde_json::from_value(row.get(11)?).unwrap_or(Speed::S0),
                                toughness: serde_json::from_value(row.get(12)?)
                                    .unwrap_or(Toughness::T0),
                            },
                            // We use a default player to fill in the fields we did not query from.
                            ..Player::default()
                        },
                    },
                )
            })?
            .filter_map(|x| x.ok());
        // We print a line of headers for each category to display
        println!("Team_name,Player_Name,Pos,Age,Hand,Bt,OBT_Mod,OBT,Traits,Tier");
        // We then loop over the player iter to print what we need.
        for prw in player_iter {
            // We remove the PlayerRankWrapper from the ok, and deconstruct it

            let PlayerRankWrapper { team_name, player } = prw;
            let tier = player.get_tier();
            /* Since we have already implemented the Display trait for Player, and
            the string generated matches what we want, we cna just print the player directly */
            println!("{},{},{}", team_name, player, tier)
        }
        // We then return Ok as we have gotten this far without an error from the database.
        Ok(())
    }
    ///Prints a ranking of the top 10 pitchers in the league.
    /// Players with better pd and traits go higher, preference is also given to leftys.
    pub fn display_top_pitchers(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = conn.prepare(
            "
            SELECT 
                teams.team_name,
                players.player_name,
                players.age,
                players.hand,
                players.PD,
                players.pitcher_trait,
                players.pos,
                players.pd_int,
                CASE
                    WHEN players.pitcher_trait IS NULL THEN 0
                    WHEN players.pitcher_trait LIKE '%CN-%' THEN -1
                    ELSE 1
                END AS pitcher_trait_num,
                CASE
                    WHEN players.hand LIKE '%L%' THEN 1
                    ELSE 0
                END AS hand_num
            FROM
                teams
            INNER JOIN 
                players
            ON
                teams.team_id = players.team_id
            WHERE
                teams.league_id = ?1
                AND players.PD IS NOT NULL
            
            ORDER BY players.pd_int DESC, pitcher_trait_num DESC, hand_num DESC, players.age DESC
        LIMIT 10;
        ",
        )?;

        let player_iter = stmt
            .query_map([self.league_id], |row| {
                Ok(PlayerRankWrapper {
                    team_name: row.get(0)?,
                    player: Player {
                        name: row.get(1)?,
                        age: row.get(2)?,
                        hand: serde_json::from_value(row.get(3)?).unwrap(),
                        pd: serde_json::from_value(row.get(4)?).unwrap(),
                        pitcher_trait: serde_json::from_value(row.get(5)?).unwrap(),
                        pos: row.get(6)?,
                        // We fill the rest of the player fields with default datat.
                        ..Player::default()
                    },
                })
            })?
            .filter_map(|x| x.ok());
        println!("Team,name,pos,hand,age,PD,Trait,Tier");
        for prw in player_iter {
            let PlayerRankWrapper { team_name, player } = prw;
            let tier = player.get_tier();
            let Player {
                name,
                pos,
                hand,
                age,
                pd,
                pitcher_trait,
                ..
            } = player;

            // Since not all pitchers will have PitcherTraits, we match to create a string value.
            let p_trait_value = pitcher_trait.map(|value| format!("{},", value));

            // We print the fields. Since a player will always have a PD , we are OK to unwrap it.
            println!(
                "{},{},{},{},{},{},{}{}",
                team_name,
                name,
                pos,
                hand,
                age,
                pd.unwrap(),
                p_trait_value.unwrap_or("".to_string()),
                tier
            )
        }
        Ok(())
    }
    /// Prompts the user to what player rankings they would like to see.
    pub fn display_ranking(&self, conn: &mut Connection) -> Result<(), EditLeagueError> {
        // We give the user the chance to pick if we are going to rank batters or pitchers.
        self.display_league_averages(conn).unwrap();
        let options = vec![RankingsChoice::Batters, RankingsChoice::Pitchers];
        let answer = Select::new("Which player rankings would you like to see?", options).prompt();
        match answer {
            // We return the inquire error if their is one
            Err(message) => Err(EditLeagueError::Inquire(message)),
            Ok(value) => {
                //If they choose to view batters, we ask if they would like to filter batters byy position.

                let filter_choice = {
                    if let RankingsChoice::Batters = value {
                        let filter_answer = Confirm::new(
                            "Would you like to filter the batters by their position type?",
                        )
                        .prompt();
                        match filter_answer {
                            Ok(false) => None,
                            Ok(true) => {
                                let options = vec![
                                    BatterPosType::Catchers,
                                    BatterPosType::Infielders,
                                    BatterPosType::Outfielders,
                                ];
                                let pos = Select::new(
                                    "Please pick the position you would like to view.",
                                    options,
                                )
                                .prompt();
                                match pos {
                                    Ok(value) => Some(value),
                                    Err(message) => return Err(EditLeagueError::Inquire(message)),
                                }
                            }
                            Err(message) => return Err(EditLeagueError::Inquire(message)),
                        }
                    } else {
                        //If the user wants to only view pitchers, the filter choice is none.
                        None
                    }
                };
                let query_result = match value {
                    RankingsChoice::Batters => self.display_top_hitters(conn, filter_choice),
                    RankingsChoice::Pitchers => self.display_top_pitchers(conn),
                };
                // Finally, we check if their was an error running the query.
                match query_result {
                    Ok(()) => Ok(()),
                    Err(message) => Err(EditLeagueError::DatabaseError(message)),
                }
            }
        }
    }

    /// Query's the database to create the average player for the league.
    pub fn get_average_player(&self, conn: &mut Connection) -> Result<Player, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "
            WITH pitching_data AS(
                SELECT leagues.league_id AS league_id,
                ROUND(AVG(players.pd_int)) as avg_pd
            FROM leagues
            INNER JOIN
                teams ON teams.league_id = leagues.league_id
            INNER JOIN 
                players ON players.team_id = teams.team_id
            WHERE
                players.PD IS NOT NULL
            GROUP BY
                leagues.league_id
            ),
            gen_data AS (
                SELECT
                    leagues.league_id AS league_id,
                    ROUND(AVG(players.age)) as avg_age
                FROM leagues
                INNER JOIN
                    teams ON teams.league_id = leagues.league_id
                INNER JOIN
                    players ON players.team_id = teams.team_id
                GROUP BY leagues.league_id
            )
            SELECT
                CAST(ROUND(AVG(players.bt)) AS INT)  AS average_bt,
                CAST(ROUND(AVG(players.obt)) AS INT) AS average_obt,
                CAST(pitching_data.avg_pd AS INT) AS average_pd,
                CAST(gen_data.avg_age AS INT) AS average_age
            FROM leagues
            INNER JOIN
               teams ON teams.league_id = leagues.league_id
            INNER JOIN
                players ON players.team_id = teams.team_id
            INNER JOIN
                pitching_data ON pitching_data.league_id = leagues.league_id
            INNER JOIN
                gen_data ON gen_data.league_id = leagues.league_id
            WHERE
                players.PD IS NULL
            AND leagues.league_id = ?1
            GROUP BY
                leagues.league_id;

        
        ",
        )?;
        let mut player_iter = stmt
            .query_map([self.league_id], |row| {
                Ok(Player {
                    name: self.name.clone(),
                    bt: row.get(0)?,
                    obt: row.get(1)?,
                    pd: Some(PD::new_custom_pd(row.get(2)?)),
                    age: row.get(3)?,
                    ..Player::default()
                })
            })?
            .filter_map(|x| x.ok());

        let top_player = player_iter.next();
        let result = match top_player {
            Some(avg_player) => avg_player,
            _ => {
                println!("Unable to calculate, please make sure there are players in this league in the database");
                return Err(rusqlite::Error::InvalidQuery);
            }
        };
        Ok(result)
    }

    pub fn display_league_averages(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        let decimal = 100.0;
        // We create the average player for the league
        let player = self.get_average_player(conn)?;
        let Player { bt, obt, pd, .. } = player;
        let pd_result = pd.unwrap();
        let (expected_batting, expected_obp) = player.expected_batting_obp(pd_result);
        let expected_batting_decimal = expected_batting / decimal;
        let expected_obp_decimal = expected_obp / decimal;
        println!("Average bt: {}", bt);
        println!("Average obt: {}", obt);
        println!("Average pdL {}", pd_result);
        println!("Expected batting average: {:.3}", expected_batting_decimal);
        println!("Expected on base percentage: {:.3}", expected_obp_decimal);

        Ok(())
    }
    /*  Take a new abbreviation and name for a team, a thread for random number, a league id and connection to the the database.
        If there are no teams in the league that have the same name or abbreviation,  we attempt to add the team to the league. If it succeeds, an Ok is returned
    */
    pub fn new_team(
        &mut self,
        new_abrv: &String,
        new_name: &String,
        thread: &mut ThreadRng,
        league_id: i64,
        conn: &mut Connection,
    ) -> Result<(), EditLeagueError> {
        for team in &self.teams {
            if new_abrv == &team.abrv {
                return Err(EditLeagueError::AbrvTaken);
            } else if new_name == &team.name {
                return Err(EditLeagueError::NameTaken);
            };
        }
        // We create a new team
        let mut new_team = Team::new(new_abrv, new_name, self.gender, self.era, thread);
        // We get the team score for hte new team.
        // We enter the team into the database.
        let team_enter_result = conn.execute(
            "INSERT INTO teams(team_name,abrv, league_id) VALUES(?1,?2, ?3)",
            [&new_name, &new_abrv, &league_id.to_string()],
        );
        // We save the team ID, so that we we generate the new players they can be saved in the databse with the league id as the foreign key.
        let new_team_id = conn.last_insert_rowid();
        //new_team.team_id = team_id as i32;
        match team_enter_result {
            Ok(_) => (),
            Err(message) => return Err(EditLeagueError::DatabaseError(message)),
        };
        //If all has gone well, we save the players that have been generated into the database

        new_team.save_players_sql(conn, new_team_id)?;
        // And we insert the team struct into the league's team vector.
        self.teams.push(new_team);
        Ok(())
    }

    pub fn new_team_hash(&self) -> HashMap<i64, &Team> {
        let mut result = HashMap::new();
        for team in self.teams.iter() {
            result.insert(team.team_id, team);
        }
        result
    }
    /// Creates a randomly generated pennant race that is saved to the database and is also exported as a file.
    pub fn create_pennant_race(
        &self,
        thread: &mut ThreadRng,
        conn: &mut Connection,
        games_played: i32,
    ) -> Result<(), EditLeagueError> {
        // First, we check to see if there are already any pennant races associated with the leauge.

        let mut existing_stmt = conn.prepare(
            "
            SELECT 
                COUNT(pennants.pennant_id)
            FROM 
                pennants
            WHERE
                pennants.league_id = ?1
        
        
        ",
        );
        let mut pennant_stmt = match existing_stmt {
            Ok(value) => value,
            Err(message) => return Err(EditLeagueError::DatabaseError(message)),
        };
        let pennants_iter = pennant_stmt.query_map([self.league_id], |row| {
            let num: i32 = row.get(0).unwrap();
            Ok(num)
        });

        let count: i32 = handle_sql_error(pennants_iter)?
            .filter_map(|x| x.ok())
            .next()
            .unwrap_or(0);
        // And use it to determine the file name we will be writing to.
        let file_name = format!("{}_Pennant_{}.txt", self.name, count);
        // We map all teams to a vector of team pennant wrappers.
        let mut pennant_wrappers: Vec<TeamPennantWrapper> = self
            .teams
            .iter()
            .map(|x| TeamPennantWrapper {
                id: x.team_id,
                team_score: x.team_score,
                weight: (100 - x.team_score).abs(),
                name: &x.name,
            })
            .collect();
        // We create a target len that is used to calculate how many stands are needed.
        let target_len = self.teams.len();
        // We create a team ranks vector.
        let mut team_ranks = Vec::new();
        // This is a litte complicated, but basically we
        while team_ranks.len() != target_len {
            // First we create a temporary vector.
            let mut temp_vec = Vec::new();
            // We take an iter of the penant wrappers that also has an enumerate, so we know where in the vector is located.
            let modified_ranks = pennant_wrappers.iter().enumerate();
            // We take the index and value for each listing in the vecotr, and save it in a tuple, which is saved in the temp_vec/
            for (i, rank) in modified_ranks {
                let new_tup = (i, rank);
                temp_vec.push(new_tup)
            }
            // We pick a random value from temp vec via a weighted selection.
            let (j, sample) = temp_vec.choose_weighted(thread, |x| x.1.weight).unwrap();
            // We push the sample to team ranks.
            team_ranks.push(**sample);
            // And using the index, we remove the value from pennant wrappers.
            pennant_wrappers.remove(*j);
        }
        // We retrieve the pennant standing, which is a vector of vectors of i32.
        // The first value represents a teams wins, and the second represents the teams losses.
        let input_standings = generate_pennant_standings(games_played, thread, target_len as i32)?;

        let final_standings = team_ranks.iter().zip(input_standings);
        handle_sql_error(conn.execute(
            "INSERT INTO pennants(league_id) VALUES(?1)",
            [self.league_id],
        ))?;
        let pennant_id = conn.last_insert_rowid();

        let mut file_string = "Name,Wins,Losses, Games Behind,Team Score".to_string();
        let mut top_wins = None;
        for (i, standing) in final_standings.rev().enumerate() {
            let (pennant_wrapper, win_loss) = standing;
            let TeamPennantWrapper {
                id,
                name,
                team_score,
                ..
            } = pennant_wrapper;
            let PennantStanding { wins, losses } = win_loss;
            if i == 0 {
                top_wins = Some(wins)
            };
            let games_behind = top_wins.unwrap_or(wins) - wins;
            let standing_line = format!(
                "\n{},{},{},{},{}",
                name, wins, losses, games_behind, team_score
            );
            file_string.push_str(&standing_line);
            handle_sql_error(conn.execute(
                "
                INSERT INTO pennants_standings(team_id,pennant_id,wins,losses) VALUES(?1,?2,?3,?4)
            
            
            ",
                [id, &pennant_id, &(wins as i64), &(losses as i64)],
            ))?;
        }
        println!("{}", file_string);
        fs::write(file_name, file_string).unwrap();
        Ok(())
    }
}

pub fn check_name_vec(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT league_name FROM leagues")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut names = Vec::new();
    for name_result in rows {
        names.push(name_result?);
    }

    Ok(names)
}
// Returns a hash
pub fn check_name_hash(conn: &Connection) -> Result<HashMap<String, bool>, rusqlite::Error> {
    let names_vec = check_name_vec(conn)?;
    let mut result = HashMap::new();
    for name in names_vec.into_iter() {
        result.insert(name, true);
    }

    Ok(result)
}

// Creates a new leagues, and saves the league in the database
pub fn create_new_league(
    thread: &mut ThreadRng,
    conn: &mut Connection,
) -> Result<(), EditLeagueError> {
    let _validator = MinLengthValidator::new(3);

    let league_name: String;

    let taken_names = match check_name_vec(conn) {
        Ok(data) => data,
        Err(message) => return Err(EditLeagueError::DatabaseError(message)),
    };
    // let mut taken_hash = HashMap::new();
    let taken_hash = vec_to_hash(&taken_names);
    //for name in taken_names.iter() {
    //taken_hash.insert(name, true);
    // }
    loop {
        if !taken_names.is_empty() {
            println!("The following league names have already been taken:");
            for name in &taken_names {
                println!("{}", name)
            }
        };

        let potential_name_choice = Text::new("Please enter a name for the new league").prompt();
        let potential_name = match potential_name_choice {
            Ok(input) => input,
            Err(message) => return inquire_check(message),
        };
        if taken_hash.get(&potential_name).is_none() {
            league_name = potential_name;
            break;
        } else {
            println!("Name already taken.")
        }
    }

    // We have the user select the era for the league.
    let era_choice = select_era();
    let era = match era_choice {
        Ok(input) => input,
        Err(message) => return inquire_check(message),
    };
    // As well as the gender of the players for the league.
    let gender_choice = select_gender();
    let gender = match gender_choice {
        Ok(input) => input,
        Err(message) => return inquire_check(message),
    };
    // We then create a league struct.

    // We then serialize the era and jender to json.
    let era_json = match serde_json::to_string(&era) {
        Ok(data) => data,
        Err(message) => return Err(EditLeagueError::SerdeError(message)),
    };

    let gender_json = match serde_json::to_string(&gender) {
        Ok(gender) => gender,
        Err(message) => return Err(EditLeagueError::SerdeError(message)),
    };
    // And we create a new entry in the sql databse.
    match conn.execute(
        "INSERT INTO leagues(league_name,era,gender) VALUES(?1, ?2, ?3)",
        [&league_name, &era_json, &gender_json],
    ) {
        Ok(_) => {}
        Err(message) => return Err(EditLeagueError::DatabaseError(message)),
    };

    // if league_entry.is_err() {
    // println!("Error creating a new league in the database.");
    // return Ok(());
    //};
    // Via last_inster_rowid, we get the SQl id for the new league, as the teams we generate will need it.
    let league_id = conn.last_insert_rowid();
    // We then create a leage struct in rust.
    let mut new_league = League::new(&league_name, gender, era, league_id);
    println!("{} created", &league_name);
    //And then prompt the user to create the first team for the league.
    match add_new_team(&mut new_league, thread, conn, league_id, true) {
        Ok(_) => Ok(()),
        Err(message) => Err(message),
    }
}

#[derive(Debug)]
//Teamwrapper contains a team id from the database, as well as the team struct that correspond ot the id.
pub struct TeamWrapper {
    pub team_id: i64,
    pub team: Team,
}

/// Loads teams from SQL database and adds to league struct.
pub fn load_teams_from_sql(
    league_id: i64,
    league: &mut League,
    conn: &mut Connection,
) -> Result<(), rusqlite::Error> {
    let era = league.era;
    // We query the database to select all teams in the database that belong to the league via the league_id car
    let mut stmt = conn.prepare(
        "SELECT team_id,abrv,team_name,wins,losses,team_note
        FROM teams 
        WHERE league_id = ?1",
    )?;

    let team_iter: Vec<Team> = stmt
        .query_map([league_id], |row| {
            // For each team that matchers, we create a new TeamWrapper that is wrapped in an Ok.
            Ok(
                // We use the remaining rows to deseirialize the team
                Team {
                    // We fill out the rest of the fields in the team struct from the database entry.
                    team_id: row.get(0)?,
                    abrv: row.get(1)?,
                    name: row.get(2)?,
                    wins: row.get(3)?,
                    losses: row.get(4)?,
                    // We create a vector for each player pool that a team has.
                    lineup: Vec::new(),
                    bench: Vec::new(),
                    starting_pitching: Vec::new(),
                    //Ancient Era teams do not have a bullpen, while Modern Era teams do.
                    bullpen: match era {
                        Era::Ancient => None,
                        Era::Modern => Some(Vec::new()),
                    },

                    team_score: 0,
                    note: serde_json::from_value(row.get(5)?).unwrap(),
                },
            )
        })?
        .filter_map(|x| x.ok())
        .collect();

    // We drop stmt so we can borrow conn later.
    drop(stmt);
    // We then loa
    for team in team_iter {
        // We load the team from the database in the form of a Rust struct.
        let loaded_team = load_team(conn, team)?;

        // And add the team to the league's teams vector.
        league.teams.push(loaded_team)
    }

    Ok(())
}

pub fn load_league(
    thread: &mut ThreadRng,
    conn: &mut Connection,
    wrapper: LeagueWrapper,
    edit_input: EditLeagueInput,
) -> Result<(), EditLeagueError> {
    // We destructure the LeagueWrapper
    let LeagueWrapper {
        league_id,
        mut league,
    } = wrapper;

    if let Err(message) = league.display_standings(conn) {
        println!("Error Displaying Standings");
        return Err(EditLeagueError::DatabaseError(message));
    }
    if let Err(message) = load_teams_from_sql(league_id, &mut league, conn) {
        println!("Error loading teams");
        return Err(EditLeagueError::DatabaseError(message));
    }

    // Now that we have loaded the existing league from the database, it is time to generate a new team or create a new schedule based off the input
    match edit_input {
        EditLeagueInput::CreateNewTeam => add_new_team(&mut league, thread, conn, league_id, true)?,
        EditLeagueInput::CreateSchedule => {
            match league.teams.len() % 2 == 0 {
                true => save_schedule_sql(conn, &league)?,
                false => {
                    println!("League must have an even number of teams");
                    save_league(&league);
                }
            };
        }
        EditLeagueInput::GeneratePennant => {
            if league.teams.len() <= 3 {
                return Err(EditLeagueError::PennantError(
                    "Error: A league must have 4 or more teams in order to generate a pennant."
                        .to_string(),
                ));
            };
            let games_played_input = CustomType::<i32>::new(
                "Please enter how many games have been played in the season.",
            )
            .with_error_message("Please enter a valid whole number.")
            .prompt();
            match games_played_input {
                Ok(games_played) => league.create_pennant_race(thread, conn, games_played)?,
                Err(err) => return inquire_check(err),
            }
        }
    };
    Ok(())
}

/*  The League Wrapper struct is used when the program checks to see what leagues are saved in the database.

 It contains the ID which the leagues is saved in the database, as well a deserialzied League struct from the database
*/
pub fn get_all_leagues_from_db(
    conn: &mut Connection,
) -> Result<Vec<LeagueWrapper>, rusqlite::Error> {
    // We query the database to get all the leagues that already exist.
    let mut stmt = conn.prepare("SELECT * from leagues").unwrap();
    // We wrap the rows into a LeagueWrapper that is part of a Rust Ok.
    let result = stmt
        .query_map([], |row| {
            Ok(LeagueWrapper {
                league_id: row.get(0)?,
                league: League {
                    name: row.get(1)?,
                    era: serde_json::from_value(row.get(2)?).unwrap(),
                    gender: serde_json::from_value(row.get(3)?).unwrap(),
                    note: serde_json::from_value(row.get(4)?).unwrap(),
                    league_id: row.get(0)?,
                    teams: Vec::new(),
                },
            })
        })?
        .filter_map(|x| x.ok())
        .collect();
    Ok(result)
}

//This function queries the database for all leagues. If there are no leagues in the database, the user is prompted to create one.
pub fn league_check(
    conn: &mut Connection,
    thread: &mut ThreadRng,
    input: LoadLeagueInput,
) -> Result<(), EditLeagueError> {
    // We query the database to get all the leagues that already exist.
    let options = match get_all_leagues_from_db(conn) {
        Ok(wrappers) => wrappers,
        Err(message) => return Err(EditLeagueError::DatabaseError(message)),
    };
    // If there are no leagues in the database, the user is prompted to create a league
    if options.is_empty() {
        println!("No Leagues created yet! Let's create a new league to get started.");
        create_new_league(thread, conn)
    } else {
        //Otherwise, the user is shown a list of all leagues that currently exist, and is prompted to select one.
        let ans: Result<LeagueWrapper, InquireError> =
            Select::new("Select an existing league", options).prompt();
        match ans {
            Ok(select) => match input {
                //If the users decided they wanted to create a new team earlier they are taken to the prompt to create a new team
                LoadLeagueInput::EditLeague(edit) => {
                    match load_league(thread, conn, select, edit) {
                        Ok(()) => Ok(()),
                        Err(message) => Err(message),
                    }
                }
                //Otherwise, the league is saved to the users disk.
                LoadLeagueInput::RefreshLeague => {
                    println!("Refreshing league.");
                    save_league(&select.league);
                    Ok(())
                }
                LoadLeagueInput::ViewSchedule => view_schedule(&select.league, conn),
                LoadLeagueInput::ViewRankings => select.league.display_ranking(conn),
            },
            Err(message) => inquire_check(message),
        }
    }
}

// Once a league is saved, we save a copy of the league data in a folder.
pub fn save_league_to_folders(league: &League) -> std::io::Result<()> {
    println!();
    let flder_path_string = league.name.to_string();
    let folder_path = Path::new(&flder_path_string);
    fs::create_dir_all(folder_path)?;

    for team in &league.teams {
        let file_path = folder_path.join(format!("{}.txt", team.name).as_str());

        let mut file = File::create(file_path)?;
        file.write_all(team.to_string().as_bytes())?;
    }
    Ok(())
}

pub fn save_league(league: &League) {
    let save_league_attempt = save_league_to_folders(league);
    match save_league_attempt {
        Ok(()) => println!("League saved successfully."),
        Err(message) => {
            println!(
                "Unable to save league to a folder.\nThe error was {}",
                message
            );
            println!("If you were able to fix the issue with the folder, use the Refresh league option from the main menu to save the league as a folder.")
        }
    }
}
