use super::EditorData;

fn get_len_of_file_line(editor_data: &EditorData, index: usize) -> usize {
    editor_data.file.lines.get(index-1)
        .map(|line| line.chars.len() + 1)
        .unwrap_or(1)
}

pub fn move_cursor_up(editor_data: &EditorData) -> Option<(usize, usize)> {
    let (cy, cx) = editor_data.file.cursor;
    match (cy, cx) {
        (1, 1) => return None,
        (1, _) => return Some((1, 1)),
        _ => ()
    };
    let line_period = editor_data.dimensions.0 - editor_data.indent - 1;
    let cursor_height = cx.saturating_sub(editor_data.indent + 1).div_euclid(line_period);
    if cursor_height == 0 {
        let prev_line_len = get_len_of_file_line(editor_data, cy-1);
        let prev_line_height = prev_line_len
            .saturating_sub(editor_data.indent + 1)
            .div_euclid(line_period);
        let x = if prev_line_height == 0 {
            cx
        } else {
            cx.max(editor_data.indent + 1) + prev_line_height * line_period
        };
        Some((cy-1, x.min(prev_line_len)))
    } else {
        Some((cy, cx-line_period))
    }
}

pub fn move_cursor_down(editor_data: &EditorData) -> Option<(usize, usize)> {
    let (cy, cx) = editor_data.file.cursor;
    let line_len = get_len_of_file_line(editor_data, cy);
    let file_len = editor_data.file.lines.len();
    if (cy, cx) == (file_len, line_len) { return None; };
    if cy == file_len { return Some((file_len, line_len)); };
    let line_period = editor_data.dimensions.0 - editor_data.indent - 1;
    let line_height = line_len.saturating_sub(editor_data.indent + 1).div_euclid(line_period);
    let cursor_height = cx.saturating_sub(editor_data.indent + 1).div_euclid(line_period);
    if cursor_height == line_height {
        let next_line_len = get_len_of_file_line(editor_data, cy+1);
        let x = cx - cursor_height * line_period;
        Some((cy+1, x.min(next_line_len)))
    } else {
        let x = cx.max(editor_data.indent + 1) + line_period;
        Some((cy, x.min(line_len)))
    }
}

pub fn move_cursor_left(editor_data: &EditorData) -> Option<(usize, usize)> {
    let (cy, cx) = editor_data.file.cursor;
    if (cy, cx) == (1, 1) { return None; };
    match cx {
        1 => Some((cy-1, get_len_of_file_line(editor_data, cy-1))),
        _ => Some((cy, cx-1))
    }
}

pub fn move_cursor_right(editor_data: &EditorData) -> Option<(usize, usize)> {
    let (cy, cx) = editor_data.file.cursor;
    let line_len = get_len_of_file_line(editor_data, cy);
    if (cy, cx) == (editor_data.file.lines.len(), line_len) { return None; };
    if cx == line_len { Some((cy+1, 1)) } else { Some((cy, cx+1)) }
}
