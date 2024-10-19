use inquire::InquireError;

#[derive(Debug)]
//Possible Errors that could arise from editing a league.
pub enum EditLeagueError {
    /// Error when there is a team with the same abbreviation in the league.
    AbrvTaken,
    /// Error when there is a team with the same name in the league.
    NameTaken,
    /// Error accessing the database.
    DatabaseError(rusqlite::Error),
    ///Error Serializing
    SerdeError(serde_json::Error),
    ///Error using Inquire
    Inquire(InquireError),
    PennantError(String),
}

/// Takes a result that could produce a rusqlite error. If it is an error, it is converted into a EditLeagueError.
pub fn handle_sql_error<T>(input: Result<T, rusqlite::Error>) -> Result<T, EditLeagueError> {
    match input {
        Err(message) => Err(EditLeagueError::DatabaseError(message)),
        Ok(value) => Ok(value),
    }
}

/// Takes a result that could produce a serde_json error. If it is an error, it is converted into a EditLeagueError.
pub fn handle_serde_error<T>(input: Result<T, serde_json::Error>) -> Result<T, EditLeagueError> {
    match input {
        Err(message) => Err(EditLeagueError::SerdeError(message)),
        Ok(value) => Ok(value),
    }
}

pub fn handle_inquire_error<T>(
    input: Result<T, inquire::InquireError>,
) -> Result<T, EditLeagueError> {
    match input {
        Err(message) => Err(EditLeagueError::Inquire(message)),
        Ok(value) => Ok(value),
    }
}
