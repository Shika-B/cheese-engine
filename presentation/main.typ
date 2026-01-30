#import "@preview/polylux:0.4.0": slide

// Page setup - landscape mode
#set page(
  paper: "presentation-16-9",
  margin: 1cm,
)

// Color theme
#let primary = rgb("#2563eb")
#let secondary = rgb("#7c3aed")
#let accent = rgb("#06b6d4")
#let dark = rgb("#1e293b")
#let light = rgb("#f8fafc")

// Text settings
#set text(size: 16pt, fill: dark)

// Custom slide template
#let title-slide(title, subtitle: none) = {
  set page(fill: gradient.linear(primary, secondary, angle: 135deg))
  set text(fill: white)
  slide[
    #align(center + horizon)[
      #text(size: 44pt, weight: "bold")[#title]
      #if subtitle != none {
        v(0.8em)
        text(size: 22pt, weight: "light")[#subtitle]
      }
    ]
  ]
}

#let section-slide(title) = {
  set page(fill: gradient.linear(secondary, accent, angle: 135deg))
  set text(fill: white)
  slide[
    #align(center + horizon)[
      #text(size: 38pt, weight: "bold")[#title]
    ]
  ]
}

#let content-slide(title, body) = {
  set page(fill: light)
  slide[
    #box(
      width: 100%,
      inset: (bottom: 0.5em),
      stroke: (bottom: 2pt + primary),
    )[
      #text(size: 28pt, weight: "bold", fill: primary)[#title]
    ]
    #v(0.3em)
    #body
  ]
}

// ============================================
// SLIDES
// ============================================

#title-slide("Cheese Engine", subtitle: "A Chess Engine with NNUE Evaluation")

// --------------------------------------------
// Slide 2: Chess as a Zero-Sum Game
// --------------------------------------------

#content-slide("Chess: A Zero-Sum Game")[
  #grid(
    columns: (1fr, 1fr),
    gutter: 1.5em,
    [
      #box(fill: rgb("#dbeafe"), radius: 6pt, inset: 0.7em, width: 100%)[
        #text(weight: "bold", fill: primary, size: 15pt)[What is Zero-Sum?]
        #v(0.2em)
        #text(size: 14pt)[
          One player's gain = other's loss. \
          White wins (+1), Black loses (-1). Total: zero.
        ]
      ]
      #v(0.4em)
      #box(fill: rgb("#fef3c7"), radius: 6pt, inset: 0.7em, width: 100%)[
        #text(weight: "bold", fill: rgb("#d97706"), size: 15pt)[Perfect Information]
        #v(0.2em)
        #text(size: 14pt)[
          Both players see entire board. \
          No hidden cards, no dice, no luck.
        ]
      ]
    ],
    [
      #box(fill: rgb("#fee2e2"), radius: 6pt, inset: 0.7em, width: 100%)[
        #text(weight: "bold", fill: rgb("#dc2626"), size: 15pt)[Why Can't We Solve It?]
        #v(0.2em)
        #text(size: 14pt)[
          - Legal positions: ~10#super[44]
          - Possible games: ~10#super[120] (Shannon Number)
          #v(0.2em)
          Compare: Checkers solved with 5×10#super[20] positions (2007)
        ]
      ]
      #v(0.4em)
      #align(center)[
        #box(fill: rgb("#f1f5f9"), radius: 6pt, inset: 0.5em)[
          #text(size: 13pt, fill: rgb("#64748b"))[
            We need smarter ways to explore the game tree
          ]
        ]
      ]
    ]
  )
]

// --------------------------------------------
// Slide 3: Exploring the Game Tree
// --------------------------------------------

