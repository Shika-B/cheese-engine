import numpy as np
import chess
import torch.nn.functional as F
from torch import FloatTensor

INPUT_SIZE = 2*6*64 + 4 + 16 + 1   # 2 players * 6 ^piece_types * 64 squares + 4 castling rights + 16 possible squares for en_passant + 1 who to play 
HIDDEN_SIZE = 256
EVAL_SCALE = 400
QA = 255
QB = 64

def piece_index(piece_type, color, square):
    return 64 * (piece_type - 1 + (6 * color)) + square

def fen_to_onehot(fen : str):
    b = chess.Board()
    b.set_fen(fen)
    board = np.zeros((INPUT_SIZE, ), dtype=int)     
    
    # position
    for square, piece in b.piece_map().items():
        board[piece_index(piece.piece_type, piece.color, square)] = 1
    
    # castling
    board[768] = b.has_kingside_castling_rights(chess.WHITE)
    board[769] = b.has_queenside_castling_rights(chess.WHITE)
    board[770] = b.has_kingside_castling_rights(chess.BLACK)
    board[771] = b.has_queenside_castling_rights(chess.BLACK)

    # en passant
    en_passant = b.ep_square
    if not (en_passant is None): 
        board[772 + ((en_passant - 16) if (16 <= en_passant <= 23) else (en_passant - 40 + 8))] = 1  # en passant squares are on row 3 or 6, so reduce the square interval from 0,63 to 0,17

    # turn
    board[788] = int(b.turn)

    return board

def clamp(x):
    return max(0, min(x, 1))


class NNUE:
    """
    NNUE for Inference
    """
    def __init__(self):
        self.params = {
            'acc_weights' : np.random.random((HIDDEN_SIZE, INPUT_SIZE)),
            'acc_biases' : np.random.random((HIDDEN_SIZE,)),
            'output_weights' : np.random.random((2*HIDDEN_SIZE)),
            'output_bias' : 0
        }   
        self.QA = self.QB = 1
        
        self.values = {
            "prev_input" : np.random.random((INPUT_SIZE,)),
            "input" : np.random.random((INPUT_SIZE,)),
            "acc" : np.random.random((HIDDEN_SIZE,))
        }

        self.quantized = False

    def quantize(self):
        if self.quantized:
            raise ValueError("Network is already quantized")
        
        self.params["acc_weights"] = np.rint(QA * self.params["acc_weights"])
        self.params["acc_biases"] = np.rint(QA * self.params["acc_biases"])
        self.params["output_weights"] = np.rint(QB * self.params["output_weights"])
        self.params["output_bias"] = np.rint(QA * QA * QB * self.params["output_bias"])

        self.QA = QA
        self.QB = QB
        self.quantized = True

    def newly_switched_on(self):
        return [i for i in range(INPUT_SIZE) if self.values["prev_input"][i] == 0 and self.values["input"][i] == 1]

    def newly_switched_off(self):
        return [i for i in range(INPUT_SIZE) if self.values["prev_input"][i] == 1 and self.values["input"][i] == 0]
    
    def update_acc_index_add(self, k):
        for i in range(HIDDEN_SIZE):
            self.values["acc"][i] += self.params['acc_weights'][i][k]
        
        self.values["acc"] += self.params['acc_weights'].T[i]
    
    def update_acc_index_substract(self, k):
        for i in range(HIDDEN_SIZE):
            self.values["acc"][i] -= self.params['acc_weights'][i][k]
    
    def update_acc(self):
        for i in self.newly_switched_on():
            self.update_acc_index_add(i)
        
        for i in self.newly_switched_off():
            self.update_acc_index_substract(i)

    def get_true_acc(self):
        return np.concat([self.values['acc'], -self.values['acc']])

    def get_evaluation(self):
        activated_acc_values = np.square(np.clip(self.get_true_acc(), 0, self.QA))
        return F.linear(FloatTensor(activated_acc_values), FloatTensor(self.params["output_weights"]), FloatTensor(self.params["output_bias"]))

    def first_forward(self, board):
        self.values['input'] = board
        self.values['acc'] = F.linear(FloatTensor(board), FloatTensor(self.params['acc_weights']), FloatTensor(self.params['acc_biases']))
        eval = self.get_evaluation()
        return eval / self.QA * self.QA * self.QB

    def forward(self, board):
        self.values['prev_input'] = self.values['input']
        self.values['input'] = board
        self.update_acc()
        eval = self.get_evaluation()
        return eval / self.QA * self.QB

if __name__ == '__main__':
    print(fen_to_onehot("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"))
    quit()
    model = NNUE()
    r = model.forward(fen_to_onehot("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"))
    print(r)