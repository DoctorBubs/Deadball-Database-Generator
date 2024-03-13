
use crate::b_traits::BTraits;
use crate::league::AddTeamError;
use inquire::validator::MaxLengthValidator;
use inquire::validator::MinLengthValidator;
use inquire::Confirm;
use inquire::Text;
use rusqlite::Connection;

use crate::league::save_league;
use crate::league::League;
use crate::league::TeamWrapper;
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

use std::fmt::Write;

/* A teams consists of a name, a vector for the starting lineup, bench, pitching rotation, and an option for the bullpen.
Team's als ohave a team score, which is used in Deadball to simulate a game with only a few dice rolls.' */
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
            abrv: abrv.to_string(),
            name: name.to_string(),
            lineup: new_starting_lineup(gender, thread, era),
            bench: new_bench(gender, thread, era),
            starting_pitching: new_rotation(gender, thread, era),
            bullpen: new_bullpen(gender, thread, era),
            team_score: 0,
            wins: 0,
            losses: 0,
        };

        new_team.calc_team_score();
        new_team
    }

    pub fn calc_team_score(&mut self) {
        // To calculate a team score, first we add up all the BT of each batter on the team.
        let batter_score = team_score_from_vec(&self.lineup) + team_score_from_vec(&self.bench);
        //Next, we add the the team score contribution from the pitching rotation to get a pitching score.
        let mut pitcher_score = team_score_from_vec(&self.starting_pitching);
        //If a team has a bullepen, the pitchers in the bullpen add to the pitching score.
        match &self.bullpen {
            Some(bullpen) => {
                pitcher_score += team_score_from_vec(bullpen);
            }

            None => (),
        };
        // Next, we multiply the pitcher score by 7.
        pitcher_score *= 7;
        // Finally, the team score is caluclated by adding the batter score to the pitcher score and divide by 10.
        self.team_score = (batter_score + pitcher_score) / 10;
    }

    pub fn save_players_sql(
        &self,
        conn: &mut Connection,
        team_id: i64,
    ) -> Result<(), rusqlite::Error> {
        for starter in &self.lineup {
            starter.save_sql(conn, team_id, TeamSpot::StartingLineup)?;
        }

        for bench in &self.bench {
            bench.save_sql(conn, team_id, TeamSpot::BenchHitter)?;
        }

        for starter in &self.starting_pitching {
            starter.save_sql(conn, team_id, TeamSpot::StartingPitcher)?;
        }

        match &self.bullpen {
            Some(pen) => {
                for reliever in pen {
                    reliever.save_sql(conn, team_id, TeamSpot::Bullpen)?;
                }
            }
            None => (),
        };
        Ok(())
    }

    /*pub fn to_string(&self) -> String {
        let base_info = format!("Name:{} , Team Score: {}\n", self.name, self.team_score);
        let lineup_string = get_batter_info_string("Lineup".to_string(), &self.lineup);
        let bench_string = get_batter_info_string("Bench".to_string(), &self.bench);
        let rotation_string =
            get_pitcher_info_string("Rotation".to_string(), &self.starting_pitching);
        let non_bullpen_string = format!(
            "{}{}{}{}",
            base_info, lineup_string, bench_string, rotation_string
        );
        match &self.bullpen {
            Some(bullpen) => format!(
                "{}{}",
                non_bullpen_string,
                get_pitcher_info_string("Bullpen".to_string(), bullpen)
            ),
            None => non_bullpen_string,
        }
    }*/
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

// Takea vector of players, and reduces it to the sum of how much they contribute to a team score.
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

struct PlayerWrapper {
    team_spot: TeamSpot,
    player: Player,
}

pub fn load_team(conn: &mut Connection, wrapper: TeamWrapper) -> Result<Team, rusqlite::Error> {
    let TeamWrapper { team_id, mut team } = wrapper;
    //let stmt_string = format!("SELECT * FROM players WHERE team_id = {}", team_id);
    let mut stmt = conn.prepare("SELECT * FROM players WHERE team_id = ?1")?;
    let _test = 3;
    let team_iter = stmt.query_map([team_id], |row| {
        Ok(PlayerWrapper {
            team_spot: {
                let input: String = row.get(12)?;
                let chars = input.as_str();
                let output = serde_json::from_str(chars);
                output.unwrap()
            },

            player: Player {
                name: row.get(2)?,
                age: row.get(3)?,
                pos: row.get(4)?,
                hand: serde_json::from_value(row.get(5)?).unwrap(),
                bt: row.get(6)?,
                obt_mod: row.get(7)?,
                obt: row.get(8)?,
                pd: serde_json::from_value(row.get(9)?).unwrap(),
                pitcher_trait: serde_json::from_value(row.get(11)?).unwrap(),
                b_traits: BTraits {
                    contact: serde_json::from_value(row.get(14)?).unwrap(),
                    defense: serde_json::from_value(row.get(16)?).unwrap(),
                    power: serde_json::from_value(row.get(18)?).unwrap(),
                    speed: serde_json::from_value(row.get(20)?).unwrap(),
                    toughness: serde_json::from_value(row.get(22)?).unwrap(),
                },
            },
        })
    })?;

    for result in team_iter {
        let wrapper = result.unwrap();
        let PlayerWrapper { team_spot, player } = wrapper;
        match team_spot {
            TeamSpot::StartingLineup => team.lineup.push(player),
            TeamSpot::BenchHitter => team.bench.push(player),
            TeamSpot::StartingPitcher => team.starting_pitching.push(player),
            TeamSpot::Bullpen => match &mut team.bullpen {
                Some(pen) => pen.push(player),
                None => panic!("Attemped to add a reliever to a team with no bullpen"),
            },
        }

        /*let goal_vec = match team_spot{
            TeamSpot::StartingLineup => &mut team.lineup,
            TeamSpot::BenchHitter => &mut team.bench,
            TeamSpot::StartingPitcher => &mut team.starting_pitching,
            TeamSpot::Bullpen => &mut &team.bullpen.unwrap()
        };

        goal_vec.push(player)*/
    }
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
        .map(|x| Player::new(x.to_string(), gender, quality, thread, era))
        .collect()
}