#content-slide("Exploring the Game Tree")[
  #align(center)[
    #box(fill: rgb("#f1f5f9"), radius: 10pt, inset: 1em)[
      #text(size: 16pt)[We can't explore every position, so we need:]
      #v(0.4em)
      #grid(
        columns: (1fr, 1fr),
        gutter: 2em,
        [
          #box(fill: gradient.linear(primary, rgb("#3b82f6")), radius: 6pt, inset: 0.8em)[
            #text(fill: white, weight: "bold", size: 20pt)[Search Algorithm]
            #v(0.2em)
            #text(fill: white, size: 14pt)[How to traverse the tree efficiently]
          ]
        ],
        [
          #box(fill: gradient.linear(secondary, rgb("#8b5cf6")), radius: 6pt, inset: 0.8em)[
            #text(fill: white, weight: "bold", size: 20pt)[Evaluation Function]
            #v(0.2em)
            #text(fill: white, size: 14pt)[How to score positions we reach]
          ]
        ]
      )
    ]
  ]
  #v(0.6em)
  #grid(
    columns: (1fr, 1fr),
    gutter: 1.5em,
    [
      #align(center)[
        #box(fill: rgb("#dbeafe"), radius: 6pt, inset: 0.8em)[
          #text(weight: "bold", fill: primary)[Negamax]
          #v(0.2em)
          #text(size: 14pt)[Deterministic depth-first search with pruning]
        ]
      ]
    ],
    [
      #align(center)[
        #box(fill: rgb("#ede9fe"), radius: 6pt, inset: 0.8em)[
          #text(weight: "bold", fill: secondary)[MCTS]
          #v(0.2em)
          #text(size: 14pt)[Probabilistic sampling with statistics]
        ]
      ]
    ]
  )
]

// --------------------------------------------
// Slide 4: Negamax Search
// --------------------------------------------

#content-slide("Negamax Search")[
  #grid(
    columns: (1.2fr, 1fr),
    gutter: 1.5em,
    [
      #box(fill: rgb("#dbeafe"), radius: 6pt, inset: 0.8em, width: 100%)[
        #text(weight: "bold", fill: primary, size: 18pt)[Core Insight]
        #v(0.3em)
        In a zero-sum game:
        #align(center)[
          #box(fill: white, radius: 4pt, inset: 0.4em)[
            #text(size: 16pt, fill: dark)[max(a, b) = −min(−a, −b)]
          ]
        ]
        #v(0.2em)
        #text(size: 14pt)[Your best move = opponent's worst outcome. Just negate scores!]
      ]
      #v(0.3em)
      #box(fill: rgb("#f0fdf4"), radius: 6pt, inset: 0.8em, width: 100%)[
        #text(weight: "bold", fill: rgb("#16a34a"), size: 18pt)[Alpha-Beta Pruning]
        #v(0.2em)
        #text(size: 14pt)[
          Track best scores for both players. Skip branches that can't affect result. \
          Reduces $O(b^d)$ to $O(b^(d/2))$ in best case!
        ]
      ]
    ],
    [
      #align(center)[
        #box(fill: rgb("#f8fafc"), radius: 6pt, inset: 0.8em)[
          #text(size: 13pt, fill: rgb("#64748b"))[Game Tree]
          #v(0.5em)
          #grid(
            columns: (1fr,),
            gutter: 0.4em,
            [
              #align(center)[
                #box(fill: primary, radius: 50%, width: 2.2em, height: 2.2em)[
                  #align(center + horizon)[#text(fill: white, size: 11pt)[MAX]]
                ]
              ]
            ],
            [
              #align(center)[
                #grid(
                  columns: (1fr, 1fr, 1fr),
                  gutter: 0.3em,
                  [#box(fill: secondary, radius: 50%, width: 1.6em, height: 1.6em)[#align(center + horizon)[#text(fill: white, size: 9pt)[min]]]],
                  [#box(fill: secondary, radius: 50%, width: 1.6em, height: 1.6em)[#align(center + horizon)[#text(fill: white, size: 9pt)[min]]]],
                  [#box(fill: secondary, radius: 50%, width: 1.6em, height: 1.6em)[#align(center + horizon)[#text(fill: white, size: 9pt)[min]]]]
                )
              ]
            ],
            [#align(center)[#text(size: 11pt, fill: rgb("#64748b"))[...]]]
          )
        ]
      ]
    ]
  )
]

// --------------------------------------------
// Slide 5: Monte Carlo Tree Search
// --------------------------------------------

