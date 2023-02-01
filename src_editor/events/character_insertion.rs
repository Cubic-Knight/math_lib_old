use super::EditorData;
use crate::parsing::{
    parse_file,
    parse_new_syntax,
    parse_formula,
    ColorInfo, Color,
    FileLine, LineContext
};
use crate::library_data::SyntaxType;

fn update_file(line: Vec<char>, context: LineContext, editor_data: &mut EditorData) {
    let (cy, _) = editor_data.file.cursor;
    let l_index = cy.saturating_sub(1);

    let file_line_to_inject = match context {
        // If the line changed is important
        // we want to reparse the whole file 
        LineContext::Title
        | LineContext::Section
        | LineContext::Hypothesis
        | LineContext::ProofLine
        => Err(line),
        // Here we only parse the changed line
        LineContext::SyntaxDefinition => {
            let (file_line, _) = parse_new_syntax(line, SyntaxType::Formula);
            Ok(file_line)
        },
        LineContext::AxiomHypothesis => Ok(
            parse_formula(line, &editor_data.lib_data, None, context)
        ),
        LineContext::UnprovenAssertion => Ok(
            parse_formula(line, &editor_data.lib_data, None, context)
        ),
        LineContext::AssumedAssertion => Ok(
            parse_formula(line, &editor_data.lib_data, None, context)
        ),
        // These require no additional parsing
        LineContext::Raw => {
            let colors = line.iter()
                .map(|_| ColorInfo::NO_COLOR)
                .collect::<Vec<_>>();
            Ok(FileLine { context, chars: line, colors })
        },
        LineContext::UnexpectedLine => {
            let colors = line.iter()
                .map(|_| ColorInfo::fg_color(Color::Red))
                .collect::<Vec<_>>();
            Ok(FileLine { context, chars: line, colors })
        }
    };
    match file_line_to_inject {
        Ok(line) => editor_data.file.lines[l_index] = line,
        Err(line) => {
            let mut lines = editor_data.file.lines.iter()
                .map(|fl| fl.chars.clone())
                .collect::<Vec<_>>();
            lines[l_index] = line;
            editor_data.file.lines = parse_file(
                lines, &editor_data.lib_data, &editor_data.references
            );
        }
    };
}

pub fn insert_character(ch: char, editor_data: &mut EditorData) {
    let (cy, cx) = editor_data.file.cursor;
    let l_index = cy.saturating_sub(1);
    let c_index = cx.saturating_sub(1);
    let Some(current_line) = editor_data.file.lines.get(l_index) else {
        return;
    };

    let mut current_chars = current_line.chars.iter();
    let mut new_line = current_chars.by_ref().take(c_index)
        .map(|c| *c)    
        .chain( Some(ch) )
        .collect::<Vec<_>>();
    new_line.extend( current_chars );

    let context = current_line.context;
    update_file(new_line, context, editor_data);
}

pub fn insert_newline(editor_data: &mut EditorData) {
    let (cy, cx) = editor_data.file.cursor;
    let l_index = cy.saturating_sub(1);
    let c_index = cx.saturating_sub(1);

    let last_lines = editor_data.file.lines
        .split_off(l_index + 1)
        .into_iter()
        .map(|fl| fl.chars)
        .collect::<Vec<_>>();
    let current_line = editor_data.file.lines.pop().unwrap();
    let first_lines = editor_data.file.lines
        .split_off(0)
        .into_iter()
        .map(|fl| fl.chars)
        .collect::<Vec<_>>();
    
    let mut first_part = current_line.chars;
    let second_part = first_part.split_off(c_index);

    let lines = first_lines.into_iter()
        .chain( Some(first_part) )
        .chain( Some(second_part) )
        .chain( last_lines.into_iter() )
        .collect::<Vec<_>>();
    editor_data.file.lines = parse_file(
        lines, &editor_data.lib_data, &editor_data.references
    );
}

pub fn delete_character(editor_data: &mut EditorData) {
    let (cy, cx) = editor_data.file.cursor;
    if cx == 1 {
        delete_newline(editor_data);
        return;
    };

    let l_index = cy.saturating_sub(1);
    let c_index = cx.saturating_sub(1);
    let Some(current_line) = editor_data.file.lines.get(l_index) else {
        return;
    };

    let mut current_chars = current_line.chars.iter();
    let mut new_line = current_chars.by_ref().take(c_index.saturating_sub(1))
        .map(|c| *c)
        .collect::<Vec<_>>();
    current_chars.next();  // consume char that should be destroyed
    new_line.extend( current_chars );

    let context = current_line.context;
    update_file(new_line, context, editor_data);
}

fn delete_newline(editor_data: &mut EditorData) {
    let (cy, _) = editor_data.file.cursor;
    let l_index = cy.saturating_sub(1);

    let last_lines = editor_data.file.lines
        .split_off(l_index + 1)
        .into_iter()
        .map(|fl| fl.chars)
        .collect::<Vec<_>>();
    let second_part = editor_data.file.lines.pop().unwrap();
    let first_part = editor_data.file.lines.pop().unwrap();
    let first_lines = editor_data.file.lines
        .split_off(0)
        .into_iter()
        .map(|fl| fl.chars)
        .collect::<Vec<_>>();
    
    let combined_line = first_part.chars.into_iter()
        .chain( second_part.chars.into_iter() )
        .collect::<Vec<_>>();

    let lines = first_lines.into_iter()
        .chain( Some(combined_line) )
        .chain( last_lines.into_iter() )
        .collect::<Vec<_>>();
    editor_data.file.lines = parse_file(
        lines, &editor_data.lib_data, &editor_data.references
    );
}
