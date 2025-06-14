use anyhow::{Context, anyhow, bail};
use sdl2::pixels::Color;
use std::{collections::HashMap, fs};

pub fn parse_color_palette() -> anyhow::Result<HashMap<String, Color>> {
    let raw = fs::read_to_string("data/color.palette").context("reading colour.palette")?;
    let mut palette = HashMap::new();

    for line in raw.lines() {
        if line.is_empty() {
            continue;
        }

        let (hex, ident) = line
            .split_once(' ')
            .ok_or_else(|| anyhow!("invalid color def: {line:?}"))?;
        let hex = hex
            .strip_prefix('#')
            .ok_or_else(|| anyhow!("invalid colour hex: {hex:?}"))?;

        let [_, r, g, b] = match u32::from_str_radix(hex, 16) {
            Ok(n) => n.to_be_bytes(),
            Err(e) => bail!("invalid color {hex:?}: {e}"),
        };

        palette.insert(ident.to_string(), Color::RGB(r, g, b));
    }

    if palette.is_empty() {
        bail!("no colours defined")
    }

    Ok(palette)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_works() {
        assert!(parse_color_palette().is_ok());
    }
}