#content-slide("Monte Carlo Tree Search (MCTS)")[
  #align(center)[
    #box(fill: rgb("#f8fafc"), radius: 10pt, inset: 0.8em)[
      #grid(
        columns: (1fr, 1fr, 1fr, 1fr),
        gutter: 0.8em,
        [
          #box(fill: rgb("#dbeafe"), radius: 6pt, inset: 0.6em)[
            #align(center)[
              #text(weight: "bold", fill: primary, size: 15pt)[1. Select]
              #v(0.2em)
              #text(size: 12pt)[UCT formula balances explore vs exploit]
              #v(0.2em)
              #text(size: 11pt)[$"UCT" = Q/N + c sqrt(ln N_p / N)$]
            ]
          ]
        ],
        [
          #box(fill: rgb("#ede9fe"), radius: 6pt, inset: 0.6em)[
            #align(center)[
              #text(weight: "bold", fill: secondary, size: 15pt)[2. Expand]
              #v(0.2em)
              #text(size: 12pt)[Add child node for unexplored move]
            ]
          ]
        ],
        [
          #box(fill: rgb("#cffafe"), radius: 6pt, inset: 0.6em)[
            #align(center)[
              #text(weight: "bold", fill: accent, size: 15pt)[3. Evaluate]
              #v(0.2em)
              #text(size: 12pt)[Score position with eval function]
            ]
          ]
        ],
        [
          #box(fill: rgb("#fef3c7"), radius: 6pt, inset: 0.6em)[
            #align(center)[
              #text(weight: "bold", fill: rgb("#d97706"), size: 15pt)[4. Backprop]
              #v(0.2em)
              #text(size: 12pt)[Update stats back to root]
            ]
          ]
        ]
      )
    ]
  ]
  #v(0.5em)
  #grid(
    columns: (1fr, 1fr),
    gutter: 1.5em,
    [
      #box(fill: rgb("#f0fdf4"), radius: 6pt, inset: 0.7em)[
        #text(weight: "bold", fill: rgb("#16a34a"), size: 14pt)[Pros]
        #text(size: 13pt)[
          - Works well with neural network eval
          - Naturally handles uncertainty
          - Used by AlphaZero
        ]
      ]
    ],
    [
      #box(fill: rgb("#fef2f2"), radius: 6pt, inset: 0.7em)[
        #text(weight: "bold", fill: rgb("#dc2626"), size: 14pt)[Cons]
        #text(size: 13pt)[
          - Slower in tactical positions
          - Needs many iterations
          - Memory intensive
        ]
      ]
    ]
  )
]

// --------------------------------------------
// Slide 6: Why Negamax is Hard
// --------------------------------------------

#content-slide("Why Negamax for Chess is Hard")[
  #box(fill: rgb("#fef2f2"), radius: 6pt, inset: 0.6em, width: 100%)[
    #align(center)[
      #text(fill: rgb("#dc2626"), weight: "bold", size: 16pt)[
        Branching factor: ~35 moves/position
      ]
      #h(0.8em)
      #text(size: 14pt)[At depth 6: 35#super[6] = 1.8 billion nodes!]
    ]
  ]
  #v(0.4em)
  #text(weight: "bold", size: 18pt)[We need heuristics to prune effectively:]
  #v(0.3em)
  #grid(
    columns: (1fr, 1fr),
    gutter: 0.8em,
    [
      #box(fill: rgb("#dbeafe"), radius: 6pt, inset: 0.6em)[
        #text(weight: "bold", fill: primary, size: 14pt)[Iterative Deepening]
        #v(0.2em)
        #text(size: 12pt)[Search depth 1, then 2, then 3... Use shallow results to order deeper search.]
      ]
      #v(0.3em)
      #box(fill: rgb("#ede9fe"), radius: 6pt, inset: 0.6em)[
        #text(weight: "bold", fill: secondary, size: 14pt)[Transposition Tables]
        #v(0.2em)
        #text(size: 12pt)[Cache positions by hash. Same position via different moves? Reuse!]
      ]
    ],
    [
      #box(fill: rgb("#cffafe"), radius: 6pt, inset: 0.6em)[
        #text(weight: "bold", fill: accent, size: 14pt)[Move Ordering]
        #v(0.2em)
        #text(size: 12pt)[Try best first: TT move, captures, killers, history heuristic.]
      ]
      #v(0.3em)
      #box(fill: rgb("#fef3c7"), radius: 6pt, inset: 0.6em)[
        #text(weight: "bold", fill: rgb("#d97706"), size: 14pt)[Quiescence Search]
        #v(0.2em)
        #text(size: 12pt)[Don't stop in "noisy" positions. Extend for captures until quiet.]
      ]
    ]
  )
]

