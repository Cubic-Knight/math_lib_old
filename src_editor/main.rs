use termwiz::{
    caps::Capabilities,
    terminal::{new_terminal, Terminal},
    Error
};

mod graphics;
use graphics::display_menu;

mod parsing;

mod library_data;
use library_data::read_lib_data;

mod events;
use events::{
    poll_terminal_for_events,
    EditorData, EditorState,
    handle_event_in_menu,
    handle_event_in_file_edition,
    handle_event_in_char_insertion
};

fn main() -> Result<(), Error> {
    let caps = Capabilities::new_from_env()?;
    let mut terminal = new_terminal(caps)?;
    terminal.set_raw_mode()?;

    let mut editor_data = EditorData::default();
    let (lib_data, references) = read_lib_data().unwrap();
    editor_data.lib_data = lib_data;
    editor_data.references = references;

    print!("\x1b[?25l");  // Hides the cursor
    display_menu(&editor_data.menu, &editor_data.dimensions);

    loop {
        let Some(event) = poll_terminal_for_events(&mut terminal)? else {
            continue;
        };
        match editor_data.state {
            EditorState::InMenu => handle_event_in_menu(event, &mut editor_data),
            EditorState::EditingFile => handle_event_in_file_edition(event, &mut editor_data),
            EditorState::InsertSpecialChar => handle_event_in_char_insertion(event, &mut editor_data),
            EditorState::ShouldExit => break
        };
        match editor_data.state {
            EditorState::ShouldExit => break,
            _ => ()
        }
    };
    Ok(())
}
