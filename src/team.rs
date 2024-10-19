use crate::b_traits::BTraits;
use crate::edit_league_error::handle_serde_error;
use crate::edit_league_error::handle_sql_error;
use crate::edit_league_error::EditLeagueError;
use crate::inquire_check;
use crate::note::Notable;
use crate::note::Note;

use crate::position::PlayerPosition;
use crate::traits::Contact;
use crate::traits::Defense;
use crate::traits::Power;
use crate::traits::Speed;
use crate::traits::Toughness;
use inquire::validator::MaxLengthValidator;
use inquire::validator::MinLengthValidator;
use inquire::Confirm;
use inquire::Text;
use rusqlite::Connection;

use serde_json::Value;

use crate::league::save_league;
use crate::league::League;

use crate::lineup_score::LineupScore;
use crate::pitcher_rank_info::PitcherRankInfo;
use crate::BatterQuality;
use crate::Deserialize;
use crate::Era;
use crate::PitcherQuality;
use crate::Player;
use crate::PlayerGender;
use crate::PlayerQuality;
use crate::Serialize;
use crate::ThreadRng;
use core::fmt;

use core::panic;
use std::fmt::Write;

/* A teams consists of a name, a vector for the starting lineup, bench, pitching rotation, and an option for the bullpen.
Team's also have a team score, which is used in Deadball to simulate a game with only a few dice rolls.' */
#[derive(Serialize, Deserialize, Debug)]
pub struct Team {
    pub abrv: String,
    pub name: String,
    pub lineup: Vec<Player>,
    pub bench: Vec<Player>,
    pub starting_pitching: Vec<Player>,
    pub bullpen: Option<Vec<Player>>,

    pub team_score: i32,
    pub wins: i32,
    pub losses: i32,
    pub team_id: i64,
    pub note: Note,
}

impl Team {
    pub fn new(
        abrv: &String,
        name: &String,
        gender: PlayerGender,
        era: Era,
        thread: &mut ThreadRng,
    ) -> Team {
        let mut new_team = Team {
            team_id: 0,
            abrv: abrv.to_string(),
            name: name.to_string(),
            lineup: new_starting_lineup(gender, thread, era),
            bench: new_bench(gender, thread, era),
            starting_pitching: new_rotation(gender, thread, era),
            bullpen: new_bullpen(gender, thread, era),
            team_score: 0,
            wins: 0,
            losses: 0,
            note: None,
        };

        new_team.calc_team_score();
        new_team
    }

    pub fn calc_team_score(&mut self) {
        // To calculate a team score, first we add up all the BT of each batter on the team.
        let batter_score = team_score_from_vec(&self.lineup) + team_score_from_vec(&self.bench);
        //Next, we add the the team score contribution from the pitching rotation to get a pitching score.
        let mut pitcher_score = team_score_from_vec(&self.starting_pitching);
        //If a team has a bullpen, the pitchers in the bullpen add to the pitching score.
        match &self.bullpen {
            Some(bullpen) => {
                pitcher_score += team_score_from_vec(bullpen);
            }

            None => (),
        };
        // Next, we multiply the pitcher score by 7.
        pitcher_score *= 7;
        // Finally, the team score is calculated by adding the batter score to the pitcher score and divide by 10.
        self.team_score = (batter_score + pitcher_score) / 10;
    }

    /* For each player in a team, we create a new entry in the database with a provided team id.
    The TeamSpot enum is used to distinguish each players role on the team in its entry in the database.
    */
    pub fn save_players_sql(
        &mut self,
        conn: &mut Connection,
        team_id: i64,
    ) -> Result<(), EditLeagueError> {
        for starter in &mut self.lineup {
            starter.save_sql(conn, team_id, TeamSpot::StartingLineup)?;
        }

        for bench in &mut self.bench {
            bench.save_sql(conn, team_id, TeamSpot::BenchHitter)?;
        }

        for starter in &mut self.starting_pitching {
            starter.save_sql(conn, team_id, TeamSpot::StartingPitcher)?;
        }
        // As not every team has a bullpen, we do a check to make sure.
        match &mut self.bullpen {
            Some(pen) => {
                for reliever in pen {
                    reliever.save_sql(conn, team_id, TeamSpot::Bullpen)?;
                }
            }
            None => (),
        };
        Ok(())
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let base_info = format!(
            "Name:{}, Abrv:{}, Team Score: {}\n",
            self.name, self.abrv, self.team_score
        );
        let lineup_string = get_batter_info_string("Lineup".to_string(), &self.lineup);
        let bench_string = get_batter_info_string("Bench".to_string(), &self.bench);
        let rotation_string =
            get_pitcher_info_string("Rotation".to_string(), &self.starting_pitching);
        let non_bullpen_string = format!(
            "{}{}{}{}",
            base_info, lineup_string, bench_string, rotation_string
        );
        let chars = match &self.bullpen {
            Some(bullpen) => format!(
                "{}{}",
                non_bullpen_string,
                get_pitcher_info_string("Bullpen".to_string(), bullpen)
            ),
            None => non_bullpen_string,
        };

        write!(f, "{}", chars)
    }
}

