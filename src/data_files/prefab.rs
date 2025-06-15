use crate::{
    grid::{Grid, Tile},
    tileset::TileSet,
};
use anyhow::{Context, anyhow, bail};
use sdl2::pixels::Color;
use std::{collections::HashMap, fs, path::Path};

pub fn parse_cp437_prefab(
    path: impl AsRef<Path>,
    ts: &TileSet<'_>,
    palette: &HashMap<String, Color>,
) -> anyhow::Result<Grid> {
    let raw = fs::read_to_string(path).context("reading prefab")?;
    let mut lines = raw.lines().peekable();
    let mut grid = Grid::default();

    let mut defs = HashMap::new();
    defs.insert(' ', Tile::new(ts.tile_index(" ").unwrap()));

    // parse defs
    loop {
        let line = match lines.next() {
            None => bail!("invalid prefab: no map provided"),
            Some("") => break,
            Some(line) => line,
        };
        let (ch, color, ident) =
            parse_tile_def(line, palette).ok_or_else(|| anyhow!("invalid tile def: {line:?}"))?;
        let idx = ts
            .tile_index(ident)
            .ok_or_else(|| anyhow!("unknown tile ident: {ident}"))?;

        defs.insert(ch, Tile::new_with_color(idx, color));
    }

    // determine line length for the prefab
    grid.w = lines
        .peek()
        .ok_or_else(|| anyhow!("invalid prefab: no map provided"))?
        .len();

    // parse the prefab into tiles
    for line in lines {
        for ch in line.chars() {
            let tile = defs.get(&ch).ok_or_else(|| anyhow!("no def for {ch:?}"))?;
            grid.tiles.push(*tile);
        }
    }

    Ok(grid)
}

fn parse_tile_def<'a>(
    line: &'a str,
    colors: &HashMap<String, Color>,
) -> Option<(char, Color, &'a str)> {
    let (char, tail) = line.split_once(' ')?;
    let (color, ident) = tail.split_once(' ')?;

    let mut chars = char.chars();
    let ch = chars.next()?;
    if chars.next().is_some() {
        return None;
    }

    let color = *colors.get(color.trim())?;

    Some((ch, color, ident.trim()))
}
