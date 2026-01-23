use std::{
    i16,
    time::{Duration, Instant},
};
use log;
use crate::{engine::SearchEngine};

use chess::{BoardStatus, ChessMove, Game, MoveGen};

use crate::engine::{EvaluateEngine, GameState, TimeInfo};

#[derive(Default)]
pub struct MCTSNode {
    state : GameState,
    visits : u16,
    score : i16,
    children : Vec<usize>,
    is_explored : bool
}

impl MCTSNode {
    // Calculate UCT value for this node
    fn uct(&self, parent_visits: u16) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let exploitation = self.score as f32  / self.visits as f32;
        let exploration = 20.0 * ((parent_visits as f32 + 1.0).ln() / self.visits as f32).sqrt();
        exploitation + exploration
    }
}


pub struct MCTS {
    nodes: Vec<MCTSNode>,
    nodes_explored: usize,
    pub root_moves : Vec<ChessMove>,
    selected_branch : Vec<usize>,
    selected_score : i16
}

impl MCTS {
    pub fn new(root : GameState) -> Self {
        let mut mcts = Self {
            nodes: vec![MCTSNode {state : root, ..Default::default()}],
            nodes_explored: 0,
            root_moves: Vec::<ChessMove>::new(),
            selected_branch : Vec::<usize>::new(),
            selected_score: 0
        };
        mcts.root_moves = mcts.explore(0);
        mcts
    }

    fn add_child(&mut self, parent: usize, node : MCTSNode) -> usize {
        let index = self.nodes.len();
        self.nodes.push(node);
        self.nodes[parent].children.push(index);
        index
    }

    fn explore(&mut self, id : usize) -> Vec<ChessMove> {
        self.nodes_explored += 1;
        let board = self.nodes[id].state.last_board();
        let mut legal_moves = MoveGen::new_legal(&board);
        //let targets = board.color_combined(!board.side_to_move());
        //legal_moves.set_iterator_mask(*targets);

        let mut moves_vec = Vec::new();
        for mv in &mut legal_moves {
            moves_vec.push(mv);
            self.nodes[id].state.make_move(mv);
            self.add_child(id, MCTSNode {state : self.nodes[id].state.clone(), ..Default::default()});
            self.nodes[id].state.undo_last_move();
        }
        self.nodes[id].is_explored = true;
        moves_vec
    }

    // Select the best child using UCT
    fn select_best_child(&self, id : usize) -> usize {
        let visits = self.nodes[id].visits;
        match self.nodes[id].children
            .iter()
            .max_by(|a, b| {
                self.nodes[**a].uct(visits)
                    .partial_cmp(&self.nodes[**b].uct(visits))
                    .unwrap()
            })
            {
                Some(n) => *n,
                None => 0
            }
    }

    fn select(&mut self) {
        self.selected_branch.clear();
        self.selected_branch.push(0);
        let mut current : usize = 0;
        while self.nodes[current].is_explored {
            current = self.select_best_child(current);
            self.selected_branch.push(current);
        }
    }

    fn expand(&mut self) {
        let leaf : usize = match self.selected_branch.last() {Some(n) => *n, None => 0};
        if self.nodes[leaf].state.last_board().status() == BoardStatus::Ongoing {
            self.explore(leaf);
            self.selected_branch.push(self.select_best_child(leaf));
        }
    }

    fn evaluate<E : EvaluateEngine>(&mut self, evaluator : &mut E) {
        let leaf : usize = match self.selected_branch.last() {Some(n) => *n, None => 0};
        let state = &self.nodes[leaf].state;
        let root_state = &self.nodes[0].state;
        self.selected_score = (if state.turn() == root_state.turn() {1} else {-1}) * (*evaluator).evaluate(&state).unwrap();
    }

    fn backpropagate(&mut self) {
        for node in &mut self.selected_branch {
            self.nodes[*node].visits += 1;
            self.nodes[*node].score = (self.nodes[*node].score as i32 + self.selected_score as i32).min(i16::MAX as i32 / 2).max(-i16::MAX as i32 / 2)  as i16;
        }
    }

    pub fn root_scores(&self) -> Vec::<i16> {
        self.nodes[0].children.iter().map(|n| self.nodes[*n].score).collect()
    }

    pub fn mcts_step<E : EvaluateEngine>(&mut self, evaluator : &mut E) {
        self.select();
        self.expand();
        self.evaluate::<E>(evaluator);
        self.backpropagate();
    }
}


pub struct MCTSEngine<E : EvaluateEngine> {
    evaluator : E
}

impl<E : EvaluateEngine> MCTSEngine<E>{
    pub fn new(evaluator : E) -> Self {
        Self {evaluator : evaluator}
    }
}

impl<E: EvaluateEngine> SearchEngine<E> for MCTSEngine<E> {
    fn next_move(
        &mut self,
        mut state: GameState,
        time_info: TimeInfo
    ) -> Option<ChessMove> {
        let start = Instant::now();
                
        let mut tree_search = MCTS::new(state);
        tree_search.nodes_explored = 0;
        for _i in 0..4000 {
            tree_search.mcts_step::<E>(&mut self.evaluator)
        }
        let argmax: Option<usize> = tree_search.root_scores()
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(index, _)| index);

        let best_move = match argmax {Some(n) => tree_search.root_moves[n], None => tree_search.root_moves[0]};

        let elapsed = Instant::now() - start;
        log::info!(
            "Nodes explored: {} in {}ms. {:.0} NPS",
            tree_search.nodes_explored,
            elapsed.as_millis(),
            (tree_search.nodes_explored as f64 / elapsed.as_secs_f64()).round()
        );
        log::info!("Root scores : {:?}", tree_search.root_scores());
        Some(best_move)
    }

    fn clear_search_state(&mut self) {
        
    }
}


//    pub fn search_eval<E: EvaluateEngine>(
//        &mut self,
//        state: &mut GameState,
//        _time_info: &Option<TimeInfo>,
//        mut alpha: i16,
//        beta: i16,
//        depth: u16,
//    ) -> i16 """
