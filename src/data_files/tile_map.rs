use crate::tileset::{Pos, TileSet};
use anyhow::{Context, anyhow};
use sdl2::pixels::Color;
use std::{
    fs,
    iter::Peekable,
    path::Path,
    str::{Lines, SplitWhitespace},
};

/// Parse a tileset from a 16x16 sprite-sheet of IMB code page 437 glyphs.
pub fn parse_ibm437_tileset<'a>(
    path: impl AsRef<Path>,
    d: u16,
    bg: Option<Color>,
) -> anyhow::Result<TileSet<'a>> {
    let mut ts = TileSet::new(path, d, d, Pos::new(0, 0), 0, bg)?;
    let raw = fs::read_to_string("assets/tiles/df/tile.map").context("reading tile.map")?;
    let mut lines = raw.lines().peekable();
    parse_lines(&mut lines, &mut ts)?;

    Ok(ts)
}

/// Parse an arbitrary tilemap using a tile.map file
pub fn parse_tile_map<'a>(path: impl AsRef<Path>) -> anyhow::Result<TileSet<'a>> {
    let raw = fs::read_to_string(path).context("reading tile.map")?;
    let mut lines = raw.lines().peekable();

    let Header {
        path,
        dx,
        dy,
        gap,
        start,
        bg,
    } = try_parse_header(&mut lines).ok_or_else(|| anyhow!("invalid tile.map header"))?;
    let mut ts = TileSet::new(path, dx, dy, start, gap, bg)?;
    parse_lines(&mut lines, &mut ts)?;

    Ok(ts)
}

fn parse_lines(lines: &mut Peekable<Lines<'_>>, ts: &mut TileSet<'_>) -> anyhow::Result<()> {
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

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Header<'a> {
    path: &'a str,
    dx: u16,
    dy: u16,
    gap: u16,
    start: Pos,
    bg: Option<Color>,
}

/// Expected header format is:
///   path assets/urizen/urizen_onebit_tileset__v1d1.png
///   size 12 12
///   gap 1
///   start 1 1
fn try_parse_header<'a>(lines: &mut Peekable<Lines<'a>>) -> Option<Header<'a>> {
    let path = lines.next()?.strip_prefix("path ")?.trim();
    let (dx, dy) = lines.next()?.strip_prefix("size ")?.split_once(' ')?;
    let dx: u16 = dx.parse().ok()?;
    let dy: u16 = dy.parse().ok()?;
    let gap: u16 = lines.next()?.strip_prefix("gap ")?.trim().parse().ok()?;
    let (x, y) = lines.next()?.strip_prefix("start ")?.split_once(' ')?;
    let x: i32 = x.parse().ok()?;
    let y: i32 = y.parse().ok()?;

    let bg = match lines.peek() {
        Some(line) if line.starts_with("bg ") => {
            let (r, gb) = lines.next()?.strip_prefix("bg ")?.split_once(' ')?;
            let (g, b) = gb.split_once(' ')?;

            Some(Color::RGB(
                r.parse().ok()?,
                g.parse().ok()?,
                b.parse().ok()?,
            ))
        }
        _ => None,
    };

    Some(Header {
        path,
        dx,
        dy,
        gap,
        start: Pos::new(x, y),
        bg,
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
        let opt = try_parse_header(&mut raw.lines().peekable());
        let expected = Header {
            path: "assets/urizen/urizen_onebit_tileset__v1d1.png",
            dx: 12,
            dy: 12,
            gap: 1,
            start: Pos::new(1, 1),
            bg: Some(Color::BLACK),
        };

        assert_eq!(opt, Some(expected));
    }
}
