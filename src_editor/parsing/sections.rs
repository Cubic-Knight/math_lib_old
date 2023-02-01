use std::collections::HashMap;
use std::iter::repeat;
use super::{
    FileLine, LineContext,
    ColorInfo, Color,
    parse_new_syntax,
    parse_formula
};
use crate::library_data::{
    LibraryData, Reference,
    Syntax, SyntaxType
};

pub fn parse_syntax_section(section: Vec<Vec<char>>, syntax_type: SyntaxType) -> (Vec<FileLine>, Option<Syntax>) {
    let mut lines = section.into_iter();

    let section_name_line = match lines.next() {
        Some(first_line) => {
            let section_name = first_line.into_iter().collect::<String>();
            let name_color = match section_name == "# Syntax" {
                true => ColorInfo::fg_color(Color::White).bold_underlined(),
                false => ColorInfo::fg_color(Color::Red)
            };
            let chars = section_name.chars().collect::<Vec<_>>();
            let colors = chars.iter().map(|_| name_color).collect();
            FileLine { context: LineContext::Section, chars, colors }
        },
        None => return (vec![], None)
    };
    let (syntax_def_line, syntax) = match lines.next() {
        Some(line) => parse_new_syntax(line, syntax_type),
        None => return (vec![ section_name_line ], None)
    };
    
    let mut result_lines = vec![ section_name_line, syntax_def_line ];
    for line in lines {
        let colors = line.iter().map(|_| ColorInfo::fg_color(Color::Red)).collect();
        result_lines.push( FileLine { context: LineContext::UnexpectedLine, chars: line, colors } );
    };
    (result_lines, syntax)
}

pub fn parse_definition_section(
    section: Vec<Vec<char>>, lib_data: &LibraryData, new_syntax: Option<Syntax>
) -> Vec<FileLine> {
    let mut lines = section.into_iter();

    let section_name_line = match lines.next() {
        Some(first_line) => {
            let section_name = first_line.into_iter().collect::<String>();
            let name_color = match section_name == "# Definition" {
                true => ColorInfo::fg_color(Color::White).bold_underlined(),
                false => ColorInfo::fg_color(Color::Red)
            };
            let chars = section_name.chars().collect::<Vec<_>>();
            let colors = chars.iter().map(|_| name_color).collect();
            FileLine { context: LineContext::Section, chars, colors }
        },
        None => return vec![]
    };
    let definition_line = match lines.next() {
        Some(line) => {
            let context = LineContext::AssumedAssertion;
            parse_formula(line, lib_data, new_syntax, context)
        },
        None => return vec![ section_name_line ]
    };
    
    let mut result_lines = vec![ section_name_line, definition_line ];
    for line in lines {
        let colors = line.iter().map(|_| ColorInfo::fg_color(Color::Red)).collect();
        result_lines.push( FileLine { context: LineContext::UnexpectedLine, chars: line, colors } );
    };
    result_lines
}

pub fn parse_hypotesis_section(
    section: Vec<Vec<char>>, lib_data: &LibraryData
) -> (Vec<FileLine>, Vec<String>) {
    let mut lines = section.into_iter();

    let section_name_line = match lines.next() {
        Some(first_line) => {
            let section_name = first_line.into_iter().collect::<String>();
            let is_valid = section_name == "# Hypothesis" || section_name == "# Hypotheses";
            let name_color = match is_valid {
                true => ColorInfo::fg_color(Color::White).bold_underlined(),
                false => ColorInfo::fg_color(Color::Red)
            };
            let chars = section_name.chars().collect::<Vec<_>>();
            let colors = chars.iter().map(|_| name_color).collect();
            FileLine { context: LineContext::Section, chars, colors }
        },
        None => return (vec![], vec![])
    };

    let mut result_lines = vec![ section_name_line ];
    let mut hypot_names = Vec::new();
    for line in lines {
        let mut split = line.splitn(2, |c| *c == ':');
        let hypothesis = match (split.next(), split.next()) {
            (Some(name), Some(hypot)) => {
                hypot_names.push( name.iter().collect::<String>() );
                let context = LineContext::Hypothesis;
                let FileLine {
                    context: _, chars: hyp_chars, colors: hyp_colors
                } = parse_formula(hypot.to_vec(), lib_data, None, context);
                let chars = name.to_vec().into_iter()
                    .chain(Some(':'))
                    .chain(hyp_chars)
                    .collect();
                let colors = name.iter()
                    .map(|_| ColorInfo::NO_COLOR)
                    .chain(Some(ColorInfo::NO_COLOR))
                    .chain(hyp_colors)
                    .collect();
                FileLine { context, chars, colors }
            },
            (Some(line), None) => {
                let chars = line.to_vec();
                let colors = chars.iter().map(
                    |_| ColorInfo::fg_color(Color::Red)
                ).collect();
                FileLine { context: LineContext::UnexpectedLine, chars, colors }
            },
            _ => unreachable!()
        };
        result_lines.push( hypothesis );
    };
    (result_lines, hypot_names)
}

