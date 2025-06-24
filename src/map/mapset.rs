use crate::map::Map;
use std::cmp::min;

#[derive(Debug, Default, Clone)]
pub struct MapSet {
    current: usize,
    maps: Vec<Map>,
}

impl MapSet {
    pub fn new() -> Self {
        Self {
            current: 0,
            maps: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.maps.is_empty()
    }

    pub fn current(&self) -> &Map {
        &self.maps[self.current]
    }

    pub fn current_mut(&mut self) -> &mut Map {
        &mut self.maps[self.current]
    }

    pub fn push(&mut self, map: Map) {
        self.maps.push(map);
    }

    pub fn next(&mut self) {
        self.current = min(self.current + 1, self.maps.len() - 1);
    }

    pub fn prev(&mut self) {
        self.current = self.current.saturating_sub(1);
    }
}