impl Notable for Team {
    fn get_note(&self) -> &Note {
        &self.note
    }

    fn get_note_input_string(&self) -> String {
        format!("Please enter the note you wish to attach to {}", self.name)
    }
}

// Take a vector of players, and reduces it to the sum of how much they contribute to a team score.
// Batters contribution is based off their BT, while pitchers is based off their pitch dice.
fn team_score_from_vec(vec: &[Player]) -> i32 {
    vec.iter()
        .map(|player| player.get_team_score_contribution())
        .reduce(|acc, e| acc + e)
        .unwrap_or(0)
}

fn sorted_pitcher_pool(vec: &[Player]) -> String {
    let mut ranks: Vec<PitcherRankInfo> = vec
        .iter()
        .map(|player| player.get_pitcher_rank_info())
        .collect();
    ranks.sort();
    ranks.iter().rev().fold(String::new(), |mut output, rank| {
        let new_str = &rank.string;
        let _ = write!(output, "\n{new_str}");
        output
    })

    /*.reduce(|acc, e| format!("{}{}", acc, e))
    .unwrap_or_else(|| "\n".to_string())*/
}

fn get_sorted_batter_strings(vec: &[Player]) -> String {
    let mut scores: Vec<LineupScore> = vec.iter().map(|player| player.get_lineup_score()).collect();
    scores.sort();
    scores
        .iter()
        .rev()
        .fold(String::new(), |mut output, score| {
            let new_str = &score.string;
            let _ = write!(output, "\n{new_str}");
            output
        })
}
/// PlayerWrapper contains fields that need to be deserialzed by serde.
struct PlayerWrapper {
    team_spot: Value,
    player: Player,
    hand: Value,
    pd: Value,
    pd_int: i32,
    pos: String,
    pitcher_trait: Value,
    contact: Value,
    defense: Value,
    power: Value,
    speed: Value,
    toughness: Value,
    note: Value,
}

impl PlayerWrapper {
    // Deserializes the JSON values in the wrapper, and creates a new player based off those values and the preexisting player in the wrappers
    fn gen_player(
        &self,
        conn: &mut Connection,
        era: Era,
    ) -> Result<(TeamSpot, Player, i32), serde_json::Error> {
        let player_name = &self.player.name;
        let team_spot = serde_json::from_value(self.team_spot.clone())?;
        let player_id = self.player.player_id;
        let pd = serde_json::from_value(self.pd.clone())?;
        let pos: PlayerPosition = match serde_json::from_str(&self.pos) {
            Ok(position) => position,
            Err(input) => {
                match PlayerPosition::fix_pos(conn, player_id, player_name, era, pd, &input) {
                    None => return Err(input),
                    Some(value) => value,
                }
            }
        };
        let new_player = Player {
            name: self.player.name.clone(),
            pos,
            hand: serde_json::from_value(self.hand.clone())?,
            pd,
            pitcher_trait: serde_json::from_value(self.pitcher_trait.clone())?,
            b_traits: BTraits {
                contact: serde_json::from_value(self.contact.clone()).unwrap_or(Contact::C0),
                defense: serde_json::from_value(self.defense.clone()).unwrap_or(Defense::D0),
                power: serde_json::from_value(self.power.clone()).unwrap_or(Power::P0),
                speed: serde_json::from_value(self.speed.clone()).unwrap_or(Speed::S0),
                toughness: serde_json::from_value(self.toughness.clone()).unwrap_or(Toughness::T0),
            },
            note: serde_json::from_value(self.note.clone())?,
            // The remaining fields can be copied over from the original player saved in the wrapper.
            ..self.player
        };
        Ok((team_spot, new_player, self.pd_int))
    }
}

