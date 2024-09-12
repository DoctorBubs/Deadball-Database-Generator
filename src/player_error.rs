use core::fmt;
use std::fmt::Display;

#[derive(Debug, Copy, Clone)]
pub struct CompTable<T: Eq + Display> {
    pub expected: T,
    pub actual: T,
}

impl<T: Eq + Display> fmt::Display for CompTable<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expected: {}, Actual:{}", self.expected, self.actual)
    }
}

fn option_string<'a, T: Display>(opt: Option<T>, field_name: &str) -> Option<String> {
    if let Some(value) = opt {
        Some(format!("\n{} is invalid for {}", value, field_name))
    } else {
        None
    }
}
type CompOpt<T> = Option<CompTable<T>>;

fn add_comp_table_string<T: Eq + Display>(opt: CompOpt<T>, field_name: &str) -> Option<String> {
    if let Some(table) = opt {
        Some(format!("\n{} is invalid.{}", field_name, table))
    } else {
        None
    }
}

#[derive(Debug)]
/// Contains Options that corresponds if a Player has a value that is incorrect. This is used to generate warnings when loading a player from SQl, in case the user has edited a player with incorrect values.
pub struct PlayerError<'a> {
    pub valid_age: Option<i32>,
    pub valid_bt: Option<i32>,
    pub valid_obt_mod: Option<i32>,
    pub valid_obt: Option<i32>,
    pub valid_obt_sum: CompOpt<i32>,
    pub valid_pd_int: CompOpt<i32>,
    pub name: &'a String,
    pub id: i64,
}
impl fmt::Display for PlayerError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = format!("Warning: {} has errors in the database[", self.name);
        let addons = [
            option_string(self.valid_age, "age"),
            option_string(self.valid_bt, "bt"),
            option_string(self.valid_obt_mod, "obt_mod"),
            option_string(self.valid_obt, "obt"),
            add_comp_table_string(self.valid_obt_sum, "obt"),
            add_comp_table_string(self.valid_pd_int, "pd_int"),
        ];

        for addon in addons {
            if let Some(new_string) = addon {
                result = format! {"{}{}",result,new_string}
            }
        }

        result = format!("{}\n]\n Player ID = {}", result, self.id);
        write!(f, "{}", result)
    }
}
