use sdl2::pixels::Color;

pub mod palette {
    use super::Color;

    /// Const parse strings of the form "RRGGBB"
    pub const fn from_hex(s: &str) -> Color {
        let [_, r, g, b] = match u32::from_str_radix(s, 16) {
            Ok(n) => n.to_be_bytes(),
            Err(_) => panic!("invalid hex color string"),
        };

        Color::RGB(r, g, b)
    }

    pub const HIDDEN: Color = from_hex("2a2a37"); // #2a2a37

    pub const BLACK: Color = from_hex("1d2021"); // #1d2021
    pub const WHITE: Color = from_hex("f9f5d7"); // #f9f5d7
    pub const IBM_BLACK: Color = from_hex("323c39"); // #323c39
    pub const IBM_WHITE: Color = from_hex("d3c9a1"); // #d3c9a1

    pub const FOREST_BG: Color = from_hex("051200"); // #051200
    pub const EARTH: Color = from_hex("080402"); // #080402 #68423d #4d2c17
    pub const TREE_1: Color = from_hex("154406"); // #154406
    pub const TREE_2: Color = from_hex("063b08"); // #063b08

    pub const FADED_PURPLE: Color = from_hex("9775a6"); // #9775a6

    pub const FIRE_1: Color = from_hex("fc8e26"); // #fc8e26
    pub const FIRE_2: Color = from_hex("ac4427"); // #ac4427

    pub const WATER_1: Color = from_hex("3d515b"); // #3d515b

    pub const GREY_13: Color = from_hex("504945"); // #504945
    pub const GREY_15: Color = from_hex("32302f"); // #32302f
}

impl ColorExt for Color {
    fn rgba(&self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color::RGB(r, g, b)
    }
}

pub trait ColorExt: Sized {
    fn rgba(&self) -> (u8, u8, u8, u8);
    fn from_rgb(r: u8, g: u8, b: u8) -> Self;

    fn blend(&self, other: Self, perc: f32) -> Self {
        let (c1, m1, y1, k1) = self.to_cmyk();
        let (c2, m2, y2, k2) = other.to_cmyk();

        Self::from_cmyk(
            c1 * perc + c2 * (1.0 - perc),
            m1 * perc + m2 * (1.0 - perc),
            y1 * perc + y2 * (1.0 - perc),
            k1 * perc + k2 * (1.0 - perc),
        )
    }

    fn darken(&self, perc: f32) -> Self {
        let (h, s, mut v) = self.to_hsv();
        v *= perc.clamp(0.0, 1.0);

        Self::from_hsv(h, s, v)
    }

    fn desaturate(&self, perc: f32) -> Self {
        let (h, mut s, v) = self.to_hsv();
        s *= perc.clamp(0.0, 1.0);

        Self::from_hsv(h, s, v)
    }

    fn to_cmyk(&self) -> (f32, f32, f32, f32) {
        let (r, g, b, _) = self.rgba();

        let mut c = 1.0 - (r as f32 / 255.0);
        let mut m = 1.0 - (g as f32 / 255.0);
        let mut y = 1.0 - (b as f32 / 255.0);

        let mut k = if c < m { c } else { m };
        k = if k < y { k } else { y };

        c = (c - k) / (1.0 - k);
        m = (m - k) / (1.0 - k);
        y = (y - k) / (1.0 - k);

        (c, m, y, k)
    }

    fn from_cmyk(c: f32, m: f32, y: f32, k: f32) -> Self {
        let mut r = c * (1.0 - k) + k;
        let mut g = m * (1.0 - k) + k;
        let mut b = y * (1.0 - k) + k;

        r = (1.0 - r) * 255.0 + 0.5;
        g = (1.0 - g) * 255.0 + 0.5;
        b = (1.0 - b) * 255.0 + 0.5;

        Self::from_rgb(r as u8, g as u8, b as u8)
    }

    fn to_hsv(&self) -> (f32, f32, f32) {
        let (r, g, b, _) = self.rgba();
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));

        let mut h: f32 = max;
        let v: f32 = max;

        let d = max - min;
        let s = if max == 0.0 { 0.0 } else { d / max };

        if (max - min).abs() < f32::EPSILON {
            h = 0.0; // Achromatic
        } else {
            if (max - r).abs() < f32::EPSILON {
                if g < b {
                    h = (g - b) / d + 6.0;
                } else {
                    h = (g - b) / d;
                }
            } else if (max - g).abs() < f32::EPSILON {
                h = (b - r) / d + 2.0;
            } else if (max - b).abs() < f32::EPSILON {
                h = (r - g) / d + 4.0;
            }

            h /= 6.0;
        }

        (h, s, v)
    }

    fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let i = (h * 6.0).floor() as i32;
        let f = h * 6.0 - i as f32;
        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);

        let (r, g, b) = match i % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            5 => (v, p, q),
            _ => unreachable!(),
        };

        Self::from_rgb((255.0 * r) as u8, (255.0 * g) as u8, (255.0 * b) as u8)
    }
}
