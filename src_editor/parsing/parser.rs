use std::collections::HashMap;
use super::{
    FileLine, LineContext, FileType,
    ColorInfo, Color,
    parse_syntax_section,
    parse_definition_section,
    parse_hypotesis_section,
    parse_assertion_section,
    parse_proof_section
};
use crate::library_data::{
    LibraryData, Reference, SyntaxType
};

pub fn parse_title(line: Vec<char>) -> (FileLine, FileType) {
    let line = line.into_iter().collect::<String>();
    if !line.starts_with("##") {
        let chars = line.chars().collect::<Vec<_>>();
        let colors = chars.iter().map(|_| ColorInfo::NO_COLOR).collect();
        return (
            FileLine { context: LineContext::Raw, chars, colors },
            FileType::Unknown
        );
    }
    let (title, name) = match line.rsplit_once(' ') {
        None => {
            let chars = line.chars().collect::<Vec<_>>();
            let colors = chars.iter().map(|_| ColorInfo::NO_COLOR).collect();
            return (
                FileLine { context: LineContext::Raw, chars, colors },
                FileType::Unknown
            );
        },
        Some((title, name)) => (title, name)
    };
    let (title_bg_color, file_type) = match title {
        "## Syntax Definition (formula)" => (Color::Blue, FileType::SyntaxDefinitionFormula),
        "## Syntax Definition (object)" => (Color::Blue, FileType::SyntaxDefinitionObject),
        "## Axiom" => (Color::Blue, FileType::Axiom),
        "## Theorem" => (Color::Blue, FileType::Theorem),
        _ => (Color::Red, FileType::Unknown)
    };
    let name_color = match name.chars().all(|c| c.is_ascii_alphanumeric()) {
        true => ColorInfo::fg_bg_color(Color::Black, Color::Cyan).underlined(),
        false => ColorInfo::fg_bg_color(Color::Black, Color::Red).underlined()
    };
    let chars = title.chars()
        .chain(Some(' '))
        .chain(name.chars())
        .collect();
    let colors = title.chars().chain(Some(' '))
        .map(|_| ColorInfo::fg_bg_color(Color::Black, title_bg_color).underlined())
        .chain( name.chars().map(|_| name_color) )
        .collect();

    (
        FileLine { context: LineContext::Title, chars, colors },
        file_type
    )
}

pub fn parse_file(
    lines: Vec<Vec<char>>, lib_data: &LibraryData, references: &HashMap<String, Reference>
) -> Vec<FileLine> {
    let mut lines = lines.into_iter();
    let first_line = match lines.next() {
        Some(line) => line,
        None => return vec![]
    };
    let (title, file_type) = parse_title(first_line);
    
    let mut sections = Vec::new();
    let mut temp = Vec::new();
    for line in lines {
        if line.first() == Some(&'#') {
            sections.push(temp);
            temp = Vec::new();
        };
        temp.push(line);
    };
    sections.push(temp);
    let mut sections = sections.into_iter();

    let mut result_lines = vec![ title ];
    if let Some(empty_first_section) = sections.next() {
        for line in empty_first_section {
            let colors = line.iter().map(|_| ColorInfo::NO_COLOR).collect();
            result_lines.push( FileLine { context: LineContext::Raw, chars: line, colors } );
        };
    };
    match file_type {
        FileType::SyntaxDefinitionFormula => {
            let Some(syntax_section) = sections.next() else {
                return result_lines;
            };
            let (
                mut syntax_lines, new_syntax
            ) = parse_syntax_section(syntax_section, SyntaxType::Formula);
            result_lines.append( &mut syntax_lines );
            if let Some(definition_section) = sections.next() {
                result_lines.append(
                    &mut parse_definition_section(definition_section, lib_data, new_syntax)
                );
            };
        },
        FileType::SyntaxDefinitionObject => {
            let Some(syntax_section) = sections.next() else {
                return result_lines;
            };
            let (
                mut syntax_lines, new_syntax
            ) = parse_syntax_section(syntax_section, SyntaxType::Object);
            result_lines.append( &mut syntax_lines );
            if let Some(definition_section) = sections.next() {
                result_lines.append(
                    &mut parse_definition_section(definition_section, lib_data, new_syntax)
                );
            };
        },
        FileType::Axiom => {
            if let Some(hypothesis_section) = sections.next() {
                let context = LineContext::AxiomHypothesis;
                result_lines.append(
                    // Hypotheses are not named in Axioms,
                    // So they are parsed as if they were assertions
                    &mut parse_assertion_section(hypothesis_section, lib_data, context)
                );
            };
            if let Some(assertion_section) = sections.next() {
                let context = LineContext::AssumedAssertion;
                result_lines.append(
                    &mut parse_assertion_section(assertion_section, lib_data, context)
                );
            };
        },
        FileType::Theorem => {
            let Some(hypothesis_section) = sections.next() else {
                return result_lines;
            };
            let (
                mut hypot_lines, hypot_names
            ) = parse_hypotesis_section(hypothesis_section, lib_data);
            result_lines.append( &mut hypot_lines );
            if let Some(assertion_section) = sections.next() {
                let context = LineContext::UnprovenAssertion;
                result_lines.append(
                    &mut parse_assertion_section(assertion_section, lib_data, context)
                );
            };
            if let Some(proof_section) = sections.next() {
                result_lines.append(
                    &mut parse_proof_section(proof_section, lib_data, references, hypot_names)
                );
            };
        },
        FileType::Unknown => {
            for lines in sections.by_ref() {
                for line in lines {
                    let colors = line.iter().map(|_| ColorInfo::NO_COLOR).collect();
                    result_lines.push( FileLine { context: LineContext::Raw, chars: line, colors } );
                };
            };
        }
    }
    for lines in sections {
        for line in lines {
            let colors = line.iter().map(|_| ColorInfo::fg_color(Color::Red)).collect();
            result_lines.push( FileLine { context: LineContext::UnexpectedLine, chars: line, colors } );
        };
    };
    result_lines
}
