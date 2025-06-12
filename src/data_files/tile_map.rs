use crate::tileset::{Pos, TileSet};
use anyhow::{Context, anyhow};
use std::{
    fs,
    path::Path,
    str::{Lines, SplitWhitespace},
};

pub fn parse_tile_map<'a>(path: impl AsRef<Path>) -> anyhow::Result<TileSet<'a>> {
    let raw = fs::read_to_string(path).context("reading tile.map")?;
    let mut lines = raw.lines();

    let Header {
        path,
        dx,
        dy,
        gap,
        start,
    } = try_parse_header(&mut lines).ok_or_else(|| anyhow!("invalid tile.map header"))?;
    let mut ts = TileSet::new(path, dx, dy, start, gap)?;

    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut words = line.split_whitespace();
        let (row, mut col) =
            try_parse_line_pos(&mut words).ok_or_else(|| anyhow!("invalid line: {line:?}"))?;

        for ident in words {
            ts.map_tile(ident, row, col);
            col += 1;
        }
    }

    Ok(ts)
}

#[derive(Debug, PartialEq, Eq)]
struct Header<'a> {
    path: &'a str,
    dx: u16,
    dy: u16,
    gap: u16,
    start: Pos,
}

/// Expected header format is:
///   path assets/urizen/urizen_onebit_tileset__v1d1.png
///   size 12 12
///   gap 1
///   start 1 1
fn try_parse_header<'a>(lines: &mut Lines<'a>) -> Option<Header<'a>> {
    let path = lines.next()?.strip_prefix("path ")?.trim();
    let (dx, dy) = lines.next()?.strip_prefix("size ")?.split_once(' ')?;
    let dx: u16 = dx.parse().ok()?;
    let dy: u16 = dy.parse().ok()?;
    let gap: u16 = lines.next()?.strip_prefix("gap ")?.trim().parse().ok()?;
    let (x, y) = lines.next()?.strip_prefix("start ")?.split_once(' ')?;
    let x: i32 = x.parse().ok()?;
    let y: i32 = y.parse().ok()?;

    Some(Header {
        path,
        dx,
        dy,
        gap,
        start: Pos::new(x, y),
    })
}

/// expect lines to be "$row $col $(ident)+"
fn try_parse_line_pos(words: &mut SplitWhitespace<'_>) -> Option<(u16, u16)> {
    let row: u16 = words.next()?.parse().ok()?;
    let col: u16 = words.next()?.parse().ok()?;

    Some((row, col))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_parse_header_works() {
        let raw = include_str!("../../assets/urizen/tile.map");
        let opt = try_parse_header(&mut raw.lines());
        let expected = Header {
            path: "assets/urizen/urizen_onebit_tileset__v1d1.png",
            dx: 12,
            dy: 12,
            gap: 1,
            start: Pos::new(1, 1),
        };

        assert_eq!(opt, Some(expected));
    }
}
