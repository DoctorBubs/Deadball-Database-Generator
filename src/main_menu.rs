use core::fmt;

use crate::edit_league_error::EditLeagueError;
use crate::{
    inquire_check, league::create_new_league, league_check, league_template::load_new_template,
};
use inquire::{InquireError, Select};
use rand::rngs::ThreadRng;
use rusqlite::Connection;
#[derive(Copy, Clone, Debug)]
pub enum LoadLeagueInput {
    EditLeague(EditLeagueInput),
    RefreshLeague,
    ViewSchedule,
    ViewRankings,
}

#[derive(Copy, Clone, Debug)]
pub enum EditLeagueInput {
    CreateNewTeam,
    CreateSchedule,
    GeneratePennant,
    CreateArchive,
}
// MenuInput contains all the valid choices a user can use at the main menu.
#[derive(Copy, Clone, Debug)]
pub enum MenuInput {
    CreateNewLeague,
    LoadExistingLeague(LoadLeagueInput),
    LoadLeagueFromTemplate,
    Exit,
}

impl fmt::Display for MenuInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::CreateNewLeague => "Create a new league.",
            Self::LoadExistingLeague(input) => match input {
                LoadLeagueInput::RefreshLeague => "Refresh an existing league.",
                LoadLeagueInput::EditLeague(edit_input) => match edit_input {
                    EditLeagueInput::CreateNewTeam => "Create a new team.",
                    EditLeagueInput::CreateSchedule => "Generate a schedule for an existing league",
                    EditLeagueInput::GeneratePennant => "Generate a new pennant chase.",
                    EditLeagueInput::CreateArchive => "Archive a copy of a league in it's current state in the database as well as a text file."
                },
                LoadLeagueInput::ViewSchedule => "View schedule.",
                LoadLeagueInput::ViewRankings => "View the top 10 batter or pitchers for a league.",
            },
            Self::LoadLeagueFromTemplate => "Create a new league from a template.",
            Self::Exit => "Exit",
        };
        write!(f, "{}", chars)
    }
}

pub enum RankingsChoice {
    Batters,
    Pitchers,
}

impl fmt::Display for RankingsChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Batters => "View top 10 Batters",
            Self::Pitchers => "View top 10 Pitchers",
        };
        write!(f, "{}", text)
    }
}

pub fn run_main_menu(conn: &mut Connection, thread: &mut ThreadRng) -> Result<(), EditLeagueError> {
    // We load a vector of the possible options a view can pick in the main menu.
    let new_team = EditLeagueInput::CreateNewTeam;
    let new_pennant = EditLeagueInput::GeneratePennant;
    let new_archive = EditLeagueInput::CreateArchive;
    //let new_sched = EditLeagueInput::CreateSchedule;
    let starting_options: Vec<MenuInput> = vec![
        MenuInput::CreateNewLeague,
        MenuInput::LoadExistingLeague(LoadLeagueInput::EditLeague(new_team)),
        MenuInput::LoadExistingLeague(LoadLeagueInput::EditLeague(new_pennant)),
        MenuInput::LoadExistingLeague(LoadLeagueInput::RefreshLeague),
        MenuInput::LoadExistingLeague(LoadLeagueInput::ViewRankings),
        MenuInput::LoadExistingLeague(LoadLeagueInput::EditLeague(new_archive)),
        // Uncomment the next 2 lines to enable schedule generation.
        //MenuInput::LoadExistingLeague(LoadLeagueInput::EditLeague(new_sched)),
        //MenuInput::LoadExistingLeague(LoadLeagueInput::ViewSchedule),
        MenuInput::LoadLeagueFromTemplate,
        MenuInput::Exit,
    ];
    // We prompt the user via Inquire.
    let starting_choice: Result<MenuInput, InquireError> =
        Select::new("What would you like to do?", starting_options).prompt();

    match starting_choice {
        Ok(choice) => match choice {
            // And we use the input to direct the user to where they would like to go first.
            MenuInput::CreateNewLeague => create_new_league(thread, conn),
            // If the user selects exit, the function returns Ok, which exit the program
            MenuInput::Exit => Ok(()),
            //Both CreateNewTeam and RefreshLeague are used in the league check function, so a selection of either will call the function.
            MenuInput::LoadExistingLeague(choice) => match league_check(conn, thread, choice) {
                Ok(_) => Ok(()),
                Err(message) => Err(message),
            },
            MenuInput::LoadLeagueFromTemplate => match load_new_template(conn, thread) {
                Ok(_) => Ok(()),
                Err(message) => Err(message),
            },
        },

        Err(message) => inquire_check(message),
    }
}
