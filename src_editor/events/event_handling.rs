use std::collections::HashMap;
use super::{
    Event, Direction,
    move_cursor_up,
    move_cursor_down,
    move_cursor_left,
    move_cursor_right,
    insert_character,
    insert_newline,
    delete_character
};
use crate::graphics::{
    get_menu, MenuGraphics, MenuLine,
    get_file, FileGraphics,
    display_menu, display_file,
    display_command_bar
};
use crate::library_data::{Reference, LibraryData};
use termwiz::input::{KeyCode, Modifiers};

pub enum EditorState {
    InMenu,
    EditingFile,
    InsertSpecialChar,
    ShouldExit
}

pub struct EditorData {
    pub state: EditorState,
    pub menu: MenuGraphics,
    pub file: FileGraphics,
    pub special_char_command: String,
    pub dimensions: (usize, usize),
    pub indent: usize,
    pub lib_data: LibraryData,
    pub references: HashMap<String, Reference>
}

impl Default for EditorData {
    fn default() -> Self {
        EditorData {
            state: EditorState::InMenu,
            menu: get_menu().unwrap(),
            file: FileGraphics::default(),
            special_char_command: String::with_capacity(20),
            dimensions: (80, 24),
            indent: 4,
            lib_data: LibraryData {
                syntaxes: Vec::new(),
                definitions: Vec::new(),
                axioms: Vec::new(),
                theorems: Vec::new()
            },
            references: HashMap::new()
        }
    }
}

pub fn handle_event_in_menu(event: Event, editor_data: &mut EditorData) {
    match event {
        Event::KeyboardInterrupt => editor_data.state = EditorState::ShouldExit,
        Event::KeyPressedArrow(direction, Modifiers::NONE) => {
            editor_data.menu.cursor = match direction {
                Direction::Up => (editor_data.menu.cursor - 1).max(2),
                Direction::Down => (editor_data.menu.cursor + 1).min(editor_data.menu.lines.len()),
                Direction::Left => return,
                Direction::Right => return
            };
            display_menu(&editor_data.menu, &editor_data.dimensions);
        },
        Event::KeyPressedArrow(direction, Modifiers::SHIFT) => {
            editor_data.menu.camera = match direction {
                Direction::Up => (editor_data.menu.camera - 1).max(1),
                Direction::Down => (editor_data.menu.camera + 1).min(editor_data.menu.lines.len()),
                Direction::Left => return,
                Direction::Right => return
            };
            display_menu(&editor_data.menu, &editor_data.dimensions);
        },
        Event::KeyPressedOther(KeyCode::Enter, Modifiers::NONE) => {
            let index = editor_data.menu.cursor - 1;
            if let Some(MenuLine::File(_, path, _, _)) = editor_data.menu.lines.get(index) {
                editor_data.file = get_file(
                    path.to_owned(), &editor_data.lib_data, &editor_data.references
                ).unwrap();
                editor_data.state = EditorState::EditingFile;
                display_file(&editor_data.file, &editor_data.dimensions, editor_data.indent);
            };
        },
        Event::WindowResize(cols, rows) => {
            editor_data.dimensions = (cols, rows);
            display_menu(&editor_data.menu, &editor_data.dimensions);
        },
        _ => ()
    };
}

pub fn handle_event_in_file_edition(event: Event, editor_data: &mut EditorData) {
    match event {
        Event::KeyPressedOther(KeyCode::Escape, Modifiers::NONE) => {
            editor_data.state = EditorState::InMenu;
            display_menu(&editor_data.menu, &editor_data.dimensions);
        },
        Event::KeyPressedArrow(direction, Modifiers::NONE) => {
            let new_pos = match direction {
                Direction::Up => move_cursor_up(editor_data),
                Direction::Down => move_cursor_down(editor_data),
                Direction::Left => move_cursor_left(editor_data),
                Direction::Right => move_cursor_right(editor_data)
            };
            match new_pos {
                Some(pos) => editor_data.file.cursor = pos,
                None => return
            };
            display_file(&editor_data.file, &editor_data.dimensions, editor_data.indent);
        },
        Event::KeyPressedArrow(direction, Modifiers::SHIFT) => {
            editor_data.file.camera = match direction {
                Direction::Up => (editor_data.file.camera - 1).max(1),
                Direction::Down => (editor_data.file.camera + 1).min(editor_data.file.lines.len()),
                Direction::Left => return,
                Direction::Right => return
            };
            display_file(&editor_data.file, &editor_data.dimensions, editor_data.indent);
        },
        Event::KeyPressedChar(c, Modifiers::NONE) => {
            insert_character(c, editor_data);
            match move_cursor_right(editor_data) {
                Some(pos) => editor_data.file.cursor = pos,
                None => return
            };
            display_file(&editor_data.file, &editor_data.dimensions, editor_data.indent);
        },
        Event::KeyPressedOther(KeyCode::Enter, Modifiers::NONE) => {
            insert_newline(editor_data);
            editor_data.file.cursor.0 += 1;
            editor_data.file.cursor.1 = 1;
            display_file(&editor_data.file, &editor_data.dimensions, editor_data.indent);
        },
        Event::KeyPressedOther(KeyCode::Backspace, Modifiers::NONE) => {
            delete_character(editor_data);
            match move_cursor_left(editor_data) {
                Some(pos) => editor_data.file.cursor = pos,
                None => return
            };
            display_file(&editor_data.file, &editor_data.dimensions, editor_data.indent);
        },
        Event::KeyPressedOther(KeyCode::Tab, Modifiers::NONE) => {
            editor_data.special_char_command.clear();
            editor_data.state = EditorState::InsertSpecialChar;
            display_command_bar(&editor_data.special_char_command, &editor_data.dimensions);
        },
        Event::WindowResize(cols, rows) => {
            editor_data.dimensions = (cols, rows);
            editor_data.indent = (editor_data.dimensions.0 / 10).clamp(1, 4);
            display_file(&editor_data.file, &editor_data.dimensions, editor_data.indent);
        },
        _ => ()
    };
}

pub fn handle_event_in_char_insertion(event: Event, editor_data: &mut EditorData) {
    match event {
        Event::KeyPressedOther(KeyCode::Enter, Modifiers::NONE) => {
            editor_data.state = EditorState::EditingFile;
            
            // insert_character(c, editor_data);
            // match move_cursor_right(editor_data) {
            //     Some(pos) => editor_data.file.cursor = pos,
            //     None => return
            // };
            
            display_file(&editor_data.file, &editor_data.dimensions, editor_data.indent);
        },
        Event::KeyPressedChar(c, Modifiers::NONE) => {
            editor_data.special_char_command.push(c);
            display_command_bar(&editor_data.special_char_command, &editor_data.dimensions);
        },
        Event::KeyPressedOther(KeyCode::Backspace, Modifiers::NONE) => {
            editor_data.special_char_command.pop();
            display_command_bar(&editor_data.special_char_command, &editor_data.dimensions);
        },
        Event::WindowResize(cols, rows) => {
            editor_data.dimensions = (cols, rows);
            editor_data.indent = (editor_data.dimensions.0 / 10).clamp(1, 4);
            display_file(&editor_data.file, &editor_data.dimensions, editor_data.indent);
            display_command_bar(&editor_data.special_char_command, &editor_data.dimensions);
        },
        _ => ()
    };
}
