use crate::tileset::{Pos, TileSet};
use anyhow::{Context, anyhow, bail};
use std::{fs, path::Path};

#[derive(Default, Debug)]
pub struct Prefab {
    pub cells: Vec<usize>,
    pub tiles: Vec<Pos>,
    pub w: usize,
}

impl Prefab {
    pub fn parse_ibm437(path: impl AsRef<Path>, ts: &TileSet<'_>) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path).context("reading prefab")?;
        let mut lines = raw.lines().peekable();
        let mut p = Prefab::default();

        let mut defs = Vec::new();
        defs.push(' ');
        p.tiles.push(ts.tile(" ").unwrap());

        // parse defs
        loop {
            let line = match lines.next() {
                None => bail!("invalid prefab: no map provided"),
                Some("") => break,
                Some(line) => line,
            };
            let (char, ident) = line
                .split_once(' ')
                .ok_or_else(|| anyhow!("invalid def: {line:?}"))?;
            let pos = ts
                .tile(ident)
                .ok_or_else(|| anyhow!("invalid tile ident: {ident}"))?;

            if char.len() != 1 {
                bail!("invalid def {char:?}: expected a single char");
            }

            defs.push(char.chars().next().unwrap());
            p.tiles.push(pos);
        }

        // determine line length for the prefab
        p.w = lines
            .peek()
            .ok_or_else(|| anyhow!("invalid prefab: no map provided"))?
            .len();

        // parse the prefab into cells
        for line in lines {
            for ch in line.chars() {
                let idx = defs
                    .iter()
                    .position(|def| *def == ch)
                    .ok_or_else(|| anyhow!("no def for {ch:?}"))?;
                p.cells.push(idx);
            }
        }

        Ok(p)
    }
}
