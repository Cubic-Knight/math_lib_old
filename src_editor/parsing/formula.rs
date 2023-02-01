use std::collections::HashMap;
use super::{
    FileLine, LineContext,
    ColorInfo, Color
};
use crate::library_data::{
    Syntax, SyntaxType,
    Placeholder,
    LibraryData
};

const WFF_VAR_COLOR: ColorInfo = ColorInfo::fg_color(Color::Blue).bold();
const WFF_SINGLETON_COLOR: ColorInfo = ColorInfo::fg_color(Color::Green);
const WFF_SYNTAX_COLOR: ColorInfo = ColorInfo::fg_color(Color::Cyan);
const OBJ_VAR_COLOR: ColorInfo = ColorInfo::fg_color(Color::Red).bold();
const OBJ_SINGLETON_COLOR: ColorInfo = ColorInfo::fg_color(Color::Yellow);
const OBJ_SYNTAX_COLOR: ColorInfo = ColorInfo::fg_color(Color::Magenta);
const NEW_SYNTAX_COLOR: ColorInfo = ColorInfo::fg_color(Color::White);

pub fn parse_new_syntax(line: Vec<char>, syntax_type: SyntaxType) -> (FileLine, Option<Syntax>) {
    if line.len() == 0 {
        return (
            FileLine { context: LineContext::SyntaxDefinition, chars: vec![], colors: vec![] },
            None
        );
    };
    let mut chars = Vec::new();
    let mut colors = Vec::new();
    let mut formula = Vec::new();
    let mut wff_mapping = HashMap::new();
    let mut obj_mapping = HashMap::new();
    for c in line {
        if c == ' ' {
            chars.push(c);
            colors.push(ColorInfo::NO_COLOR);
            continue;
        };
        if c == '…' {
            chars.push(c);
            colors.push(ColorInfo::fg_color(Color::Black));
            formula.push(Placeholder::Repetition);
        } else if '𝑎' <= c && c <= '𝑧' {  // '𝑎' and '𝑧' here are NOT ascii
            chars.push(c);
            colors.push(OBJ_VAR_COLOR);
            match wff_mapping.get(&c) {
                Some(id) => formula.push(Placeholder::WellFormedFormula(*id)),
                None => {
                    formula.push(Placeholder::WellFormedFormula(wff_mapping.len()));
                    wff_mapping.insert(c, wff_mapping.len());
                }
            };
        } else if '𝛼' <= c && c <= '𝜔' {
            chars.push(c);
            colors.push(WFF_VAR_COLOR);
            match obj_mapping.get(&c) {
                Some(id) => formula.push(Placeholder::Object(*id)),
                None => {
                    formula.push(Placeholder::Object(obj_mapping.len()));
                    obj_mapping.insert(c, obj_mapping.len());
                }
            };
        } else {
            chars.push(c);
            colors.push(NEW_SYNTAX_COLOR);
            formula.push(Placeholder::LiteralChar(c))
        };
    };
    let syntax = Syntax {
        syntax_type,
        formula,
        distinct_wff_count: wff_mapping.len(),
        distinct_object_count: obj_mapping.len()
    };
    (FileLine { context: LineContext::SyntaxDefinition, chars, colors }, Some(syntax))
}

#[derive(Debug)]
enum PartiallyCompiled {
    NotCompiled(char),
    Space,
    CompiledFormula { chars: Vec<char>, colors: Vec<ColorInfo> },
    CompiledObject { chars: Vec<char>, colors: Vec<ColorInfo> }
}

fn monochromatic_formula(line: String, color: Color, context: LineContext) -> FileLine {
    let color = ColorInfo::fg_color(color);
    FileLine {
        context,
        chars: line.chars().collect(),
        colors: line.chars().map(|_| color).collect()
    }
}