pub fn load_team(conn: &mut Connection, mut team: Team, era: Era) -> Result<Team, EditLeagueError> {
    // We prepare a statement that will select all players from the database that has a matching team id
    println!("Loading Teams");
    let mut stmt = handle_sql_error(conn.prepare(
        "SELECT 
        team_spot,player_name,age,pos,hand,bt,obt_mod,obt,PD,pitcher_trait,contact,defense,power,speed,toughness,trade_value,team_id,player_id,pd_int,player_note
        FROM players 
        WHERE team_id = ?1"
    ))?;
    // We use the statement to query the database
    let rows_received = stmt.query_map([team.team_id], |row| {
        //For each result that matches the query, we create a new player wrapper that is wrapped in an Ok.
        Ok(PlayerWrapper {
            // Team spot is deserialized from the team spot row.
            team_spot: row.get(0)?,
            pos: row.get(3)?,
            hand: row.get(4)?,
            pd: row.get(8)?,
            pitcher_trait: row.get(9)?,
            contact: row.get(10)?,
            defense: row.get(11)?,
            power: row.get(12)?,
            speed: row.get(13)?,
            toughness: row.get(14)?,
            pd_int: row.get(18).unwrap_or(0),
            note: row.get(19)?,
            // And we use the rest to fill out the player.
            player: Player {
                name: row.get(1)?,
                age: row.get(2)?,
                bt: row.get(5)?,
                obt_mod: row.get(6)?,
                obt: row.get(7)?,
                trade_value: row.get(15)?,
                team_id: row.get(16)?,
                player_id: row.get(17)?,
                ..Player::default()
            },
        })
    });
    let player_iter: Vec<Result<PlayerWrapper, rusqlite::Error>> =
        handle_sql_error(rows_received)?.collect();
    drop(stmt);
    for r in player_iter {
        let pw = handle_sql_error(r)?;
        let (team_spot, player, pd_int) = handle_serde_error(pw.gen_player(conn, era))?;

        // We check if the loaded player has any error, e.g age is 0 or obt != bt + obt_,mod
        let player_error_opt = player.get_player_error(pd_int);
        // If there is an error, we print a warning.
        if let Some(player_error) = player_error_opt {
            println!("{}", player_error)
        };
        // And based off the team spot, the player is assigned to the correct player pool.
        match team_spot {
            TeamSpot::StartingLineup => team.lineup.push(player),
            TeamSpot::BenchHitter => team.bench.push(player),
            TeamSpot::StartingPitcher => team.starting_pitching.push(player),
            TeamSpot::Bullpen => match &mut team.bullpen {
                Some(pen) => pen.push(player),
                None => {
                    // If the database has been altered to include a reliever in an Ancient era team, we warn the user and save the player in the team's pitching pool.
                    println!("Warning: {} is listed in the database as a reliever, however {} is an Ancient Era team that does not have a bullpen.",player.name, team.name);
                    println!("Please fix the listing for this player in the database, the player's id is {}.Afterward, please refresh this league via the main menu", player.player_id);
                    println!(
                        "For now, {} will be added to the pitcher pool of {}.",
                        player.name, team.name
                    )
                }
            },
        }
    }
    team.calc_team_score();
    Ok(team)
}

// This function is used to create vectors of players, based off a vector of strings.
fn new_player_vec<T: Copy + PlayerQuality>(
    vec: Vec<&str>,
    gender: PlayerGender,
    thread: &mut ThreadRng,
    quality: T,
    era: Era,
) -> Vec<Player> {
    vec.into_iter()
        .map(|x| Player::new(x.into(), gender, quality, thread, era))
        .collect()
}

// A starting lineup consists of 8 players, one for each position on the field

fn new_starting_lineup(gender: PlayerGender, thread: &mut ThreadRng, era: Era) -> Vec<Player> {
    let base = vec!["C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"];
    new_player_vec(base, gender, thread, BatterQuality::TopProspect, era)
}

// The bench consist of all non starters. The Ancient and Modern era's have different quantities and positions on the bench, so we use the Era enum to keep track.
fn new_bench(gender: PlayerGender, thread: &mut ThreadRng, era: Era) -> Vec<Player> {
    let base = match era {
        Era::Ancient => vec!["C", "INF", "OF", "UT"],
        Era::Modern => vec!["C", "INF", "INF", "OF", "OF"],
    };

    new_player_vec(base, gender, thread, BatterQuality::Farmhand, era)
}

// The ancient and modern era have different definitions of what a pitcher is. Modern era pitchers can be either starters or relievers, while the ancient era does not make the distinction.
//Thus, Modern pitchers in the rotation are marked as SP, while ancient are just P
fn new_rotation(gender: PlayerGender, thread: &mut ThreadRng, era: Era) -> Vec<Player> {
    let base = match era {
        Era::Ancient => vec!["P", "P", "P", "P", "P"],
        Era::Modern => vec!["SP", "SP", "SP", "SP", "SP"],
    };

    new_player_vec(base, gender, thread, PitcherQuality::TopProspect, era)
}
// Ancient Era teams do not have a bullpen, so a bullpen is wrapped in an option.
fn new_bullpen(gender: PlayerGender, thread: &mut ThreadRng, era: Era) -> Option<Vec<Player>> {
    match era {
        Era::Ancient => None,
        Era::Modern => {
            let base = vec!["RP"; 7];
            Some(new_player_vec(
                base,
                gender,
                thread,
                PitcherQuality::TopProspect,
                era,
            ))
        }
    }
}

