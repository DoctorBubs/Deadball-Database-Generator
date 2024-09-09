![akers_logo](https://github.com/user-attachments/assets/f1ae08f7-4324-4923-9e41-dabdda777994)

# Summary
Based off the [Deadball tabletop baseball game by W.M. Akers](http://wmakers.net/deadball), this Rust program creats a SQLite database on the users machine for information that is usefull if the user were running their own Deadball League. The program also provides an interface for the users to automatically generate leagues and teams based off the user's choice of options such as league era and gender.
The program also creates a folder for each league, and in each league folder a plain text file is created. If a user updates a team in the database and wishes to see the changes in the text files, the program will also automate that via the "Refresh an existing league" option from the main menu.

# Scheduling and Issues

The plan is to enable schedule generation in the program as well, however this has turned out to be tricky to solve, so the option to generate a schedule is currently commented out untill a solution is implemented.  The program will also not work if it is run in a folder that does not allow writing inside.