// A starting lineup consists of 8 players, one for each position on the field
//Todo: option to have a DH?
fn new_starting_lineup(gender: PlayerGender, thread: &mut ThreadRng, era: Era) -> Vec<Player> {
    let base = vec!["C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"];
    new_player_vec(base, gender, thread, BatterQuality::TopProspect, era)
}

// The bench consist of all non starters. The Ancient and Modern era's have different quantities and posiions on the bech, so we use the Era enum to keep track.
fn new_bench(gender: PlayerGender, thread: &mut ThreadRng, era: Era) -> Vec<Player> {
    let base = match era {
        Era::Ancient => vec!["C", "INF", "OF", "UT"],
        Era::Modern => vec!["C", "INF", "INF", "OF", "OF"],
    };

    new_player_vec(base, gender, thread, BatterQuality::Farmhand, era)
}

// The ancienct and modern era have different definitions of what a pitcher is. Modern era pitchers can be either starters or relievers, while the ancient era does not make the distinction.
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

fn get_batter_info_string(desc: String, vec: &[Player]) -> String {
    let header = format!("{}:\nName Pos Age Hand BT OBT Traits", desc);
    format!("{}{}\n", header, get_sorted_batter_strings(vec))
}

fn get_pitcher_info_string(desc: String, vec: &[Player]) -> String {
    let header = format!("{}:\nName Pos Age Hand PD Trait BT OBT", desc);
    format!("{}{}\n", header, sorted_pitcher_pool(vec))
}

// Declare if a player is in hte starting lineup, on the bench, in the starting roation, or in the bullpen.
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
) -> std::io::Result<()> {
    let ans = Confirm::new("Would you like to create another team?")
        .with_default(true)
        .prompt();

    match ans {
        // If the user selects true, the user adds another team, however we note that this is not the first team created for the league.
        Ok(true) => add_new_team(league, thread, conn, league_id, false)?,
        //If not, we save the leauge.
        Ok(false) => save_league(league, conn, thread)?,
        Err(_) => {
            panic!("Error on add team prompt");
        }
    };

    Ok(())
}


pub fn add_new_team(
    league: &mut League,
    thread: &mut ThreadRng,
    conn: &mut Connection,
    league_id: i64,
    first_team: bool,
) -> std::io::Result<()> {
    let result: std::io::Result<()>;
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
            Err(_) => panic!("Error creating team name."),
        };

        let abrv_input = Text::new("Please enter an abbreviation for the new team.")
            .with_validator(abrv_min_validator.clone())
            .with_validator(abrv_max_validator.clone())
            .with_default(&team_name[0..=1].to_string().to_uppercase())
            .prompt();

        let abrv = match abrv_input {
            Ok(input) => input.trim().to_string(),
            Err(_) => panic!("Error creating team abbreviation."),
        };
        /*  The league takes the name and abreviation we just created. If there is already a team in the league with that name or abbreviation,  it returns an error.
        Otherwise, the league generates a new team, and then returns the team as a string wrapped in an Ok, which we use to save the team as a file on the disk.*/
        match league.new_team(&abrv, &team_name, thread, league_id, conn) {
            Err(message) => {
                match message {
                    AddTeamError::AbrvTaken => println!(
                        "This league already has a team with that abbreviation, please try again."
                    ),
                    AddTeamError::NameTaken => {
                        println!("This league already has a team with that name, please try again.")
                    }
                    AddTeamError::DatabaseError => {
                        println!("Error adding team to the data base, please try again.");
                        return Ok(());
                    }
                };
                //println!("Error {:?}",message);
                prompt_string = "Enter a unique team name.";
            }
            // If the league returns OK, we take the string, and write it to a new file in the leauge folder
            Ok(()) => {
                result = add_team_check(league, conn, thread, league_id);
                break;
            }
        };
    }

    result
}