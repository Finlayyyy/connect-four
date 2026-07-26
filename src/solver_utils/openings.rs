use crate::board::*;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use crate::solver_utils::Cache;
use crate::solver_utils::EntryKind;

const FILE_PATH: &str = "openings.txt";

struct Openings {
    openings: Vec<(Moves, isize)>,
}

impl Openings {
    pub fn new() -> Self {
        Openings { openings: vec![] }
    }

    pub fn push(&mut self, moves: Moves, score: isize) {
        if self.openings.iter().any(|(m, s)| moves == *m) {
            return;
        }

        self.openings.push((moves, score))
    }

    pub fn read() -> Self {
        let contents = fs::read_to_string(FILE_PATH)
            .expect(format!("Could not read file: {}", FILE_PATH).as_str());

        let mut openings = Self::new();

        for line in contents.lines() {
            let test = line.split_whitespace().collect::<Vec<&str>>();
            assert!(test.len() == 2);

            let moves = test[0];
            let score = test[1];

            assert!(moves.chars().all(|c| c.is_digit(10)));
            let moves = Moves::from_string(moves);

            match score.parse::<isize>() {
                Err(_) => panic!("Invalid token in moves file: {}", score),
                Ok(n) => openings.push(moves.to_owned(), n),
            }
        }

        openings
    }

    pub fn write(&self) {
        let mut file = File::open(FILE_PATH).unwrap();
        file.write_all(
            self.openings.iter()
                .map(|(moves, score)| format!("{} {}", moves.to_string(), score))
                .collect::<Vec<_>>()
                .join("\n")
                .as_bytes()
        ).unwrap();
    }

    pub fn to_cache(&self) -> Cache {
        let mut cache = Cache::new(Cache::LARGE_SIZE);
        for (moves, score) in &self.openings {
            let board = BitBoard::from_moves(moves);
            cache.insert(EntryKind::Exact, &board, *score);
        }
        cache
    }
}

impl Drop for Openings {
    fn drop(&mut self) {
        self.write();
    }
}

pub fn write_opening(moves: Moves, score: isize) {
    let file = File::open(FILE_PATH).unwrap();

}
