use termwiz::{
    input::{
        InputEvent,
        KeyEvent, KeyCode, Modifiers,
        MouseEvent, MouseButtons
    },
    terminal::Terminal,
    Error
};

pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

pub enum Event {
    KeyPressedChar(char, Modifiers),
    KeyPressedArrow(Direction, Modifiers),
    KeyPressedOther(KeyCode, Modifiers),
    MouseMoved(u16, u16, MouseButtons, Modifiers),
    WindowResize(usize, usize),
    KeyboardInterrupt
}

pub fn poll_terminal_for_events(terminal: &mut impl Terminal) -> Result<Option<Event>, Error> {
    let Some(event) = terminal.poll_input(None)? else {
        return Ok(None);
    };
    let result = match event {
        InputEvent::Key(
            KeyEvent {
                key: KeyCode::Char('c'),
                modifiers: Modifiers::CTRL
            }
        ) => Some(Event::KeyboardInterrupt),
        InputEvent::Key(
            KeyEvent { key, modifiers }
        ) => match key {
            KeyCode::Char(c) => Some(Event::KeyPressedChar(c, modifiers)),

            // Arrow keys
            KeyCode::UpArrow => Some(Event::KeyPressedArrow(Direction::Up, modifiers)),
            KeyCode::DownArrow => Some(Event::KeyPressedArrow(Direction::Down, modifiers)),
            KeyCode::LeftArrow => Some(Event::KeyPressedArrow(Direction::Left, modifiers)),
            KeyCode::RightArrow => Some(Event::KeyPressedArrow(Direction::Right, modifiers)),

            // Numpad keys
            KeyCode::Numpad0 =>   Some(Event::KeyPressedChar('0', modifiers)),
            KeyCode::Numpad1 =>   Some(Event::KeyPressedChar('1', modifiers)),
            KeyCode::Numpad2 =>   Some(Event::KeyPressedChar('2', modifiers)),
            KeyCode::Numpad3 =>   Some(Event::KeyPressedChar('3', modifiers)),
            KeyCode::Numpad4 =>   Some(Event::KeyPressedChar('4', modifiers)),
            KeyCode::Numpad5 =>   Some(Event::KeyPressedChar('5', modifiers)),
            KeyCode::Numpad6 =>   Some(Event::KeyPressedChar('6', modifiers)),
            KeyCode::Numpad7 =>   Some(Event::KeyPressedChar('7', modifiers)),
            KeyCode::Numpad8 =>   Some(Event::KeyPressedChar('8', modifiers)),
            KeyCode::Numpad9 =>   Some(Event::KeyPressedChar('9', modifiers)),
            KeyCode::Multiply =>  Some(Event::KeyPressedChar('*', modifiers)),
            KeyCode::Add =>       Some(Event::KeyPressedChar('+', modifiers)),
            KeyCode::Separator => Some(Event::KeyPressedChar(',', modifiers)),
            KeyCode::Subtract =>  Some(Event::KeyPressedChar('-', modifiers)),
            KeyCode::Decimal =>   Some(Event::KeyPressedChar('.', modifiers)),
            KeyCode::Divide =>    Some(Event::KeyPressedChar('/', modifiers)),

            other => Some(Event::KeyPressedOther(other, modifiers))
        },
        InputEvent::Mouse(
            MouseEvent {
                x, y, mouse_buttons, modifiers
            }
        ) => Some(Event::MouseMoved(x, y, mouse_buttons, modifiers)),
        InputEvent::Resized {
            cols, rows
        } => Some(Event::WindowResize(cols, rows)),
        _ => None
    };
    Ok(result)
}
