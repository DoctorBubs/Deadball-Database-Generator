use core::fmt;

use inquire::{InquireError, Select};
use rand::rngs::ThreadRng;
use rusqlite::Connection;

use crate::{league::create_new_league, league_check};
// MenuInput contains all the valid choices a user can use at the main menu.
#[derive(Copy, Clone, Debug)]
pub enum MenuInput {
    CreateNewLeague,
    CreateNewTeam,
    RefreshLeague,
    Exit,
}

impl fmt::Display for MenuInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::CreateNewLeague => "Create a new league.",
            Self::CreateNewTeam => "Create a new team.",
            Self::RefreshLeague => "Refresh an existing league.",
            Self::Exit => "Exit.",
        };

        write!(f, "{}", chars)
    }
}

pub fn run_main_menu(conn: &mut Connection, thread: &mut ThreadRng) -> std::io::Result<()> {
   
    // We load a vector of the possible options a view can pick in the main menu.
    let starting_options: Vec<MenuInput> = vec![
        MenuInput::CreateNewLeague,
        MenuInput::CreateNewTeam,
        MenuInput::RefreshLeague,
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
            MenuInput::CreateNewTeam| MenuInput::RefreshLeague=> {
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