// --------------------------------------------
// Slide 7: Simple Evaluation Functions
// --------------------------------------------

#content-slide("Evaluation: Simple Approaches")[
  #grid(
    columns: (1fr, 1fr),
    gutter: 1.5em,
    [
      #box(fill: rgb("#dbeafe"), radius: 6pt, inset: 0.8em, width: 100%)[
        #text(weight: "bold", fill: primary, size: 18pt)[Material Counting]
        #v(0.3em)
        #text(size: 14pt)[Sum piece values:]
        #v(0.2em)
        #align(center)[
          #text(size: 13pt)[
            #grid(
              columns: (auto, auto, auto, auto, auto),
              gutter: 1em,
              [P=100], [N=320], [B=330], [R=500], [Q=900]
            )
          ]
        ]
        #v(0.2em)
        #text(size: 12pt, fill: rgb("#64748b"))[Simple but misses positional nuance]
      ]
    ],
    [
      #box(fill: rgb("#ede9fe"), radius: 6pt, inset: 0.8em, width: 100%)[
        #text(weight: "bold", fill: secondary, size: 18pt)[Piece-Square Tables]
        #v(0.3em)
        #text(size: 14pt)[
          Position-based bonuses:
          - Knights love the center
          - Rooks love open files
          - King hides early, advances late
        ]
        #v(0.2em)
        #text(size: 12pt, fill: rgb("#64748b"))[Separate midgame & endgame tables]
      ]
    ]
  )
  #v(0.4em)
  #align(center)[
    #box(fill: rgb("#f0fdf4"), radius: 6pt, inset: 0.6em)[
      #text(size: 14pt)[
        #text(weight: "bold", fill: rgb("#16a34a"))[Extra heuristics:]
        Passed pawns (+), Doubled pawns (-), Bishop pair (+), King safety...
      ]
    ]
  ]
]

// --------------------------------------------
// Slide 8: Introduction to NNUE
// --------------------------------------------

#section-slide("NNUE: Neural Networks for Chess")

// --------------------------------------------
// Slide 9: What is NNUE?
// --------------------------------------------

#content-slide("What Makes NNUE Special?")[
  #box(fill: rgb("#dbeafe"), radius: 6pt, inset: 0.8em, width: 100%)[
    #text(weight: "bold", fill: primary, size: 20pt)[NNUE = Efficiently Updatable Neural Network]
    #h(0.8em)
    #text(size: 14pt, fill: rgb("#64748b"))[Originally from Shogi (Yu Nasu, 2018)]
  ]
  #v(0.5em)
  #grid(
    columns: (1fr, 1fr),
    gutter: 1.5em,
    [
      #box(fill: rgb("#fef2f2"), radius: 6pt, inset: 0.8em)[
        #text(weight: "bold", fill: rgb("#dc2626"), size: 16pt)[Standard Neural Network]
        #v(0.3em)
        #text(size: 14pt)[
          Every evaluation requires:
          - Full forward pass
          - All weights × all inputs
          - 789 × hidden_size multiplications
          - #text(weight: "bold")[Expensive!]
        ]
      ]
    ],
    [
      #box(fill: rgb("#f0fdf4"), radius: 6pt, inset: 0.8em)[
        #text(weight: "bold", fill: rgb("#16a34a"), size: 16pt)[NNUE Approach]
        #v(0.3em)
        #text(size: 14pt)[
          Exploit incremental changes:
          - Most inputs unchanged between moves
          - Update only what changed
          - #text(weight: "bold")[Massive speedup!]
        ]
      ]
    ]
  )
  #v(0.4em)
  #align(center)[
    #box(fill: rgb("#fef3c7"), radius: 6pt, inset: 0.5em)[
      #text(size: 15pt, weight: "bold")[Key insight: A move only changes 2-4 pieces on the board]
    ]
  ]
]

// --------------------------------------------
// Slide 10: NNUE Architecture
// --------------------------------------------

