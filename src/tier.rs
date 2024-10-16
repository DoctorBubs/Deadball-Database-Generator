use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    F(i32),
    D(i32),
    C(i32),
    B(i32),
    A(i32),
    S(i32),
}

impl Tier {
    fn get_num(&self) -> i32 {
        match self {
            Self::F(num)
            | Self::D(num)
            | Self::C(num)
            | Self::B(num)
            | Self::A(num)
            | Self::S(num) => *num,
        }
    }
    pub fn add(&self, new_num: i32) -> Self {
        let sum = self.get_num() + new_num;
        match self {
            Self::F(_) => Self::F(sum),
            Self::D(_) => Self::D(sum),
            Self::C(_) => Self::C(sum),
            Self::B(_) => Self::B(sum),
            Self::A(_) => Self::A(sum),
            Self::S(_) => Self::S(sum),
        }
    }

    pub fn get_letter_num(&self) -> (&str, i32) {
        match self {
            Self::F(num) => ("F", *num),
            Self::D(num) => ("D", *num),
            Self::C(num) => ("C", *num),
            Self::B(num) => ("B", *num),
            Self::A(num) => ("A", *num),
            Self::S(num) => ("S", *num),
        }
    }
}

impl Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // First, we get the letter grade and number from the enum;
        let (letter, num) = self.get_letter_num();
        let (sign_letter, sign_range) = match num {
            ..=-1 => (Some("-"), Some(num..0)),
            0 => (None, None),
            _ => (Some("+"), Some(0..num)),
        };
        let text = match (sign_letter, sign_range) {
            (Some(sign_char), Some(range)) => {
                let signs: String = range.map(|_x| sign_char).collect();
                format!("{}{}", letter, signs)
            }
            _ => letter.to_string(),
        };
        /*let signs: String = match num{
            ..=-1 => (num..0).map(|x|"-").collect(),
            0 => String::from(""),
            _ => (0..num).map(|x| "+").collect()
        };*/
        write!(f, "{}", text)
    }
}
