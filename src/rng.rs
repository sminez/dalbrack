use rand::{Rng, rng};

pub fn roll(sides: u16) -> u16 {
    rng().random_range(1..=sides)
}

pub fn roll_many(dice: &[u16]) -> u16 {
    let mut r = rng();

    dice.iter().map(|sides| r.random_range(1..=*sides)).sum()
}

pub fn percentile() -> u16 {
    rng().random_range(1..=100)
}
