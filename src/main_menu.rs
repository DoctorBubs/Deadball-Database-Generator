use core::fmt;

use inquire::{InquireError, Select};
use rand::rngs::ThreadRng;
use rusqlite::Connection;

use crate::{league::create_new_league, league_check};


#[derive(Copy, Clone,  Debug)]
pub enum MenuInput{
    CreateNewLeague,
    CreateNewTeam,
    RefreshLeague,
    Exit
}


impl fmt::Display for MenuInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        
        let chars = match self{
            Self::CreateNewLeague => "Create a new league.",
            Self::CreateNewTeam => "Create a new team.",
            Self::RefreshLeague => "Refresh an existing league.",
            Self::Exit => "Exit."
        };

        write!(f, "{}", chars)
    }
}


pub fn run_main_menu(conn:&mut Connection, thread: &mut ThreadRng) -> std::io::Result<()> {
  
    
    let mut r_thread = rand::thread_rng();

    

    let starting_options: Vec<MenuInput> = vec![
        MenuInput::CreateNewLeague,
        MenuInput::CreateNewTeam,
        MenuInput::RefreshLeague,
        MenuInput::Exit
    ];

    let starting_choice: Result<MenuInput, InquireError> =
        Select::new("What would you like to do?", starting_options).prompt();

    match starting_choice {
        Ok(choice) => match choice {
            MenuInput::CreateNewLeague => create_new_league(&mut r_thread, conn),
            MenuInput::Exit => Ok(()),
            _ => {
                league_check(conn, &mut r_thread,choice).unwrap();
                Ok(())
            }
        },

        Err(_) => {
            println!("Error matching first choice");
            Ok(())
        }
    }
}