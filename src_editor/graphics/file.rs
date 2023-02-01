use std::collections::HashMap;
use std::fs;
use crate::parsing::{
    parse_file, FileLine
};
use crate::library_data::{
    LibraryData, Reference
};

#[derive(Default)]
pub struct FileGraphics {
    pub cursor: (usize, usize),
    pub camera: usize,
    pub lines: Vec<FileLine>,
    pub read_only: bool
}

pub fn get_file(
    path: String, lib_data: &LibraryData, references: &HashMap<String, Reference>
) -> Result<FileGraphics, ()> {
    let contents = fs::read_to_string(path).map_err(|_| ())?;
    let lines = contents.lines()
        .map(|s| s.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let file_lines = parse_file(lines, lib_data, references);

    Ok(
        FileGraphics {
            cursor: (1, 1),
            camera: 1,
            lines: file_lines,
            read_only: false
        }
    )
}
