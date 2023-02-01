use std::io::{stdout, Write};
use super::{
    MenuGraphics, MenuLine,
    FileGraphics
};
use crate::parsing::{
    FileLine, ColorInfo
};

const LINE_TERMINATOR: &'static str = "\x1b[m\r\n";

pub fn display_menu(menu: &MenuGraphics, dimensions: &(usize, usize)) {
    let MenuGraphics { cursor, camera, lines } = menu;

    print!("\x1b[2J\x1b[H"); // Clear display
    let start = camera.saturating_sub(1);
    let end = start + dimensions.1.saturating_sub(2);
    for index in start..end {
        let line = match lines.get(index) {
            Some(line) => line,
            None => break
        };
        let (indent, color, name) = match line {
            MenuLine::RootDirectory(name) => {
                ("", "\x1b[1;4;33m", name.to_owned())
            },
            MenuLine::SubDirectory(name, is_last) => {
                let indent = match is_last {
                    true => "\x1b[1;33m┖──\x1b[m ",
                    false => "\x1b[1;33m┠──\x1b[m "
                };
                let color = match index + 1 == *cursor {
                    true => "\x1b[1;7;34m",
                    false => "\x1b[1;34m"
                };
                let name = name.chars()
                    .take(dimensions.0.saturating_sub(5))
                    .collect::<String>();
                (indent, color, name)
            },
            MenuLine::File(name, _, is_in_last_dir, is_last_in_dir) => {
                let indent = match (is_in_last_dir, is_last_in_dir) {
                    (true, true) => "    \x1b[1;34m┖──\x1b[m ",
                    (true, false) => "    \x1b[1;34m┠──\x1b[m ",
                    (false, true) => "\x1b[1;33m┃\x1b[m   \x1b[1;34m┖──\x1b[m ",
                    (false, false) => "\x1b[1;33m┃\x1b[m   \x1b[1;34m┠──\x1b[m ",
                };
                let color = match index + 1 == *cursor {
                    true => "\x1b[7m",
                    false => "\x1b[m"
                };
                let name = name.chars()
                    .take(dimensions.0.saturating_sub(9))
                    .collect::<String>();
                (indent, color, name)
            },
        };
        print!("{}{}{}{}", indent, color, name, LINE_TERMINATOR);
    };
    stdout().flush().unwrap();
}

pub fn display_file(file: &FileGraphics, dimensions: &(usize, usize), indent: usize) {
    let FileGraphics {
        cursor, camera,
        lines,
        read_only: _
    } = file;

    let mut to_print = String::new();
    let mut line_count = 0;
    for (lno, line) in lines.iter().enumerate().skip(camera.saturating_sub(1)) {
        if line_count >= dimensions.1.saturating_sub(1) {
            break;
        };
        let FileLine { context: _, chars, colors } = line;
        let indexed_chars_and_colors = chars.into_iter()
            .chain(Some(&' '))
            .zip( colors.into_iter().chain( Some(&ColorInfo::NO_COLOR) ) )
            .enumerate();

        let mut current_string = String::new();
        let mut char_count = 0;
        let mut current_color = ColorInfo::NO_COLOR;
        for (cno, (ch, col)) in indexed_chars_and_colors {
            if char_count >= dimensions.0.saturating_sub(1) {
                to_print.push_str(&current_string);
                to_print.push_str(LINE_TERMINATOR);
                current_string = " ".repeat(indent);
                char_count = indent;
                line_count += 1;
            };
            if col != &current_color {
                current_color = *col;
                current_string.push_str(&col.to_escape_string());
                // "\x1b[{col}G" is to force cursor to stay in the correct column
                // There should not be a need for that escape sequence
                //  but there is a bug in the terminal's code with chars
                //  that are above U+FFFF that causes the terminal to think
                //  they are two characters instead of one
                current_string.push_str( &format!("\x1b[{}G", char_count+1) );
            };
            if (lno+1, cno+1) == *cursor {
                current_string.push_str("\x1b[7m");
                current_string.push_str( &format!("\x1b[{}G", char_count+1) );
            };
            current_string.push(*ch);
            if (lno+1, cno+1) == *cursor {
                current_string.push_str("\x1b[m");
                current_string.push_str( &format!("\x1b[{}G", char_count+2) );
                current_color = ColorInfo::NO_COLOR;
            };
            char_count += 1;
        };
        to_print.push_str(&current_string);
        to_print.push_str(LINE_TERMINATOR);
        line_count += 1;
    }

    print!("\x1b[2J\x1b[H"); // Clear display
    print!("{}", to_print);
    print!("\x1b[{};{}H", cursor.0, cursor.1);
    stdout().flush().unwrap();
}

pub fn display_command_bar(command: &str, dimensions: &(usize, usize)) {
    let bar_width = (dimensions.0 * 3 / 4).clamp(15, 35);
    let bar_begin = dimensions.0.saturating_sub(bar_width+1);
    let bar_capacity = dimensions.0.saturating_sub(bar_begin+1);
    
    let intro = match bar_capacity {
        0..=4 => "",
        5..=9 => ": ",
        10..=14 => "CC: ",
        15..=24 => "Code: ",
        _ => "Char Code: "
    };
    let to_print = intro.chars()
        .chain( command.chars() )
        .chain( std::iter::repeat(' ') )
        .take(bar_capacity)
        .collect::<String>();

    print!("\x1b[1;{}H", bar_begin+1);  // Set cursor pos
    print!("\x1b[0;30;47m{}\x1b[m", to_print);
    stdout().flush().unwrap();
}
