use crate::{
    map::{Cell, Grid},
    tileset::TileSet,
};
use anyhow::{Context, anyhow, bail};
use std::{collections::HashMap, fs, path::Path};

pub fn parse_ibm437_prefab(path: impl AsRef<Path>, ts: &TileSet<'_>) -> anyhow::Result<Grid> {
    let raw = fs::read_to_string(path).context("reading prefab")?;
    let mut lines = raw.lines().peekable();
    let mut grid = Grid::default();

    let mut defs = HashMap::new();
    defs.insert(' ', ts.tile_index(" ").unwrap());

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
        let idx = ts
            .tile_index(ident)
            .ok_or_else(|| anyhow!("invalid tile ident: {ident}"))?;

        if char.len() != 1 {
            bail!("invalid def {char:?}: expected a single char");
        }

        defs.insert(char.chars().next().unwrap(), idx);
    }

    // determine line length for the prefab
    grid.w = lines
        .peek()
        .ok_or_else(|| anyhow!("invalid prefab: no map provided"))?
        .len();

    // parse the prefab into cells
    for line in lines {
        for ch in line.chars() {
            let idx = defs.get(&ch).ok_or_else(|| anyhow!("no def for {ch:?}"))?;
            grid.cells.push(Cell::new(*idx));
        }
    }

    Ok(grid)
}