// When we save team to a txt file, we print out the batter in an optimized order, with a header for their relevant stats.
fn get_batter_info_string(desc: String, vec: &[Player]) -> String {
    let header = format!("{}:\nName Pos Age Hand BT OBT Traits", desc);
    format!("{}{}\n", header, get_sorted_batter_strings(vec))
}
// Same goes for pitchers.
fn get_pitcher_info_string(desc: String, vec: &[Player]) -> String {
    let header = format!("{}:\nName Pos Age Hand PD Trait BT OBT", desc);
    format!("{}{}\n", header, sorted_pitcher_pool(vec))
}

// Declare if a player is in hte starting lineup, on the bench, in the starting rotation, or in the bullpen.
#[derive(Serialize, Deserialize, Debug)]
pub enum TeamSpot {
    StartingLineup,
    BenchHitter,
    StartingPitcher,
    Bullpen,
}

impl fmt::Display for TeamSpot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::StartingLineup => "Starting Lineup",
            Self::BenchHitter => "Bench Hitter",
            Self::StartingPitcher => "Starting Pitcher",
            Self::Bullpen => "Bullpen",
        };
        write!(f, "{}", chars)
    }
}

// After creating a new team, we ask the user if they would like to create another team.
pub fn add_team_check(
    league: &mut League,
    conn: &mut Connection,
    thread: &mut ThreadRng,
    league_id: i64,
) -> Result<(), EditLeagueError> {
    let ans = Confirm::new("Would you like to create another team?")
        .with_default(true)
        .prompt();

    match ans {
        // If the user selects true, the user adds another team, however we note that this is not the first team created for the league.
        Ok(true) => add_new_team(league, thread, conn, league_id, false)?,
        //If not, we save the league.
        Ok(false) => {
            save_league(league);
        }
        Err(message) => return inquire_check(message),
    };

    Ok(())
}
// Prompts the user to create a new team, while also ensure that the user does not use the same name or abbreviation for a team in the same league more than once.
pub fn add_new_team(
    league: &mut League,
    thread: &mut ThreadRng,
    conn: &mut Connection,
    league_id: i64,
    first_team: bool,
) -> Result<(), EditLeagueError> {
    let result: Result<(), EditLeagueError>;
    // If this is the first team generated for the league, we display a different prompt to the user.
    let mut prompt_string = match first_team {
        true => "Enter the name of the first team",
        false => "Enter the name of the new team",
    };
    let current_teams = &league.teams;
    if !current_teams.is_empty() {
        let names = current_teams.iter().fold(String::new(), |acc, team| {
            let new_string = format!("\n{} {}", team.abrv, team.name);
            acc + &new_string
        });
        println!(
            "This league currently includes the following teams:{}\n",
            names
        );
    };
    let abrv_min_validator = MinLengthValidator::new(2);
    let abrv_max_validator = MaxLengthValidator::new(4);
    let name_validator = MinLengthValidator::new(3);
    // Each team must have a unique name and abbreviation, we loop until we receive one.
    loop {
        let name_input = Text::new(prompt_string)
            .with_validator(name_validator.clone())
            .prompt();

        let team_name = match name_input {
            Ok(name) => name.trim().to_string(),
            Err(message) => return inquire_check(message),
        };

        let abrv_input = Text::new("Please enter an abbreviation for the new team.")
            .with_validator(abrv_min_validator.clone())
            .with_validator(abrv_max_validator.clone())
            .with_default(&team_name[0..=1].to_string().to_uppercase())
            .prompt();

        let abrv = match abrv_input {
            Ok(input) => input.trim().to_string(),
            Err(message) => return inquire_check(message),
        };
        /* The league takes the new team name and abbreviation created.  If there is already a team with the same name and/or abbreviation, an error is returned and the user is prompted to enter in something else.
            There is also a check to see if there is an error adding the team to the database, and returns an error if it does.
            Otherwise, the function will return OK.
        */
        match league.new_team(&abrv, &team_name, thread, league_id, conn) {
            Err(message) => {
                match message {
                    EditLeagueError::AbrvTaken => println!(
                        "This league already has a team with that abbreviation, please try again."
                    ),
                    EditLeagueError::NameTaken => {
                        println!("This league already has a team with that name, please try again.")
                    }
                    _ => return Err(message),
                };

                prompt_string = "Enter a unique team name.";
            }
            // If the league returns OK, we ask the user if they would like to create a new team.
            Ok(()) => {
                result = match add_team_check(league, conn, thread, league_id) {
                    Ok(()) => Ok(()),
                    Err(message) => Err(message),
                };
                break;
            }
        };
    }

    result
}
