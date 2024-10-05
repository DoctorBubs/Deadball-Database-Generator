use inquire::{InquireError, Text};

/// We add a journaling aspect to the program by creating a Note type. Users are able to add note to leauges, teams, and players, and eventually series and games.
pub type Note = Option<String>;

pub fn new_note(input_str: String) -> Result<Note, InquireError> {
    let note_text = Text::new(input_str.as_str()).prompt()?;
    Ok(Some(note_text))
}

/// Trait for ostructs that contain notes.
pub trait Notable {
    fn get_note(&self) -> &Note;

    fn get_note_input_string(&self) -> String;

    fn generate_note(&self) -> Result<Note, InquireError> {
        let input_str = self.get_note_input_string();
        new_note(input_str)
    }
}
