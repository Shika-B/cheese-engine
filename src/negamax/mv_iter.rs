use chess::{Board, ChessMove, MoveGen};

pub struct StagedMoveIterator {
    board: Board,
    stage: MoveStage,

    move_gen: MoveGen,

    tt_move: Option<ChessMove>,

    good_captures: Vec<ChessMove>,
    good_captures_idx: usize,

    killer_moves: [Option<ChessMove>; 2],

    counter_move: Option<ChessMove>,

    quiet_moves: Vec<(ChessMove, i32)>,
    quiet_idx: usize,

    bad_captures: Vec<ChessMove>,
    bad_captures_idx: usize,

    history_table_ptr: *const [[i32; 64]; 64],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveStage {
    TTMove,
    GenerateCaptures,
    GoodCaptures,
    Killer1,
    Killer2,
    CounterMove,
    GenerateQuiet,
    QuietMoves,
    BadCaptures,
    Done,
}

impl StagedMoveIterator {
    pub fn new(
        board: Board,
        tt_move: Option<ChessMove>,
        killer_moves: &[Option<ChessMove>; 2],
        counter_move: Option<ChessMove>,
        history_table: &[[i32; 64]; 64],
    ) -> Self {
        let move_gen = MoveGen::new_legal(&board);
        Self {
            board,
            stage: MoveStage::TTMove,
            move_gen,
            tt_move,
            good_captures: Vec::with_capacity(32),
            good_captures_idx: 0,
            killer_moves: *killer_moves,
            counter_move,
            quiet_moves: Vec::with_capacity(64),
            quiet_idx: 0,
            bad_captures: Vec::with_capacity(32),
            bad_captures_idx: 0,
            history_table_ptr: history_table as *const _,
        }
    }

    fn mvv_lvv_score(mv: ChessMove, board: &Board) -> i16 {
        // P, N, B, R, Q, K
        const PIECE_VALUES: [i16; 6] = [100, 320, 330, 500, 900, 0];

        let victim = board.piece_on(mv.get_dest());
        let attacker = board.piece_on(mv.get_source());

        if let Some(victim_piece) = victim {
            let victim_value = PIECE_VALUES[victim_piece.to_index()];
            let attacker_value = if let Some(attacker_piece) = attacker {
                PIECE_VALUES[attacker_piece.to_index()]
            } else {
                0
            };
            return victim_value * 16 - attacker_value;
        }
        0
    }

    fn generate_captures(&mut self) {
        self.good_captures.clear();
        self.bad_captures.clear();

        self.move_gen.set_iterator_mask(*self.board.combined());

        // Collect all capture moves first
        let captures: Vec<ChessMove> = (&mut self.move_gen).collect();

        // Now classify them using SEE
        for mv in captures {
            // Use simplified SEE to separate good/bad captures
            if self.see_simple(mv, 0) {
                self.good_captures.push(mv);
            } else {
                self.bad_captures.push(mv);
            }
        }

        // Sort good captures by MVV-LVV descending
        self.good_captures
            .sort_unstable_by_key(|&mv| -Self::mvv_lvv_score(mv, &self.board));
        self.good_captures_idx = 0;
    }

    /// Check if a move is legal without panicking
    /// This is safer than MoveGen::legal_quick for moves from other positions
    fn is_move_legal(&self, mv: ChessMove) -> bool {
        // Basic validation first
        if self.board.piece_on(mv.get_source()).is_none() {
            return false;
        }

        // For killer/counter moves, use the safer approach of checking
        // if the move is in the legal move list
        // This is slower but prevents panics from invalid moves
        for legal_mv in MoveGen::new_legal(&self.board) {
            if legal_mv == mv {
                return true;
            }
        }
        false
    }

    /// Static exchange evaluation
    fn see_simple(&self, mv: ChessMove, threshold: i16) -> bool {
        const PIECE_VALUES: [i16; 6] = [100, 320, 330, 500, 900, 0];

        let victim = self.board.piece_on(mv.get_dest());
        let attacker = self.board.piece_on(mv.get_source());

        if victim.is_none() {
            return threshold <= 0;
        }

        if attacker.is_none() {
            return false;
        }

        let victim_value = PIECE_VALUES[victim.unwrap().to_index()];
        let attacker_value = PIECE_VALUES[attacker.unwrap().to_index()];

        // Simple: capture is good if victim >= attacker
        (victim_value - attacker_value) >= threshold
    }