pub fn parse_assertion_section(
    section: Vec<Vec<char>>, lib_data: &LibraryData, context: LineContext
) -> Vec<FileLine> {
    let mut lines = section.into_iter();

    let section_name_line = match lines.next() {
        Some(first_line) => {
            let section_name = first_line.into_iter().collect::<String>();
            let is_valid = match (context, section_name.as_str()) {
                (LineContext::AxiomHypothesis, "# Hypothesis") => true,
                (LineContext::AxiomHypothesis, "# Hypotheses") => true,
                (_, "# Assertion") => true,
                (_, "# Assertions") => true,
                _ => false
            };
            let name_color = match is_valid {
                true => ColorInfo::fg_color(Color::White).bold_underlined(),
                false => ColorInfo::fg_color(Color::Red)
            };
            let chars = section_name.chars().collect::<Vec<_>>();
            let colors = chars.iter().map(|_| name_color).collect();
            FileLine { context: LineContext::Section, chars, colors }
        },
        None => return vec![]
    };

    let mut result_lines = vec![ section_name_line ];
    for line in lines {
        let assertion = parse_formula(line, lib_data, None, context);
        result_lines.push( assertion );
    };
    result_lines
}


fn parse_used_hypots(used_hypots: &str, line_no: &str) -> Vec<(char, ColorInfo)> {
    let line_num = match line_no.parse::<usize>() {
        Ok(n) => n,
        Err(_) => {
            return used_hypots.chars()
                .map(|c| (c, ColorInfo::NO_COLOR))
                .collect::<Vec<_>>()
        }
    };
    used_hypots.split(',')
        .map(|s| {
            let leading_spaces = s.len() - s.trim_start().len();
            let trailing_spaces = s.len() - s.trim_end().len();
            let color = match s.trim().parse::<usize>() {
                Ok(n) if n < line_num => ColorInfo::NO_COLOR,
                _ => ColorInfo::fg_color(Color::Red)
            };
            Some((',', ColorInfo::NO_COLOR)).into_iter()
                .chain(
                    repeat((' ', ColorInfo::NO_COLOR)).take(leading_spaces)
                ).chain(
                    s.trim().chars().zip( repeat(color) )
                ).chain(
                    repeat((' ', ColorInfo::NO_COLOR)).take(trailing_spaces)
                )
        }).flatten()
        .skip(1)  // Skip the first comma
        .collect::<Vec<_>>()
}

fn theo_is_valid(
    theo_ref: &str, hypot_names: &Vec<String>,
    lib_data: &LibraryData, references: &HashMap<String, Reference>
) -> bool {
    if hypot_names.contains(&theo_ref.to_owned()) {
        return true;
    };
    let (name, sub_id) = match theo_ref.split_once('.') {
        Some((name, num)) => {
            let Ok(sub_id) = num.parse::<usize>() else {
                return false;
            };
            (name, sub_id)
        },
        None => (theo_ref, 1)
    };
    if sub_id == 0 { return false; }
    match references.get(name) {
        Some(Reference::DefinitionReference(_)) => sub_id == 1,
        Some(Reference::AxiomReference(id, _)) => {
            sub_id <= lib_data.axioms[*id].assertions.len()
        },
        Some(Reference::TheoremReference(id, _)) => {
            sub_id <= lib_data.theorems[*id].assertions.len()
        },
        _ => false
    }
}

