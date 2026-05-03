//! Dominant-color extraction and color-name lookup for the optional
//! "Color search (beta)" feature.
//!
//! All work is local — no extra crates beyond `image` (already a dep) and
//! `anyhow`. Colors are stored / serialised as 6-char lowercase hex.

use anyhow::{Context, Result};
use std::path::Path;

/// Resize the image to a small swatch, ignore near-black and near-white pixels
/// (they bias every photograph toward gray), bucket the rest into a 4-bit-per-
/// channel RGB cube, and return the centre of the heaviest bucket.
pub fn analyze(path: &Path) -> Result<[u8; 3]> {
    let img = image::open(path)
        .with_context(|| format!("decode for color analysis: {}", path.display()))?;
    let small = img.resize_exact(96, 96, image::imageops::FilterType::Triangle);
    let rgb = small.to_rgb8();

    // Sum (r, g, b, weight) per 4-bit-per-channel bucket. Saturated pixels
    // weigh more than gray ones so a photo dominated by neutral tones doesn't
    // collapse to mud.
    let mut sums = vec![(0u64, 0u64, 0u64, 0u64); 4096];
    for px in rgb.pixels() {
        let r = px[0] as f32 / 255.0;
        let g = px[1] as f32 / 255.0;
        let b = px[2] as f32 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        if !(0.06..=0.96).contains(&luma) {
            continue;
        }
        let saturation = if max == 0.0 { 0.0 } else { (max - min) / max };
        // Weight: saturated pixels count up to 5x; pure-gray pixels count 1x.
        let w = (1.0 + saturation * 4.0) as u64;

        let idx =
            ((px[0] as usize >> 4) << 8) | ((px[1] as usize >> 4) << 4) | (px[2] as usize >> 4);
        let s = &mut sums[idx];
        s.0 += (px[0] as u64) * w;
        s.1 += (px[1] as u64) * w;
        s.2 += (px[2] as u64) * w;
        s.3 += w;
    }

    let (_, best) = sums
        .iter()
        .enumerate()
        .max_by_key(|(_, s)| s.3)
        .filter(|(_, s)| s.3 > 0)
        .ok_or_else(|| anyhow::anyhow!("no usable pixels in image"))?;

    // True mean of the heaviest bucket — no quantization-step bias.
    let r = (best.0 / best.3) as u8;
    let g = (best.1 / best.3) as u8;
    let b = (best.2 / best.3) as u8;
    Ok([r, g, b])
}

pub fn to_hex(rgb: [u8; 3]) -> String {
    format!("{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])
}

/// Accepts `#aabbcc`, `aabbcc`, `#abc`, `abc`. Case-insensitive.
pub fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let s = s.trim().trim_start_matches('#');
    let (r, g, b) = match s.len() {
        6 => (
            u8::from_str_radix(&s[0..2], 16).ok()?,
            u8::from_str_radix(&s[2..4], 16).ok()?,
            u8::from_str_radix(&s[4..6], 16).ok()?,
        ),
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()?;
            let g = u8::from_str_radix(&s[1..2], 16).ok()?;
            let b = u8::from_str_radix(&s[2..3], 16).ok()?;
            (r * 17, g * 17, b * 17)
        }
        _ => return None,
    };
    Some([r, g, b])
}

/// HSV-aware perceptual distance. Hue dominates when both colors are
/// saturated; falls back to luma + saturation distance for grays. Lower =
/// closer.
pub fn distance(a: [u8; 3], b: [u8; 3]) -> u32 {
    let (ah, as_, av) = rgb_to_hsv(a);
    let (bh, bs, bv) = rgb_to_hsv(b);
    let raw = (ah - bh).abs();
    let dh = raw.min(360.0 - raw); // shortest way around the hue circle
    let ds = (as_ - bs).abs();
    let dv = (av - bv).abs();
    // Effective hue weight: 0 when either is unsaturated (hue is meaningless),
    // up to ~3.5 when both are vivid.
    let hue_w = as_.min(bs) * 3.5;
    let hue_term = (dh / 180.0) * hue_w; // dh / 180 ∈ [0, 1]
    let total = hue_term * hue_term + ds * ds * 0.6 + dv * dv * 0.25;
    (total * 10_000.0) as u32
}

fn rgb_to_hsv(rgb: [u8; 3]) -> (f32, f32, f32) {
    let r = rgb[0] as f32 / 255.0;
    let g = rgb[1] as f32 / 255.0;
    let b = rgb[2] as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let v = max;
    let s = if max == 0.0 { 0.0 } else { (max - min) / max };
    let h = if max == min {
        0.0
    } else if max == r {
        60.0 * (((g - b) / (max - min)) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / (max - min)) + 2.0)
    } else {
        60.0 * (((r - g) / (max - min)) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, v)
}

