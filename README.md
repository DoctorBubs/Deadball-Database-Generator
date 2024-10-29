
# Summary
Based off the [Deadball tabletop baseball game by W.M. Akers](http://wmakers.net/deadball), this Rust program creats a SQLite database on the users machine for information that is usefull if the user were running their own Deadball League. The program also provides an interface for the users to automatically generate leagues and teams based off the user's choice of options such as league era and gender. Once a league is created, the program can also add new teams to the league in the database.
The program also creates a folder for each league, and in each league folder a plain text file for each team is created. If a user updates a team or play in the database(E.G Updating a player's batting) and wishes to see the changes in the text files, the program will also automate that via the "Refresh an existing league" option from the main menu.
The program can also query the database to view the top 10 batters or pitchers in a league. Doing so will also display information regarding averages for the league. The program will display the top 10 player ranked by OBT for batters or PD for pitchers, and will also give the player a letter grade from S - F based off a tier list system. However, the letter grading system is still a WIP.
The program alo can generate standings to be used in a Nine Game Pennant. To to do, you must enter in how many games should have already been played when the campaign should start, and the program will generate standings that will be written to a text file. However, it is possible that this will fail if there are too few teams or games for the program to calculate.
When loading a player from the database, the program will check to see if the players pitch die and hand batting/pitching hand is correct. If not, the program will give you a prompt that will guide you through the process of selecting a correct value, however this check currently does not run when viewing the leaderboards for a league,

# Installation and Use

To run the program, you must have Rust installed on your machine. If you are using Windows, try using the [Rustup tool.](https://www.rust-lang.org/learn/get-started).
Next,clone the repo and save the output to a folder of your choice, open the folder via your command line, and run the program via entering "cargo run" in the command line.

# Scheduling and Issues

The plan is to enable schedule generation in the program as well, however this has turned out to be tricky to solve, so the option to generate a schedule is currently commented out until a solution is implemented.  The program will also not work if it is run in a folder that does not allow writing inside.

If you would like to generate a Deadball like to an Excel workbook, please [consider my other repo.](https://github.com/DoctorBubs/Deadball_WorkBook_Generator/tree/main)
