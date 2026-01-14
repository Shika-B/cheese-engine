use chess::{Board, ChessMove, MoveGen};

pub struct StagedMoveIterator {
    board: Board,
    stage: MoveStage,

    // Phase 1: TT move
    tt_move: Option<ChessMove>,
    tt_move_yielded: bool,

    // Phase 2: Good captures (MVV-LVV sorted, SEE >= 0)
    good_captures: Vec<ChessMove>,
    good_captures_idx: usize,

    // Phase 3-4: Killer moves
    killer_moves: [Option<ChessMove>; 2],
    killer_idx: usize,

    // Phase 5: Counter move
    counter_move: Option<ChessMove>,
    counter_yielded: bool,

    // Phase 6: Quiet moves (history sorted)
    quiet_moves: Vec<(ChessMove, i32)>,
    quiet_idx: usize,

    // Phase 7: Bad captures (SEE < 0)
    bad_captures: Vec<ChessMove>,
    bad_captures_idx: usize,

    // Reference to history table (raw pointer for efficiency)
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
        Self {
            board,
            stage: MoveStage::TTMove,
            tt_move,
            tt_move_yielded: false,
            good_captures: Vec::with_capacity(32),
            good_captures_idx: 0,
            killer_moves: *killer_moves,
            killer_idx: 0,
            counter_move,
            counter_yielded: false,
            quiet_moves: Vec::with_capacity(64),
            quiet_idx: 0,
            bad_captures: Vec::with_capacity(32),
            bad_captures_idx: 0,
            history_table_ptr: history_table as *const _,
        }
    }

    fn mvv_lvv_score(mv: ChessMove, board: &Board) -> i16 {
        const PIECE_VALUES: [i16; 6] = [100, 320, 330, 500, 900, 0]; // P, N, B, R, Q, K

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

        // Generate all captures
        let captures = MoveGen::new_legal(&self.board);

        for mv in captures {
            // Check if it's a capture
            if let Some(_) = self.board.piece_on(mv.get_dest()) {
                // Use simplified SEE to separate good/bad captures
                if self.see_simple(mv, 0) {
                    self.good_captures.push(mv);
                } else {
                    self.bad_captures.push(mv);
                }
            }
        }

        // Sort good captures by MVV-LVV descending
        self.good_captures.sort_unstable_by_key(|&mv| -Self::mvv_lvv_score(mv, &self.board));
        self.good_captures_idx = 0;
    }

    fn see_simple(&self, mv: ChessMove, threshold: i16) -> bool {
        // Simplified SEE for move ordering
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

        // Generate all legal moves
        let all_moves = MoveGen::new_legal(&self.board);

        // Get history scores (safe because we own the reference during iteration)
        let history_table = unsafe { &*self.history_table_ptr };

        for mv in all_moves {
            // Skip captures (already processed)
            if self.board.piece_on(mv.get_dest()).is_some() {
                continue;
            }

            // Skip moves already yielded (tt_move, killers, counter)
            if Some(mv) == self.tt_move
                || Some(mv) == self.killer_moves[0]
                || Some(mv) == self.killer_moves[1]
                || Some(mv) == self.counter_move
            {
                continue;
            }

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

    fn is_legal(&self, mv: ChessMove) -> bool {
        // Quick legality check - ensure the move is actually legal for this position
        MoveGen::new_legal(&self.board).any(|legal_mv| legal_mv == mv)
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
                        if self.is_legal(mv) {
                            self.tt_move_yielded = true;
                            return Some(mv);
                        }
                    }
                }

                MoveStage::GenerateCaptures => {
                    self.generate_captures();
                    self.stage = MoveStage::GoodCaptures;
                }

                MoveStage::GoodCaptures => {
                    while self.good_captures_idx < self.good_captures.len() {
                        let mv = self.good_captures[self.good_captures_idx];
                        self.good_captures_idx += 1;

                        // Skip if this is the TT move (already yielded)
                        if self.tt_move_yielded && Some(mv) == self.tt_move {
                            continue;
                        }

                        return Some(mv);
                    }
                    self.stage = MoveStage::Killer1;
                }

                MoveStage::Killer1 => {
                    self.stage = MoveStage::Killer2;
                    if let Some(mv) = self.killer_moves[0] {
                        // Check it's quiet, legal, and not the TT move
                        if self.board.piece_on(mv.get_dest()).is_none()
                            && self.is_legal(mv)
                            && !(self.tt_move_yielded && Some(mv) == self.tt_move)
                        {
                            return Some(mv);
                        }
                    }
                }

                MoveStage::Killer2 => {
                    self.stage = MoveStage::CounterMove;
                    if let Some(mv) = self.killer_moves[1] {
                        // Check it's quiet, legal, different from killer1, and not the TT move
                        if self.board.piece_on(mv.get_dest()).is_none()
                            && self.is_legal(mv)
                            && Some(mv) != self.killer_moves[0]
                            && !(self.tt_move_yielded && Some(mv) == self.tt_move)
                        {
                            return Some(mv);
                        }
                    }
                }

                MoveStage::CounterMove => {
                    self.stage = MoveStage::GenerateQuiet;
                    if let Some(mv) = self.counter_move {
                        // Check it's quiet, legal, not a killer, and not the TT move
                        if self.board.piece_on(mv.get_dest()).is_none()
                            && self.is_legal(mv)
                            && Some(mv) != self.killer_moves[0]
                            && Some(mv) != self.killer_moves[1]
                            && !(self.tt_move_yielded && Some(mv) == self.tt_move)
                        {
                            self.counter_yielded = true;
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
                    while self.bad_captures_idx < self.bad_captures.len() {
                        let mv = self.bad_captures[self.bad_captures_idx];
                        self.bad_captures_idx += 1;

                        // Skip if this is the TT move (already yielded)
                        if self.tt_move_yielded && Some(mv) == self.tt_move {
                            continue;
                        }

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
