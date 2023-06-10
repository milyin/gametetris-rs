mod frequency_regulator;
mod tetris;
mod tetris_pair;

pub use frequency_regulator::FrequencyRegulator;
pub use tetris::Tetris;
pub use tetris_pair::TetrisPair;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
