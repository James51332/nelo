use std::collections::HashMap;

use ab_glyph::FontArc;

use crate::scene::Scene;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum Font {
    #[default]
    CmuSerifRoman,
    AMSRegular,
    CaligraphicBold,
    CaligraphicRegular,
    FrakturBold,
    FrakturRegular,
    MainBold,
    MainBoldItalic,
    MainItalic,
    MainRegular,
    MathBoldItalic,
    MathItalic,
    SansSerifBold,
    SansSerifItalic,
    SansSerifRegular,
    ScriptRegular,
    Size1Regular,
    Size2Regular,
    Size3Regular,
    Size4Regular,
    TypewriterRegular,
}

impl Font {
    pub fn iter() -> &'static [Self] {
        &[
            Self::CmuSerifRoman,
            Self::AMSRegular,
            Self::CaligraphicBold,
            Self::CaligraphicRegular,
            Self::FrakturBold,
            Self::FrakturRegular,
            Self::MainBold,
            Self::MainBoldItalic,
            Self::MainItalic,
            Self::MainRegular,
            Self::MathBoldItalic,
            Self::MathItalic,
            Self::SansSerifBold,
            Self::SansSerifItalic,
            Self::SansSerifRegular,
            Self::ScriptRegular,
            Self::Size1Regular,
            Self::Size2Regular,
            Self::Size3Regular,
            Self::Size4Regular,
            Self::TypewriterRegular,
        ]
    }

    pub fn map() -> HashMap<Font, FontArc> {
        let mut map = HashMap::new();
        for &font in Self::iter() {
            match FontArc::try_from_slice(font.bytes()) {
                Ok(arc) => {
                    map.insert(font, arc);
                }
                Err(e) => log::warn!("Failed to load font: {e}"),
            };
        }
        map
    }

    pub fn bytes(&self) -> &'static [u8] {
        match self {
            Self::CmuSerifRoman => include_bytes!("fonts/cmu.serif-roman.ttf"),
            Self::AMSRegular => include_bytes!("fonts/KaTeX_AMS-Regular.ttf"),
            Self::CaligraphicBold => include_bytes!("fonts/KaTeX_Caligraphic-Bold.ttf"),
            Self::CaligraphicRegular => include_bytes!("fonts/KaTeX_Caligraphic-Regular.ttf"),
            Self::FrakturBold => include_bytes!("fonts/KaTeX_Fraktur-Bold.ttf"),
            Self::FrakturRegular => include_bytes!("fonts/KaTeX_Fraktur-Regular.ttf"),
            Self::MainBold => include_bytes!("fonts/KaTeX_Main-Bold.ttf"),
            Self::MainBoldItalic => include_bytes!("fonts/KaTeX_Main-BoldItalic.ttf"),
            Self::MainItalic => include_bytes!("fonts/KaTeX_Main-Italic.ttf"),
            Self::MainRegular => include_bytes!("fonts/KaTeX_Main-Regular.ttf"),
            Self::MathBoldItalic => include_bytes!("fonts/KaTeX_Math-BoldItalic.ttf"),
            Self::MathItalic => include_bytes!("fonts/KaTeX_Math-Italic.ttf"),
            Self::SansSerifBold => include_bytes!("fonts/KaTeX_SansSerif-Bold.ttf"),
            Self::SansSerifItalic => include_bytes!("fonts/KaTeX_SansSerif-Italic.ttf"),
            Self::SansSerifRegular => include_bytes!("fonts/KaTeX_SansSerif-Regular.ttf"),
            Self::ScriptRegular => include_bytes!("fonts/KaTeX_Script-Regular.ttf"),
            Self::Size1Regular => include_bytes!("fonts/KaTeX_Size1-Regular.ttf"),
            Self::Size2Regular => include_bytes!("fonts/KaTeX_Size2-Regular.ttf"),
            Self::Size3Regular => include_bytes!("fonts/KaTeX_Size3-Regular.ttf"),
            Self::Size4Regular => include_bytes!("fonts/KaTeX_Size4-Regular.ttf"),
            Self::TypewriterRegular => include_bytes!("fonts/KaTeX_Typewriter-Regular.ttf"),
        }
    }
}

impl TryFrom<String> for Font {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Cmu-Serif-Roman" => Ok(Self::CmuSerifRoman),
            "AMS-Regular" => Ok(Self::AMSRegular),
            "Caligraphic-Bold" => Ok(Self::CaligraphicBold),
            "Caligraphic-Regular" => Ok(Self::CaligraphicRegular),
            "Fraktur-Bold" => Ok(Self::FrakturBold),
            "Fraktur-Regular" => Ok(Self::FrakturRegular),
            "Main-Bold" => Ok(Self::MainBold),
            "Main-BoldItalic" => Ok(Self::MainBoldItalic),
            "Main-Italic" => Ok(Self::MainItalic),
            "Main-Regular" => Ok(Self::MainRegular),
            "Math-BoldItalic" => Ok(Self::MathBoldItalic),
            "Math-Italic" => Ok(Self::MathItalic),
            "SansSerif-Bold" => Ok(Self::SansSerifBold),
            "SansSerif-Italic" => Ok(Self::SansSerifItalic),
            "SansSerif-Regular" => Ok(Self::SansSerifRegular),
            "Script-Regular" => Ok(Self::ScriptRegular),
            "Size1-Regular" => Ok(Self::Size1Regular),
            "Size2-Regular" => Ok(Self::Size2Regular),
            "Size3-Regular" => Ok(Self::Size3Regular),
            "Size4-Regular" => Ok(Self::Size4Regular),
            "Typewriter-Regular" => Ok(Self::TypewriterRegular),
            _ => Err(format!("Unknown font: {value}")),
        }
    }
}

// ----- Scene -----

impl Scene {
    pub fn font(&self, font: Font) -> &FontArc {
        &self.fonts.get(&font).expect("Font not found in font map")
    }

    pub fn default_font(&self) -> &FontArc {
        self.font(Font::default())
    }
}