#content-slide("NNUE Architecture vs Standard MLP")[
  #grid(
    columns: (1.3fr, 1fr),
    gutter: 1em,
    [
      #box(fill: rgb("#f8fafc"), radius: 6pt, inset: 0.8em)[
        #text(weight: "bold", size: 16pt)[Input Encoding (789 features)]
        #v(0.3em)
        #grid(
          columns: (auto, 1fr),
          gutter: 0.4em,
          row-gutter: 0.2em,
          [#box(fill: primary, radius: 3pt, inset: 0.2em)[#text(fill: white, size: 11pt)[768]]],
          [#text(size: 13pt)[Piece-square (12 pieces × 64 squares)]],
          [#box(fill: secondary, radius: 3pt, inset: 0.2em)[#text(fill: white, size: 11pt)[4]]],
          [#text(size: 13pt)[Castling rights]],
          [#box(fill: accent, radius: 3pt, inset: 0.2em)[#text(fill: white, size: 11pt)[16]]],
          [#text(size: 13pt)[En passant squares]],
          [#box(fill: rgb("#d97706"), radius: 3pt, inset: 0.2em)[#text(fill: white, size: 11pt)[1]]],
          [#text(size: 13pt)[Side to move]],
        )
        #v(0.4em)
        #box(fill: rgb("#fef3c7"), radius: 6pt, inset: 0.5em, width: 100%)[
          #text(weight: "bold", fill: rgb("#d97706"), size: 14pt)[The Sparsity Trick]
          #v(0.2em)
          #text(size: 13pt)[768 features, but only ~32 pieces! #text(weight: "bold")[95%+ are zero]]
        ]
      ]
    ],
    [
      #box(fill: rgb("#f1f5f9"), radius: 6pt, inset: 0.8em)[
        #align(center)[
          #text(weight: "bold", size: 14pt)[Network Structure]
          #v(0.5em)
          #box(fill: primary, radius: 4pt, inset: 0.4em, width: 85%)[
            #text(fill: white, size: 12pt)[Input: 789 (sparse)]
          ]
          #v(0.3em)
          #text(size: 16pt)[↓]
          #v(0.3em)
          #box(fill: secondary, radius: 4pt, inset: 0.4em, width: 70%)[
            #text(fill: white, size: 12pt)[Hidden layers]
          ]
          #v(0.3em)
          #text(size: 16pt)[↓]
          #v(0.3em)
          #box(fill: rgb("#16a34a"), radius: 4pt, inset: 0.4em, width: 50%)[
            #text(fill: white, size: 12pt)[Output: 1]
          ]
          #v(0.2em)
          #text(size: 11pt, fill: rgb("#64748b"))[(position score)]
        ]
      ]
    ]
  )
]

// --------------------------------------------
// Slide 11: CPU Optimization
// --------------------------------------------

#content-slide("NNUE: Why CPU Beats GPU")[
  #grid(
    columns: (1fr, 1fr),
    gutter: 1em,
    [
      #box(fill: rgb("#dbeafe"), radius: 6pt, inset: 0.8em)[
        #text(weight: "bold", fill: primary, size: 16pt)[Accumulator Technique]
        #v(0.3em)
        #text(size: 13pt)[
          Maintain running sum of active features:
          #v(0.2em)
          #box(fill: white, radius: 4pt, inset: 0.3em, width: 100%)[
            #text(size: 12pt)[acc = W[piece1] + W[piece2] + ...]
          ]
          #v(0.2em)
          On each move:
          - Subtract: removed piece contribution
          - Add: new position contribution
          #v(0.2em)
          #text(weight: "bold", fill: rgb("#16a34a"))[No full forward pass needed!]
        ]
      ]
    ],
    [
      #box(fill: rgb("#ede9fe"), radius: 6pt, inset: 0.6em)[
        #text(weight: "bold", fill: secondary, size: 14pt)[Why Not GPU?]
        #v(0.2em)
        #text(size: 12pt)[
          - Batch size = 1 (single position)
          - GPU excels at large batches
          - Memory transfer overhead
          - CPU cache faster for small ops
        ]
      ]
      #v(0.3em)
      #box(fill: rgb("#f0fdf4"), radius: 6pt, inset: 0.6em)[
        #text(weight: "bold", fill: rgb("#16a34a"), size: 14pt)[SIMD Acceleration]
        #v(0.2em)
        #text(size: 12pt)[
          - AVX2/AVX-512 parallel ops
          - 8-16 values at once
          - Perfect for accumulator
        ]
      ]
    ]
  )
  #v(0.3em)
  #align(center)[
    #box(fill: rgb("#fef3c7"), radius: 6pt, inset: 0.5em)[
      #text(size: 14pt)[#text(weight: "bold")[Result:] Millions of evaluations per second on CPU]
    ]
  ]
]

