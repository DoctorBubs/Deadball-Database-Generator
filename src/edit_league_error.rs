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
}