pub fn parse_proof_section(
    section: Vec<Vec<char>>, lib_data: &LibraryData,
    references: &HashMap<String, Reference>, hypot_names: Vec<String>
) -> Vec<FileLine> {
    let mut lines = section.into_iter();

    let section_name_line = match lines.next() {
        Some(first_line) => {
            let section_name = first_line.into_iter().collect::<String>();
            let name_color = match section_name == "# Proof" {
                true => ColorInfo::fg_color(Color::White).bold_underlined(),
                false => ColorInfo::fg_color(Color::Red)
            };
            let chars = section_name.chars().collect::<Vec<_>>();
            let colors = chars.iter().map(|_| name_color).collect();
            FileLine { context: LineContext::Section, chars, colors }
        },
        None => return vec![]
    };

    let mut max_line_no_len = 2;
    let mut max_used_hypots_len = 2;
    let mut max_theo_ref_len = 2;
    let mut preparsed_lines = Vec::new();
    for (i, line) in lines.enumerate() {
        let line = line.into_iter().collect::<String>();
        let mut parts = line.splitn(4, ';')
            .map(|s| s.trim());
        let line_no = parts.next().unwrap_or("").to_owned();
        let line_no_color = match line_no.parse::<usize>() == Ok(i+1) {
            true => ColorInfo::NO_COLOR,
            false => ColorInfo::fg_color(Color::Red)
        };
        let line_no_len = line_no.chars().count();
        if line_no_len > max_line_no_len { max_line_no_len = line_no_len; };

        let used_hypots = parts.next().unwrap_or("");
        let used_hypots_color = parse_used_hypots(used_hypots, &line_no);
        let used_hypots_len = used_hypots.chars().count();
        if used_hypots_len > max_used_hypots_len { max_used_hypots_len = used_hypots_len; };

        let theo_ref = parts.next().unwrap_or("").to_owned();
        let theo_valid = theo_is_valid(&theo_ref, &hypot_names, lib_data, references);
        let theo_ref_color = match theo_valid {
            true => ColorInfo::NO_COLOR,
            false => ColorInfo::fg_color(Color::Red)
        };
        let theo_ref_len = theo_ref.chars().count();
        if theo_ref_len > max_theo_ref_len { max_theo_ref_len = theo_ref_len; };

        let context = LineContext::ProofLine;
        let line = parts.next().unwrap_or("").chars().collect::<Vec<_>>();
        let resulting_formula = parse_formula(
            line, lib_data, None, context
        );

        preparsed_lines.push((
            (line_no, line_no_color),
            used_hypots_color,
            (theo_ref, theo_ref_color),
            resulting_formula
        ));
    };
    let mut result_lines = vec![ section_name_line ];
    for line in preparsed_lines {
        let (
            (line_no, line_no_color),
            used_hypots_color,
            (theo_ref, theo_ref_color),
            FileLine {
                context,
                chars: mut formula_chars,
                colors: mut formula_colors
            }
        ) = line;

        let mut chars = Vec::new();
        let mut colors = Vec::new();
        for c in line_no.chars().chain( repeat(' ') ).take(max_line_no_len) {
            chars.push(c);
            colors.push(line_no_color);
        };
        chars.extend_from_slice( &[' ', ';', ' '] );
        colors.extend_from_slice( &[ColorInfo::NO_COLOR; 3] );
        let used_hypots_color_iter = used_hypots_color.into_iter()
            .chain( repeat((' ', ColorInfo::NO_COLOR)) )
            .take(max_used_hypots_len);
        for (c, col) in used_hypots_color_iter {
            chars.push(c);
            colors.push(col);
        };
        chars.extend_from_slice( &[' ', ';', ' '] );
        colors.extend_from_slice( &[ColorInfo::NO_COLOR; 3] );
        for c in theo_ref.chars().chain( repeat(' ') ).take(max_theo_ref_len) {
            chars.push(c);
            colors.push(theo_ref_color);
        };
        chars.extend_from_slice( &[' ', ';', ' '] );
        colors.extend_from_slice( &[ColorInfo::NO_COLOR; 3] );
        chars.append( &mut formula_chars );
        colors.append( &mut formula_colors );

        result_lines.push( FileLine { context, chars, colors } )
    };
    result_lines
}