// --------------------------------------------
// Slide 12: Implementation
// --------------------------------------------

#content-slide("Implementation in Cheese Engine")[
  #grid(
    columns: (1fr, 1fr),
    gutter: 1em,
    [
      #box(fill: rgb("#f8fafc"), radius: 6pt, inset: 0.7em)[
        #text(weight: "bold", size: 16pt)[Search: Negamax]
        #v(0.2em)
        #text(size: 13pt)[
          - Alpha-beta with PVS
          - Iterative deepening (depth 1-4)
          - 16M entry transposition table
          - Aspiration windows
          #v(0.2em)
          #text(weight: "bold")[Move Ordering:] TT → Captures → Killers → History
        ]
      ]
      #v(0.3em)
      #box(fill: rgb("#dbeafe"), radius: 6pt, inset: 0.7em)[
        #text(weight: "bold", fill: primary, size: 14pt)[Tech Stack]
        #v(0.2em)
        #text(size: 12pt)[Rust, `chess` crate, `ort` (ONNX Runtime), UCI protocol]
      ]
    ],
    [
      #box(fill: rgb("#ede9fe"), radius: 6pt, inset: 0.7em)[
        #text(weight: "bold", size: 16pt)[Evaluation Options]
        #v(0.2em)
        #text(size: 13pt)[
          #text(weight: "bold")[1. Material counting] - Simple piece sum
          #v(0.1em)
          #text(weight: "bold")[2. Piece-Square Tables] - Phase-interpolated
          #v(0.1em)
          #text(weight: "bold")[3. NNUE] - 789-dim neural network
        ]
      ]
      #v(0.3em)
      #box(fill: rgb("#f0fdf4"), radius: 6pt, inset: 0.7em)[
        #text(weight: "bold", fill: rgb("#16a34a"), size: 14pt)[Also: MCTS]
        #v(0.2em)
        #text(size: 12pt)[Alternative search with UCT, 4000 iterations/move]
      ]
    ]
  )
]

// --------------------------------------------
// Slide 13: Conclusion
// --------------------------------------------

#content-slide("Conclusion & Future Work")[
  #box(fill: gradient.linear(rgb("#dbeafe"), rgb("#ede9fe")), radius: 10pt, inset: 1em, width: 100%)[
    #align(center)[
      #text(size: 20pt, weight: "bold")[Chess AI = Search + Evaluation]
      #v(0.3em)
      #text(size: 16pt)[
        NNUE bridges classical heuristics with neural networks, \
        enabling strong play without expensive hardware.
      ]
    ]
  ]
  #v(0.5em)
  #grid(
    columns: (1fr, 1fr),
    gutter: 1.5em,
    [
      #box(fill: rgb("#f0fdf4"), radius: 6pt, inset: 0.8em)[
        #text(weight: "bold", fill: rgb("#16a34a"), size: 16pt)[Key Takeaways]
        #v(0.2em)
        #text(size: 14pt)[
          - Chess too complex to solve completely
          - Smart heuristics make search tractable
          - Sparse inputs enable CPU-friendly nets
        ]
      ]
    ],
    [
      #box(fill: rgb("#dbeafe"), radius: 6pt, inset: 0.8em)[
        #text(weight: "bold", fill: primary, size: 16pt)[Future Directions]
        #v(0.2em)
        #text(size: 14pt)[
          - Deeper search with better pruning
          - Time management optimization
          - Train stronger NNUE models
        ]
      ]
    ]
  )
  #v(0.5em)
  #align(center)[#text(size: 22pt, weight: "bold", fill: primary)[Thank you!]]
]
