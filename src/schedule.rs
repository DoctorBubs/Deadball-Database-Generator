use crate::team::Team;


pub struct Game{
    home_team_id: i32,
    away_team_id:i32

}



fn new_game(home_team_id: i32, away_team_id: i32) -> Game{

    Game{
        home_team_id,
        away_team_id
    }


}

fn new_series(home_team_id: i32, away_team_id: i32, length:i32) -> Vec<Game>{
   /*  let mut result = Vec::new();
    for _ in 1..=length{
        result.push(new_game(home_team_id, away_team_id))
    };

    result */

    vec![new_game(home_team_id, away_team_id),length]
}


fn home_team_matchups(home_team_id: i32,away_team_id: i32,series_length: i32, total_series:i32)-> Vec<Vec<Game>>{

    vec![new_series(home_team_id, away_team_id, series_length),total_series]
}
fn new_home_season(team_id: i32, all_team_ids: &Vec<i32>, series_length: i32,total_series:i32) -> Vec<Vec<Game>>{

    all_team_ids.iter()
        .filter(|x| x != team_id)
        .map(|x| home_team_matchups(team_id, x, series_length, total_series))
        .reduce(|mut acc, e|{ 
            acc.push(e); 
            acc
        })
        .collect()
        .unwrap_or(Vec::new())
}

pub fn new_season(teams: &Vec<teams>, series_length: i32, series_per_matchup:i32) -> Vec<Vec<Vec<Game>>>{

    let ids: Vec<i32> = teams.into_iter.map(|x| x.team_id).collect();
    ids.iter()
        .map(|x| new_home_season(x, &ids, series_length, series_per_matchup))
        .reduce(|mut acc,e|{
            acc.push(e);
            acc
        })
        .collect()
}