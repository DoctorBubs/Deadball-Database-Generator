use core::fmt;

use inquire::{InquireError, Select};
use rand::rngs::ThreadRng;
use rusqlite::Connection;

use crate::{league::create_new_league, league_check};
#[derive(Copy, Clone, Debug)]
pub enum LoadLeagueInput {
    EditLeague(EditLeagueInput),
    RefreshLeague,
}

#[derive(Copy, Clone, Debug)]
pub enum EditLeagueInput {
    CreateNewTeam,
    CreateSchedule,
}
// MenuInput contains all the valid choices a user can use at the main menu.
#[derive(Copy, Clone, Debug)]
pub enum MenuInput {
    CreateNewLeague,
    LoadExistingLeague(LoadLeagueInput),
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
                },
            },

            Self::Exit => "Exit",
        };
        write!(f, "{}", chars)
    }
}

pub fn run_main_menu(conn: &mut Connection, thread: &mut ThreadRng) -> std::io::Result<()> {
    // We load a vector of the possible options a view can pick in the main menu.
    let new_team = EditLeagueInput::CreateNewTeam;
    let new_sched = EditLeagueInput::CreateSchedule;
    let starting_options: Vec<MenuInput> = vec![
        MenuInput::CreateNewLeague,
        MenuInput::LoadExistingLeague(LoadLeagueInput::EditLeague(new_team)),
        MenuInput::LoadExistingLeague(LoadLeagueInput::RefreshLeague),
        MenuInput::LoadExistingLeague(LoadLeagueInput::EditLeague(new_sched)),
        MenuInput::Exit,
    ];
    // We prompt the user via Inquire.
    let starting_choice: Result<MenuInput, InquireError> =
        Select::new("What would you like to do?", starting_options).prompt();

    match starting_choice {
        Ok(choice) => match choice {
            // And we tuse the input to direct the user to where they would like to go first.
            MenuInput::CreateNewLeague => create_new_league(thread, conn),
            // If the user selects exit, the functinon returns Ok, which exit the program
            MenuInput::Exit => Ok(()),
            //Both CreateNewTeam and RefreshLeague are used in the league check function, so a selection of either will call the function.
            MenuInput::LoadExistingLeague(choice) => {
                league_check(conn, thread, choice).unwrap();
                Ok(())
            }
        },

        Err(_) => {
            println!("Error matching first choice");
            Ok(())
        }
    }
}
