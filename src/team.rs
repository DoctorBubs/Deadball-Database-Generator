use rusqlite::Connection;

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
//Thus, Modern pitchers in the rotation are marked as SP, while anienct are just P
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
    //.map(|score| format!("\n{}", &score.string))
    //.collect()
    /*.reduce(|acc, e| format!("{}{}", acc, e))
    .unwrap_or_else(|| "\n".to_string())*/

    //.collect()
}

fn get_batter_info_string(desc: String, vec: &[Player]) -> String {
    let header = format!("{}:\nName Pos Age Hand BT OBT Traits", desc);
    format!("{}{}\n", header, get_sorted_batter_strings(vec))
}

fn get_pitcher_info_string(desc: String, vec: &[Player]) -> String {
    let header = format!("{}:\nName Pos Age Hand PD Trait BT OBT", desc);
    format!("{}{}\n", header, sorted_pitcher_pool(vec))
}

/*struct LineupSlices(Vec<LineupScore>,Vec<LineupScore>);

fn get_lineup_slices(vec: &Vec<Player>) -> LineupSlices {

    let mut scores: Vec<LineupScore> = vec.iter().map(|player| player.get_lineup_score()).collect();
    scores.sort();
    scores.reverse();
    LineupSlices(scores[0..=3].to_vec(),scores[4..=7].to_vec())

}

fn sort_lineup_slice(slices: LineupSlices) -> String{

    let LineupSlices(first_four,rest) = slices;
    let

}*/

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
            starter.save_sql(conn, team_id, TeamSpot::StartingLineup);
        }

        for bench in &self.bench {
            bench.save_sql(conn, team_id, TeamSpot::StartingLineup);
        }

        for starter in &self.starting_pitching {
            starter.save_sql(conn, team_id, TeamSpot::StartingPitcher);
        }

        match &self.bullpen {
            Some(pen) => {
                for reliever in pen {
                    reliever.save_sql(conn, team_id, TeamSpot::Bullpen);
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
            "Name:{} ,Abrv.{}, Team Score: {}\n",
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
