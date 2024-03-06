

enum MainMenuOptions{

    CreateNewLeague,
    CreateNewTeam


}



fn main_menu(thread: &mut ThreadRng) -> std::io::Result<()>{

    let starting_options: Vec<&str> = vec![
        "Create a new league.",
        "Add a new team to an existing league.",
    ];

    let starting_choice: Result<&str, InquireError> =
        Select::new("What would you like to do?", starting_options).prompt();

    match starting_choice {
        Ok(choice) => match choice {
            "Create a new league." => create_new_league(&mut r_thread),
            "Add a new team to an existing league." => league_check(&mut r_thread),
            _ => {
                println!("Error with starting choice");
                Ok(())
            }
        },

        Err(_) => {
            println!("Error matching first choice");
            Ok(())
        }
    }





}