use rand::{Rng as _, rngs::ThreadRng};
use sdl2::rect::Rect;
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct RngHandle {
    rng: ThreadRng,
}

impl RngHandle {
    pub fn new() -> Self {
        Self { rng: rand::rng() }
    }

    pub fn roll(&mut self, sides: u16) -> u16 {
        self.rng.random_range(1..=sides)
    }

    pub fn roll_many(&mut self, dice: &[u16]) -> u16 {
        dice.iter()
            .map(|sides| self.rng.random_range(1..=*sides))
            .sum()
    }

    pub fn percentile(&mut self) -> u16 {
        self.rng.random_range(1..=100)
    }

    pub fn random_point(&mut self, r: Rect, offset: i32) -> (i32, i32) {
        let rx = (r.x + offset)..(r.x + r.w - offset);
        let ry = (r.y + offset)..(r.y + r.h - offset);

        let x = if rx.is_empty() {
            r.x
        } else {
            self.rng.random_range(rx)
        };
        let y = if ry.is_empty() {
            r.y
        } else {
            self.rng.random_range(ry)
        };

        (x, y)
    }
}

impl Deref for RngHandle {
    type Target = ThreadRng;

    fn deref(&self) -> &Self::Target {
        &self.rng
    }
}

impl DerefMut for RngHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rng
    }
}
