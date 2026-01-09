use chess::{BitBoard, Board, ChessMove, EMPTY, MoveGen};

pub struct MvIter<'a> {
    explore_first: &'a [Option<ChessMove>],
    explore_first_idx: usize,
    legal_moves: MoveGen,
    flags: [BitBoard; 2],
    flag_idx: usize,
}

impl<'a> MvIter<'a> {
    pub fn new(explore_first: &'a [Option<ChessMove>], board: &Board) -> Self {
        Self {
            explore_first,
            explore_first_idx: 0,
            legal_moves: MoveGen::new_legal(board),
            flags: [board.color_combined(!board.side_to_move()).clone(), !EMPTY],
            flag_idx: 0,
        }
    }
}

impl<'a> Iterator for MvIter<'a> {
    type Item = ChessMove;
    fn next(&mut self) -> Option<Self::Item> {
        if self.explore_first_idx < self.explore_first.len() {
            let mv = self.explore_first.get(self.explore_first_idx).cloned();
            self.explore_first_idx += 1;
            if let Some(Some(mv)) = mv {
                self.legal_moves.remove_move(mv);
                return Some(mv)
            }
            self.next()
        } else if let Some(mv) = self.legal_moves.next() {
            Some(mv)
        } else if self.flag_idx + 1 < self.flags.len() {
            self.flag_idx += 1;
            self.legal_moves.set_iterator_mask(self.flags[self.flag_idx]);
            self.next()
        } else {
            None
        }
    }
}
