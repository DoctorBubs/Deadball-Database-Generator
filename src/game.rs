
enum Outcome {
    Out(i32),
    Walk,
    Hit(i32),
    Poss_Error(i32),
    Oddity,
}

impl Outcome {
    fn to_string(&self) -> String {
        match self {
            Self::Out(num) => format!("Out {}", num.to_string()),
            Self::Walk => "Walk".to_string(),
            Self::Hit(num) => format!("Hit {}", num.to_string()),
            Self::Poss_Error(num) => format!("Poss Error {}", num.to_string()),
            Self::Oddity => "Oddity".to_string(),
        }
    }
}



fn match_check(num: i32) -> bool {
    match num {
        1 => true,
        2..=5 => true,
        13..=19 => false,
        _ => true,
    }
}

fn last_digit(num: i32) -> i32 {
    let ab_num = num.abs();
    match ab_num < 10 {
        true => ab_num,
        false => ab_num % 10,
    }
}


struct Game<'a> {
    home_team: &'a mut Team,
    away_team: &'a mut Team,
}

impl Game<'_> {
    fn new<'a>(home: &'a mut Team, away: &'a mut Team) -> Game<'a> {
        Game {
            home_team: home,
            away_team: away,
        }
    }

    fn to_string(&self) -> &String {
        &self.home_team.nick_name
    }
}

fn play_game<'a>(game: Game<'a>, mut thread: &'a mut ThreadRng) {
    let Game {
        home_team: home,
        away_team: away,
    } = game;
    let mut favorite;
    let mut underdog;
    match home.team_score >= away.team_score {
        true => {
            favorite = home;
            underdog = away;
        }
        false => {
            favorite = away;
            underdog = home;
        }
    };

    let mut diff = favorite.team_score - underdog.team_score + 50;
    match diff <= 80 {
        true => diff = 80,
        false => (),
    };
    let roll = thread.gen_range(1..=99);
    let mut winner;
    let mut loser;
    match roll <= diff {
        true => {
            winner = favorite;
            loser = underdog;
        }

        false => {
            winner = underdog;
            loser = favorite;
        }
    }

    winner.wins += 1;
    loser.losses += 1;
    println!("Winner = {}", winner.nick_name);
}

fn plate_appearance(bt: &i32, obt: &i32, die: &PD, mut thread: &mut ThreadRng) -> String {
    let mss = die.roll(&mut thread) + thread.gen_range(1..=99);
    match mss <= *bt {
        true => "Hit".to_string(),
        false => match mss <= *obt {
            true => "Walk".to_string(),
            false => "Out".to_string(),
        },
    }
}

fn less_than_target(mss: i32, target: i32) -> bool {
    mss <= target
}

fn get_outcome(batter: &Player, pitcher: &Player, mut thread: &mut ThreadRng) -> Outcome {
    let die = pitcher.get_base_pd();
    let mss = die.roll(&mut thread) + thread.gen_range(1..=99);
    match mss {
        1 => Outcome::Oddity,
        99 => Outcome::Oddity,
        _ => batter.compare_target(mss),
    }
}

fn form_check(num: i32) -> i32 {
    num * 2
}

fn new_pa(batter: &Player, pitcher: &Player, mut thread: &mut ThreadRng) {
    /*let roll = pitcher.base_pd_roll_with_string(thread);*/
    println!("{}'s BT = {}, OBT = {}", batter.name, batter.bt, batter.obt);
    let PD_Roll(pitcher_roll_i, pitcher_roll_string) = pitcher.base_pd_roll_with_string(thread);
    println!("{}", pitcher_roll_string);
    let batter_roll = thread.gen_range(1..=99);
    println!("{} rolled a {}", batter.name, batter_roll);
    let mss = pitcher_roll_i + batter_roll;
    println!("Mss = {}", mss);
    let outcome = batter.compare_target(mss);
    println!("Outcome = {}", outcome.to_string())
}