    fn generate_quiet(&mut self) {
        self.quiet_moves.clear();

        // Set mask to exclude captures (use NOT of combined pieces)
        self.move_gen.set_iterator_mask(!*self.board.combined());

        // Get history scores (safe because we own the reference during iteration)
        let history_table = unsafe { &*self.history_table_ptr };
 
        for mv in &mut self.move_gen {
            // Get history score
            let from = mv.get_source().to_index();
            let to = mv.get_dest().to_index();
            let history_score = history_table[from][to];

            self.quiet_moves.push((mv, history_score));
        }

        // Sort by history score descending
        self.quiet_moves.sort_unstable_by_key(|(_, score)| -score);
        self.quiet_idx = 0;
    }

}

impl Iterator for StagedMoveIterator {
    type Item = ChessMove;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stage {
                MoveStage::TTMove => {
                    self.stage = MoveStage::GenerateCaptures;
                    if let Some(mv) = self.tt_move {
                        if self.is_move_legal(mv) {
                            // Remove from move generator so it won't appear again
                            self.move_gen.remove_move(mv);
                            return Some(mv);
                        }
                    }
                }

                MoveStage::GenerateCaptures => {
                    self.generate_captures();
                    self.stage = MoveStage::GoodCaptures;
                }

                MoveStage::GoodCaptures => {
                    if self.good_captures_idx < self.good_captures.len() {
                        let mv = self.good_captures[self.good_captures_idx];
                        self.good_captures_idx += 1;
                        return Some(mv);
                    }
                    self.stage = MoveStage::Killer1;
                }

                MoveStage::Killer1 => {
                    self.stage = MoveStage::Killer2;
                    if let Some(mv) = self.killer_moves[0] {
                        // Check it's quiet and legal
                        if self.board.piece_on(mv.get_dest()).is_none()
                            && self.is_move_legal(mv)
                        {
                            // Remove from move generator so it won't appear again
                            self.move_gen.remove_move(mv);
                            return Some(mv);
                        }
                    }
                }

                MoveStage::Killer2 => {
                    self.stage = MoveStage::CounterMove;
                    if let Some(mv) = self.killer_moves[1] {
                        // Check it's quiet, legal, and different from killer1
                        if self.board.piece_on(mv.get_dest()).is_none()
                            && Some(mv) != self.killer_moves[0]
                            && self.is_move_legal(mv)
                        {
                            // Remove from move generator so it won't appear again
                            self.move_gen.remove_move(mv);
                            return Some(mv);
                        }
                    }
                }

                MoveStage::CounterMove => {
                    self.stage = MoveStage::GenerateQuiet;
                    if let Some(mv) = self.counter_move {
                        // Check it's quiet, legal, and not a killer
                        if self.board.piece_on(mv.get_dest()).is_none()
                            && Some(mv) != self.killer_moves[0]
                            && Some(mv) != self.killer_moves[1]
                            && self.is_move_legal(mv)
                        {
                            // Remove from move generator so it won't appear again
                            self.move_gen.remove_move(mv);
                            return Some(mv);
                        }
                    }
                }

                MoveStage::GenerateQuiet => {
                    self.generate_quiet();
                    self.stage = MoveStage::QuietMoves;
                }

                MoveStage::QuietMoves => {
                    while self.quiet_idx < self.quiet_moves.len() {
                        let (mv, _score) = self.quiet_moves[self.quiet_idx];
                        self.quiet_idx += 1;
                        return Some(mv);
                    }
                    self.stage = MoveStage::BadCaptures;
                }

                MoveStage::BadCaptures => {
                    if self.bad_captures_idx < self.bad_captures.len() {
                        let mv = self.bad_captures[self.bad_captures_idx];
                        self.bad_captures_idx += 1;
                        return Some(mv);
                    }
                    self.stage = MoveStage::Done;
                }

                MoveStage::Done => {
                    return None;
                }
            }
        }
    }
}