pub fn parse_formula(
    line: Vec<char>, lib_data: &LibraryData, additional_syntax: Option<Syntax>, context: LineContext
) -> FileLine {
    let line = line.into_iter().collect::<String>();
    let leading_spaces_count = line.len() - line.trim_start().len();
    let trailing_spaces_count = line.len() - line.trim_end().len();
    let try_parsing = line.trim().chars()
        .map(|ch| match ch {
            '…' => Err(()),
            ' ' => Ok(PartiallyCompiled::Space),
            c @ '𝑎'..='𝑧' => Ok(PartiallyCompiled::CompiledObject {
                chars: vec![ c ], colors: vec![ OBJ_VAR_COLOR ]
            }),
            c @ '𝛼'..='𝜔' => Ok(PartiallyCompiled::CompiledFormula {
                chars: vec![ c ], colors: vec![ WFF_VAR_COLOR ]
            }),
            c => Ok(PartiallyCompiled::NotCompiled(c))
        })
        .collect::<Result<Vec<_>, _>>();
    let mut partially_compiled = match try_parsing {
        Ok(list) => list,
        Err(()) => return monochromatic_formula(line, Color::Red, context)
    };
    let syntaxes = additional_syntax.iter()
        .chain(lib_data.syntaxes.iter())
        .collect::<Vec<_>>();
    'try_find_patterns: while partially_compiled.len() > 1 {
        'try_syntax: for (syntax_id, syntax) in syntaxes.iter().enumerate() {
            'try_position: for index in 0..partially_compiled.len() {
                if partially_compiled.len() - index < syntax.formula.len() {
                    continue 'try_syntax;  // If the syntax does not fit in the rest of the text, go to the next syntax
                };
                if let Some(PartiallyCompiled::Space) = partially_compiled.get(index) {
                    continue 'try_position;  // We ignore spaces so we skip any leading space
                };
                let syntax_color = match (
                    &syntax.syntax_type, syntax.distinct_wff_count, syntax.distinct_object_count
                ) {
                    (_, _, _) if syntax_id == 0 && additional_syntax.is_some() => NEW_SYNTAX_COLOR,
                    (SyntaxType::Formula, 0, 0) => WFF_SINGLETON_COLOR,
                    (SyntaxType::Formula, _, _) => WFF_SYNTAX_COLOR,
                    (SyntaxType::Object, 0, 0) => OBJ_SINGLETON_COLOR,
                    (SyntaxType::Object, _, _) => OBJ_SYNTAX_COLOR
                };

                let mut wffs = vec![None; syntax.distinct_wff_count];
                let mut objects = vec![None; syntax.distinct_object_count];
                
                let mut i = index;
                let mut chars = Vec::new();
                let mut colors = Vec::new();
                let mut syntax_length = 0;
                for pl in &syntax.formula {
                    let c = loop {
                        match partially_compiled.get(i) {
                            Some(PartiallyCompiled::Space) => {
                                chars.push(' ');
                                colors.push(ColorInfo::NO_COLOR);
                                syntax_length += 1;
                                i += 1;
                            },
                            Some(chr) => break chr,
                            None => continue 'try_syntax
                        };
                    };
                    let valid = match (c, pl) {
                        (
                            PartiallyCompiled::NotCompiled(c1),
                            Placeholder::LiteralChar(c2)
                        ) => {
                            chars.push(*c1);
                            colors.push(syntax_color);
                            syntax_length += 1;
                            c1 == c2
                        },
                        (
                            PartiallyCompiled::CompiledFormula {
                                chars: chs, colors: cols
                            },
                            Placeholder::WellFormedFormula(id)
                        ) => {
                            chars.extend(chs);
                            colors.extend(cols);
                            syntax_length += 1;
                            match &wffs[*id] {
                                Some(chrs) => chs == *chrs,
                                None => { wffs[*id] = Some(chs); true }
                            }
                        },
                        (
                            PartiallyCompiled::CompiledObject {
                                chars: chs, colors: cols
                            },
                            Placeholder::Object(id)
                        ) => {
                            chars.extend(chs);
                            colors.extend(cols);
                            syntax_length += 1;
                            match &objects[*id] {
                                Some(chrs) => chs == *chrs,
                                None => { objects[*id] = Some(chs); true }
                            }
                        },
                        _ => false
                    };
                    match valid {
                        true => (),
                        false => continue 'try_position  // Syntax did not match here, try next position
                    };
                    i += 1;
                };
                for _ in 0..syntax_length {
                    partially_compiled.remove(index);
                };
                let element_to_insert = match syntax.syntax_type {
                    SyntaxType::Formula => PartiallyCompiled::CompiledFormula {
                        chars, colors
                    },
                    SyntaxType::Object => PartiallyCompiled::CompiledObject {
                        chars, colors
                    }
                };
                partially_compiled.insert(index, element_to_insert);
                // Once a syntax has been compiled, we iterate through the list again from the beginning
                continue 'try_find_patterns;
            };
        };
        // We only can get here if no syntax has matched
        return monochromatic_formula(line, Color::Red, context);
    };

    match partially_compiled.pop() {
        Some(
            PartiallyCompiled::CompiledFormula { chars, colors }
        ) => FileLine {
            context,
            chars: vec![' '; leading_spaces_count].into_iter()
                .chain(chars)
                .chain(vec![' '; trailing_spaces_count])
                .collect(),
            colors: vec![ColorInfo::NO_COLOR; leading_spaces_count].into_iter()
            .chain(colors)
            .chain(vec![ColorInfo::NO_COLOR; trailing_spaces_count])
            .collect()
        },
        _ => monochromatic_formula(line, Color::Red, context)
    }
}
