use std::option;
use EditLeagueError::Inquire;
use inquire::{InquireError, Text};

use crate::league::EditLeagueError;





/// We add a journaling aspect to the program by creating a Note type. Users are able to add note to leauges, teams, and players, and eventually series and games.
pub type Note = Option<String>;

pub fn new_note() -> Result<Note, InquireError>{
    let note_text = Text::new("Enter the note you wish to save.")
    .prompt()?;
    Ok(Some(note_text))
}

