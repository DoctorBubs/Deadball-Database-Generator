use itertools::Itertools;
use rand::{rngs::ThreadRng, seq::IteratorRandom, Rng};
#[derive(Debug, Clone, Copy)]
/// Represents how many wins and losses a team has in the current pennant.
pub struct PennantStanding {
    pub wins: i32,
    pub losses: i32,
}



use crate::edit_league_error::EditLeagueError;
/// Creates a tuple containing the wins and losses of a first place team. The teams winning percntage will be approximately 60 - 70%.
fn get_first_place_standings(games_played: i32, thread: &mut ThreadRng) -> (i32, i32) {
    // We convert games_played into a float.
    let g_float = games_played as f32;
    // We calculate what the min and max number of wins the team will have based off a percentage.
    let lower_limit = (0.50 * g_float).floor() as i32;
    let upper_limit = (0.70 * g_float).floor() as i32;
    // We calculate the number of wins via a random range.
    let wins = thread.gen_range(lower_limit..=upper_limit);
    (wins, games_played - wins)
}


fn generate_losers(
    games_played: i32,
    total_games_played: i32,
    max_wins: i32,
    standings_needed: usize,
    top_3_wins: i32,
    top_3_losses: i32,
    thread: &mut ThreadRng,
    max_other_wins: i32,
) -> Option<Vec<PennantStanding>> {
    let mut result = None;

    let mut min_winning_percentage = 0.50;
    loop {
        // If we have gotten this far that winning percentage is less than 0, we return an error
        if min_winning_percentage < 0.0 {
            break;
        }
        // We calculate the minimum number of wins based off the percentage.
        let minimum_wins = ((games_played as f32) * min_winning_percentage).floor() as i32;
        // Likewise, we calculate the max number of losses
        let max_losses = games_played - minimum_wins;
        // First, we use minimum wins and amx other wins to create a range,
        let loser_standings = (minimum_wins..=max_other_wins)
            // And we get permutations of 2. This create a vector, with value 0 being a teams wins, and 1 being a teams losses.
            .permutations(2)
            .map(|x| {
                (PennantStanding {
                    wins: x[0],
                    losses: x[1],
                })
            })
            // We filter to the permutations to ensure there are not too many losses, as well as making sure the sums of the wins and losses are correct.
            .filter(|x| ((x.losses <= max_losses) & (x.wins + x.losses == games_played)))
            // We next get get the combination of permutation that fits the number of teams we need PennantStandings for.
            .combinations(standings_needed)
            // Of the combos generated, we filter out the ones that have an incorrect numbers of wins and losses
            .filter(|x| {
                let total_wins = x.iter().fold(0, |acc, a| a.wins + acc) + top_3_wins;
                let total_losses = x.iter().fold(0, |acc, a| a.losses + acc) + top_3_losses;
                (((total_wins + total_losses) / 2) == total_games_played)
                    & (total_wins == total_losses)
            })
            // And we choose a random selection.
            .choose(thread);
        // If we didn't create valid PennantStanding, we go for it again but with a lower min_winning percentage
        match loser_standings {
            None => min_winning_percentage += -0.05,

            Some(mut standings) => {
                //otherwise, we sort the PennantStanding by wins, and return it in a some.
                standings.sort_by(|a, b| a.wins.cmp(&b.wins));
                result = Some(standings);
                break;
            }
        }
    }
    result
}

pub fn generate_pennant_standings(
    games_played: i32,
    thread: &mut ThreadRng,
    total_teams: i32,
) -> Result<Vec<PennantStanding>, EditLeagueError> {
    // We calculate how many games have been played by all teams.
    let total_games_played = (total_teams / 2) * games_played;
    // We calculate how many more teams will need to have PennantStandings generated.
    let standings_needed = (total_teams - 3) as usize;

    let mut final_standings = Err(EditLeagueError::PennantError("Unable to generate a pennant race with the current setup, try altering the number of teams and/or games".to_string()));
    // We loop for 100 times in the event we can not generate good PennantStandings.
    for _ in 0..100 {
        let (first_place_wins, first_places_losses) =
            get_first_place_standings(games_played, thread);
        // Next, we randomly select the second place team to be 1-3 games behind first.
        let second_games_behind = thread.gen_range(0..=3);
        let (second_place_wins, second_place_losses) = (
            first_place_wins - second_games_behind,
            first_places_losses + second_games_behind,
        );
        // And we make the third place team be 1-3 games behind second place.
        let third_games_behind = thread.gen_range(0..=3);
        let (third_place_wins, third_place_losses) = (
            second_place_wins - third_games_behind,
            second_place_losses + third_games_behind,
        );
        // We save the wins and losses of the top 3 teams in a vector.
        let mut top_3_standings = vec![
            PennantStanding {
                wins: third_place_wins,
                losses: third_place_losses,
            },
            PennantStanding {
                wins: second_place_wins,
                losses: second_place_losses,
            },
            PennantStanding {
                wins: first_place_wins,
                losses: first_places_losses,
            },
        ];

        // We also get a sum of the top 3 teams wins and losses.
        let top_3_wins = first_place_wins + second_place_wins + third_place_wins;
        let top_3_losses = first_places_losses + second_place_losses + third_place_losses;
        // Next, we limit the amount of wins a non top 3 team can win to  1 below thir place
        let max_other_wins = third_place_wins - 1;

        // We generate the PennantStandings for the rest of the teams.
        let loser_standings = generate_losers(
            games_played,
            total_games_played,
            max_other_wins,
            standings_needed,
            top_3_wins,
            top_3_losses,
            thread,
            max_other_wins,
        );
        // If we receive a good result, we add the top 3 standings to the new PennantStanding and return it.
        if let Some(mut standings) = loser_standings {
            standings.append(&mut top_3_standings);
            final_standings = Ok(standings);
            break;
        };
    }
    final_standings
}
