mod event_handling;
pub use event_handling::{
    handle_event_in_menu,
    handle_event_in_file_edition,
    handle_event_in_char_insertion,
    EditorData, EditorState
};

mod polling;
pub use polling::{
    poll_terminal_for_events,
    Event, Direction
};

mod cursor_movement;
use cursor_movement::{
    move_cursor_up,
    move_cursor_down,
    move_cursor_left,
    move_cursor_right
};

mod character_insertion;
use character_insertion::{
    insert_character,
    insert_newline,
    delete_character
};