/// Case-insensitive lookup against an inline table of CSS color keywords plus
/// a few common aliases. Returns `None` for unknown words.
pub fn name_to_rgb(name: &str) -> Option<[u8; 3]> {
    let trimmed = name.trim().to_ascii_lowercase();
    NAMES
        .iter()
        .find(|(n, _)| *n == trimmed)
        .map(|(_, rgb)| *rgb)
}

// CSS color keywords + a handful of aliases. Inline so the binary carries a
// stable mapping with no startup cost. Sorted alphabetically for readability.
#[rustfmt::skip]
static NAMES: &[(&str, [u8; 3])] = &[
    ("aliceblue", [240, 248, 255]),
    ("antiquewhite", [250, 235, 215]),
    ("aqua", [0, 255, 255]),
    ("aquamarine", [127, 255, 212]),
    ("azure", [240, 255, 255]),
    ("beige", [245, 245, 220]),
    ("bisque", [255, 228, 196]),
    ("black", [0, 0, 0]),
    ("blanchedalmond", [255, 235, 205]),
    ("blue", [0, 0, 255]),
    ("blueviolet", [138, 43, 226]),
    ("brown", [165, 42, 42]),
    ("burlywood", [222, 184, 135]),
    ("cadetblue", [95, 158, 160]),
    ("chartreuse", [127, 255, 0]),
    ("chocolate", [210, 105, 30]),
    ("coral", [255, 127, 80]),
    ("cornflowerblue", [100, 149, 237]),
    ("cornsilk", [255, 248, 220]),
    ("crimson", [220, 20, 60]),
    ("cyan", [0, 255, 255]),
    ("darkblue", [0, 0, 139]),
    ("darkcyan", [0, 139, 139]),
    ("darkgoldenrod", [184, 134, 11]),
    ("darkgray", [169, 169, 169]),
    ("darkgreen", [0, 100, 0]),
    ("darkgrey", [169, 169, 169]),
    ("darkkhaki", [189, 183, 107]),
    ("darkmagenta", [139, 0, 139]),
    ("darkolivegreen", [85, 107, 47]),
    ("darkorange", [255, 140, 0]),
    ("darkorchid", [153, 50, 204]),
    ("darkred", [139, 0, 0]),
    ("darksalmon", [233, 150, 122]),
    ("darkseagreen", [143, 188, 143]),
    ("darkslateblue", [72, 61, 139]),
    ("darkslategray", [47, 79, 79]),
    ("darkslategrey", [47, 79, 79]),
    ("darkturquoise", [0, 206, 209]),
    ("darkviolet", [148, 0, 211]),
    ("deeppink", [255, 20, 147]),
    ("deepskyblue", [0, 191, 255]),
    ("dimgray", [105, 105, 105]),
    ("dimgrey", [105, 105, 105]),
    ("dodgerblue", [30, 144, 255]),
    ("firebrick", [178, 34, 34]),
    ("floralwhite", [255, 250, 240]),
    ("forestgreen", [34, 139, 34]),
    ("fuchsia", [255, 0, 255]),
    ("gainsboro", [220, 220, 220]),
    ("ghostwhite", [248, 248, 255]),
    ("gold", [255, 215, 0]),
    ("goldenrod", [218, 165, 32]),
    ("gray", [128, 128, 128]),
    ("green", [0, 128, 0]),
    ("greenyellow", [173, 255, 47]),
    ("grey", [128, 128, 128]),
    ("honeydew", [240, 255, 240]),
    ("hotpink", [255, 105, 180]),
    ("indianred", [205, 92, 92]),
    ("indigo", [75, 0, 130]),
    ("ivory", [255, 255, 240]),
    ("khaki", [240, 230, 140]),
    ("lavender", [230, 230, 250]),
    ("lavenderblush", [255, 240, 245]),
    ("lawngreen", [124, 252, 0]),
    ("lemonchiffon", [255, 250, 205]),
    ("lightblue", [173, 216, 230]),
    ("lightcoral", [240, 128, 128]),
    ("lightcyan", [224, 255, 255]),
    ("lightgoldenrodyellow", [250, 250, 210]),
    ("lightgray", [211, 211, 211]),
    ("lightgreen", [144, 238, 144]),
    ("lightgrey", [211, 211, 211]),
    ("lightpink", [255, 182, 193]),
    ("lightsalmon", [255, 160, 122]),
    ("lightseagreen", [32, 178, 170]),
    ("lightskyblue", [135, 206, 250]),
    ("lightslategray", [119, 136, 153]),
    ("lightslategrey", [119, 136, 153]),
    ("lightsteelblue", [176, 196, 222]),
    ("lightyellow", [255, 255, 224]),
    ("lime", [0, 255, 0]),
    ("limegreen", [50, 205, 50]),
    ("linen", [250, 240, 230]),
    ("magenta", [255, 0, 255]),
    ("maroon", [128, 0, 0]),
    ("mauve", [203, 166, 247]),
    ("mediumaquamarine", [102, 205, 170]),
    ("mediumblue", [0, 0, 205]),
    ("mediumorchid", [186, 85, 211]),
    ("mediumpurple", [147, 112, 219]),
    ("mediumseagreen", [60, 179, 113]),
    ("mediumslateblue", [123, 104, 238]),
    ("mediumspringgreen", [0, 250, 154]),
    ("mediumturquoise", [72, 209, 204]),
    ("mediumvioletred", [199, 21, 133]),
    ("midnightblue", [25, 25, 112]),
    ("mintcream", [245, 255, 250]),
    ("mistyrose", [255, 228, 225]),
    ("moccasin", [255, 228, 181]),
    ("navajowhite", [255, 222, 173]),
    ("navy", [0, 0, 128]),
    ("oldlace", [253, 245, 230]),
    ("olive", [128, 128, 0]),
    ("olivedrab", [107, 142, 35]),
    ("orange", [255, 165, 0]),
    ("orangered", [255, 69, 0]),
    ("orchid", [218, 112, 214]),
    ("palegoldenrod", [238, 232, 170]),
    ("palegreen", [152, 251, 152]),
    ("paleturquoise", [175, 238, 238]),
    ("palevioletred", [219, 112, 147]),
    ("papayawhip", [255, 239, 213]),
    ("peachpuff", [255, 218, 185]),
    ("peru", [205, 133, 63]),
    ("pink", [255, 192, 203]),
    ("plum", [221, 160, 221]),
    ("powderblue", [176, 224, 230]),
    ("purple", [128, 0, 128]),
    ("rebeccapurple", [102, 51, 153]),
    ("red", [255, 0, 0]),
    ("rosybrown", [188, 143, 143]),
    ("royalblue", [65, 105, 225]),
    ("saddlebrown", [139, 69, 19]),
    ("salmon", [250, 128, 114]),
    ("sandybrown", [244, 164, 96]),
    ("seagreen", [46, 139, 87]),
    ("seashell", [255, 245, 238]),
    ("sienna", [160, 82, 45]),
    ("silver", [192, 192, 192]),
    ("skyblue", [135, 206, 235]),
    ("slateblue", [106, 90, 205]),
    ("slategray", [112, 128, 144]),
    ("slategrey", [112, 128, 144]),
    ("snow", [255, 250, 250]),
    ("springgreen", [0, 255, 127]),
    ("steelblue", [70, 130, 180]),
    ("tan", [210, 180, 140]),
    ("teal", [0, 128, 128]),
    ("thistle", [216, 191, 216]),
    ("tomato", [255, 99, 71]),
    ("turquoise", [64, 224, 208]),
    ("violet", [238, 130, 238]),
    ("wheat", [245, 222, 179]),
    ("white", [255, 255, 255]),
    ("whitesmoke", [245, 245, 245]),
    ("yellow", [255, 255, 0]),
    ("yellowgreen", [154, 205, 50]),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_lookup_known_keywords() {
        assert_eq!(name_to_rgb("blue"), Some([0, 0, 255]));
        assert_eq!(name_to_rgb(" Red "), Some([255, 0, 0]));
        assert_eq!(name_to_rgb("MAUVE"), Some([203, 166, 247]));
        assert_eq!(name_to_rgb("grey"), Some([128, 128, 128]));
    }

    #[test]
    fn name_lookup_unknown() {
        assert_eq!(name_to_rgb("not-a-color"), None);
        assert_eq!(name_to_rgb(""), None);
    }

    #[test]
    fn hex_round_trip() {
        assert_eq!(parse_hex("#cba6f7"), Some([0xcb, 0xa6, 0xf7]));
        assert_eq!(parse_hex("cba6f7"), Some([0xcb, 0xa6, 0xf7]));
        assert_eq!(parse_hex("#abc"), Some([0xaa, 0xbb, 0xcc]));
        assert_eq!(parse_hex("abc"), Some([0xaa, 0xbb, 0xcc]));
        assert_eq!(parse_hex("nope"), None);
        assert_eq!(to_hex([0xcb, 0xa6, 0xf7]), "cba6f7");
    }

    #[test]
    fn distance_self_zero() {
        assert_eq!(distance([100, 50, 200], [100, 50, 200]), 0);
        assert!(distance([0, 0, 0], [255, 255, 255]) > 0);
    }

    #[test]
    fn analyze_solid_color_image() {
        use image::{ImageBuffer, Rgb};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("blue.png");
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(8, 8, Rgb([30, 60, 230]));
        img.save(&path).unwrap();
        let rgb = analyze(&path).unwrap();
        // Bucketed result lands near input.
        assert!(distance(rgb, [30, 60, 230]) < 5_000);
    }
}
