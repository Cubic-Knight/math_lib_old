pub enum FileType {
    SyntaxDefinitionFormula,
    SyntaxDefinitionObject,
    Axiom,
    Theorem,
    Unknown
}

#[derive(Clone, Copy)]
pub enum LineContext {
    Raw,
    Title,
    Section,
    SyntaxDefinition,
    AxiomHypothesis,
    Hypothesis,
    UnprovenAssertion,
    // ProvenAssertion,  // Not implemented for now
    AssumedAssertion,
    ProofLine,
    UnexpectedLine
}

#[derive(Clone)]
pub struct FileLine {
    pub context: LineContext,
    pub chars: Vec<char>,
    pub colors: Vec<ColorInfo>
}

#[derive(Debug)]
#[derive(PartialEq, Clone, Copy)]
pub enum Color {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7
}

#[derive(Debug)]
#[derive(PartialEq, Clone, Copy)]
pub struct ColorInfo {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub underline: bool
}

impl ColorInfo {
    pub fn to_escape_string(self) -> String {
        let mut result = String::from("\x1b[0");
        if self.bold { result.push_str(";1") };
        if self.underline { result.push_str(";4") };
        if let Some(color) = self.fg {
            result.push_str( &format!(";3{}", color as u8) );
        };
        if let Some(color) = self.bg {
            result.push_str( &format!(";4{}", color as u8) );
        };
        result.push('m');
        result
    }

    pub const NO_COLOR: Self = ColorInfo {
        fg: None, bg: None, bold: false, underline: false
    };
    pub const fn fg_color(color: Color) -> Self {
        ColorInfo { fg: Some(color), bg: None, bold: false, underline: false }
    }
    pub const fn fg_bg_color(c1: Color, c2: Color) -> Self {
        ColorInfo { fg: Some(c1), bg: Some(c2), bold: false, underline: false }
    }
    pub const fn bold(self) -> Self {
        let ColorInfo { fg, bg, bold:_, underline } = self;
        ColorInfo { fg, bg, bold: true, underline }
    }
    pub const fn underlined(self) -> Self {
        let ColorInfo { fg, bg, bold, underline: _ } = self;
        ColorInfo { fg, bg, bold, underline: true }
    }
    pub const fn bold_underlined(self) -> Self {
        let ColorInfo { fg, bg, bold: _, underline: _ } = self;
        ColorInfo { fg, bg, bold: true, underline: true }
    }
}
