use crate::{
    constants::{BLACK, UNDEFINED_POSITION, WHITE},
    pieces::{PieceColor, PieceType},
    utils::{
        chtoi, col_to_letter, color_to_ratatui_enum, convert_notation_into_position,
        convert_position_into_notation, did_piece_already_move, get_king_coordinates,
        get_piece_color, get_piece_type, is_getting_checked, letter_to_col,
    },
};
use log::info;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};
use std::{cmp::Ordering, error::Error, fs::OpenOptions, io::Write};
use uci::Engine;

/// history record
pub type HistRec = (PieceType, String);

/// Coordinates
///
/// Columns/x/j:
/// |  |  |
/// |  |  |
/// |  |  |
/// 0  1  2
///
/// Rows/y/i:
/// _  _  _ 0
/// _  _  _ 1
/// _  _  _ 2
#[derive(PartialEq, Eq, Ord, PartialOrd, Clone)]
pub struct Coords {
    /// column/x/j/[1]
    /// |  |  |
    /// |  |  |
    /// |  |  |
    /// 0  1  2
    pub col: i8,
    /// row/y/i/[0]
    /// _  _  _ 0
    /// _  _  _ 1
    /// _  _  _ 2
    pub row: i8,
}
impl Coords {
    /// warning! these arguments are not (x;y) but (y;x) coordinates
    /// # Panics
    ///
    /// should be: if `col` or `row` exceeds max: 7, or is lower than 0
    ///
    /// should be: if you need an undefined position, use `Coords::default()`
    pub fn new<T: Into<i8>>(row: T, col: T) -> Self {
        let row = row.into();
        let col = col.into();
        if row < -2 {
            panic!("row < 0");
        }
        if row > 9 {
            panic!("row > 8");
        }
        if col < -2 {
            panic!("col < 0");
        }
        if col > 9 {
            panic!("col > 8");
        }
        // if !(0..=7).contains(&row) || !(0..=7).contains(&col) {
        //     panic!("row or col is not valid as a coordinate: should be 0-7");
        // }
        Self { col, row }
    }
    pub fn to_hist(&self) -> String {
        format!("{}{}", self.row, self.col)
    }
    /// Convert history record to coordinates
    pub fn from_hist(hist_item: &str) -> Self {
        assert_eq!(hist_item.chars().count(), 2);
        Self::new(
            hist_item
                .chars()
                .nth(0)
                .unwrap()
                .to_string()
                .parse::<i8>()
                .unwrap(),
            hist_item
                .chars()
                .nth(1)
                .unwrap()
                .to_string()
                .parse()
                .unwrap(),
        )
    }
    pub fn is_valid(&self) -> bool {
        (0..8).contains(&self.row) && (0..8).contains(&self.col)
    }
    /// like e3 or b8
    fn from_basic_san(san: &str) -> Self {
        assert_eq!(san.chars().count(), 2);

        let col_ch = san.chars().next();
        let col = letter_to_col(col_ch);

        let row_ch = san.chars().nth(1);
        let row = 8 - chtoi(row_ch);

        let coords = Self { col, row };

        assert!(coords.is_valid());

        coords
    }
}
impl Default for Coords {
    fn default() -> Self {
        Coords {
            col: UNDEFINED_POSITION,
            row: UNDEFINED_POSITION,
        }
    }
}
impl std::fmt::Debug for Coords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let col = col_to_letter(self.col);
        let row = 8 - self.row;
        write!(f, "{}{}", col, row)?;
        Ok(())
    }
}

pub type Piece = Option<(PieceType, PieceColor)>;
pub type GameBoard = [[Piece; 8]; 8];

pub struct Board {
    /// how it's stored:
    ///
    /// _ 0 1 2 3 4 5 6 7 _
    /// 0 ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖ 0
    /// 1 ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙ 1
    /// 2 _ _ _ _ _ _ _ _ 2
    /// 3 _ _ _ _ _ _ _ _ 3
    /// 4 _ _ _ _ _ _ _ _ 4
    /// 5 _ _ _ _ _ _ _ _ 5
    /// 6 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟ 6
    /// 7 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜ 7
    /// _ 0 1 2 3 4 5 6 7 _
    ///
    /// how it's in real world:
    /// _ a b c d e f g h _
    /// 8 ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖ 8
    /// 7 ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙ 7
    /// 6 _ _ _ _ _ _ _ _ 6
    /// 5 _ _ _ _ _ _ _ _ 5
    /// 4 _ _ _ _ _ _ _ _ 4
    /// 3 _ _ _ _ _ _ _ _ 3
    /// 2 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟ 2
    /// 1 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜ 1
    /// _ a b c d e f g h _
    pub board: GameBoard,
    pub cursor_coordinates: Coords,
    pub selected_coordinates: Coords,
    pub selected_piece_cursor: i8,
    pub old_cursor_position: Coords,
    pub player_turn: PieceColor,
    pub move_history: Vec<HistRec>,
    pub is_draw: bool,
    pub is_checkmate: bool,
    pub is_promotion: bool,
    pub promotion_cursor: i8,
    pub consecutive_non_pawn_or_capture: i32,
    pub engine: Option<Engine>,
    pub is_game_against_bot: bool,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            board: [
                [
                    Some((PieceType::Rook, PieceColor::Black)),
                    Some((PieceType::Knight, PieceColor::Black)),
                    Some((PieceType::Bishop, PieceColor::Black)),
                    Some((PieceType::Queen, PieceColor::Black)),
                    Some((PieceType::King, PieceColor::Black)),
                    Some((PieceType::Bishop, PieceColor::Black)),
                    Some((PieceType::Knight, PieceColor::Black)),
                    Some((PieceType::Rook, PieceColor::Black)),
                ],
                [
                    Some((PieceType::Pawn, PieceColor::Black)),
                    Some((PieceType::Pawn, PieceColor::Black)),
                    Some((PieceType::Pawn, PieceColor::Black)),
                    Some((PieceType::Pawn, PieceColor::Black)),
                    Some((PieceType::Pawn, PieceColor::Black)),
                    Some((PieceType::Pawn, PieceColor::Black)),
                    Some((PieceType::Pawn, PieceColor::Black)),
                    Some((PieceType::Pawn, PieceColor::Black)),
                ],
                [None, None, None, None, None, None, None, None],
                [None, None, None, None, None, None, None, None],
                [None, None, None, None, None, None, None, None],
                [None, None, None, None, None, None, None, None],
                [
                    Some((PieceType::Pawn, PieceColor::White)),
                    Some((PieceType::Pawn, PieceColor::White)),
                    Some((PieceType::Pawn, PieceColor::White)),
                    Some((PieceType::Pawn, PieceColor::White)),
                    Some((PieceType::Pawn, PieceColor::White)),
                    Some((PieceType::Pawn, PieceColor::White)),
                    Some((PieceType::Pawn, PieceColor::White)),
                    Some((PieceType::Pawn, PieceColor::White)),
                ],
                [
                    Some((PieceType::Rook, PieceColor::White)),
                    Some((PieceType::Knight, PieceColor::White)),
                    Some((PieceType::Bishop, PieceColor::White)),
                    Some((PieceType::Queen, PieceColor::White)),
                    Some((PieceType::King, PieceColor::White)),
                    Some((PieceType::Bishop, PieceColor::White)),
                    Some((PieceType::Knight, PieceColor::White)),
                    Some((PieceType::Rook, PieceColor::White)),
                ],
            ],
            cursor_coordinates: Coords::new(4, 4),
            selected_coordinates: Coords::default(),
            selected_piece_cursor: 0,
            old_cursor_position: Coords::default(),
            player_turn: PieceColor::White,
            move_history: vec![],
            is_draw: false,
            is_checkmate: false,
            is_promotion: false,
            promotion_cursor: 0,
            consecutive_non_pawn_or_capture: 0,
            engine: None,
            is_game_against_bot: false,
        }
    }
}

impl Board {
    pub fn new(board: GameBoard, player_turn: PieceColor, move_history: Vec<HistRec>) -> Self {
        Self {
            board,
            cursor_coordinates: Coords::new(4, 4),
            selected_coordinates: Coords::default(),
            selected_piece_cursor: 0,
            old_cursor_position: Coords::default(),
            player_turn,
            move_history,
            is_draw: false,
            is_checkmate: false,
            is_promotion: false,
            promotion_cursor: 0,
            consecutive_non_pawn_or_capture: 0,
            engine: None,
            is_game_against_bot: false,
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, Box<dyn Error>> {
        info!("{:?}", fen);
        let mut fen = fen.split(' ');
        if fen.clone().count() != 6 {
            return Err("incorrect fen position: not 6 fields".into());
        }
        let board_state = fen.next().unwrap();
        let mut board = [[None; 8]; 8];
        let mut j;
        for (i, row) in board_state.split('/').enumerate() {
            j = 0;
            for ch in row.chars() {
                if let Some(piece) = PieceType::from_char(ch) {
                    board[i][j] = Some(piece);
                } else {
                    for k in j..j + ch.to_digit(10).unwrap() as usize {
                        // info!("{}", k);
                        board[i][k] = None;
                    }
                    j += ch.to_digit(10).unwrap() as usize;
                    continue;
                }
                j += 1;
            }
        }

        let player_turn = match fen.next().unwrap().chars().next().unwrap() {
            'w' => PieceColor::White,
            'b' => PieceColor::Black,
            invalid_color => {
                return Err(format!(
                    "color should be either w or b, \'{}\' is invalid",
                    invalid_color
                )
                .into())
            }
        };
        Ok(Self {
            board,
            cursor_coordinates: Coords::new(4, 4),
            selected_coordinates: Coords::default(),
            selected_piece_cursor: 0,
            old_cursor_position: Coords::default(),
            player_turn,
            move_history: vec![],
            is_draw: false,
            is_checkmate: false,
            is_promotion: false,
            promotion_cursor: 0,
            consecutive_non_pawn_or_capture: 0,
            engine: None,
            is_game_against_bot: false,
        })
    }

    // Setters
    pub fn set_board(&mut self, board: GameBoard) {
        self.board = board;
    }

    pub fn set_player_turn(&mut self, player_turn: PieceColor) {
        self.player_turn = player_turn;
    }

    pub fn set_engine(&mut self, engine_path: &str) {
        self.is_game_against_bot = true;

        self.engine = match Engine::new(engine_path) {
            Ok(engine) => Some(engine),
            _ => panic!("An error occcured with the selected chess engine path: {} Make sure you specified the right path using chess-tui -e", engine_path),
        }
    }

    // Check if a cell has been selected
    pub fn is_cell_selected(&self) -> bool {
        self.selected_coordinates.row != UNDEFINED_POSITION
            && self.selected_coordinates.col != UNDEFINED_POSITION
    }

    // fn get_mut(&mut self, coord: &Coord) -> &mut Piece {
    //     &mut self.board[coord.row as usize][coord.col as usize]
    // }
    /// get `self.board` at `coord`
    fn get(&self, coord: &Coords) -> Piece {
        self.board[coord.row as usize][coord.col as usize]
    }
    /// set `self.board` at `coord` to `piece`
    fn set(&mut self, coord: &Coords, piece: Piece) {
        self.board[coord.row as usize][coord.col as usize] = piece;
    }
    // /// set `self.board` at `coord` created from `x`,`y` to `piece`
    // fn coord_set<T: Into<usize>>(&mut self, x: T, y: T, piece: Piece) {
    //     self.board[y.into()][x.into()] = piece;
    // }

    fn get_authorized_positions(
        &self,
        piece_type: Option<PieceType>,
        piece_color: Option<PieceColor>,
        piece_position: &Coords,
    ) -> Vec<Coords> {
        match (piece_type, piece_color) {
            (Some(piece_type), Some(piece_color)) => piece_type.authorized_positions(
                piece_position,
                piece_color,
                self.board,
                &self.move_history,
                is_getting_checked(self.board, self.player_turn, &self.move_history),
            ),
            _ => Vec::new(),
        }
    }
    pub fn switch_player_turn(&mut self) {
        self.player_turn = self.player_turn.opposite();
    }

    // Methods to change the position of the cursor
    pub fn cursor_up(&mut self) {
        if !self.is_checkmate && !self.is_draw && !self.is_promotion {
            if self.is_cell_selected() {
                self.move_selected_piece_cursor(false, -1)
            } else if self.cursor_coordinates.row > 0 {
                self.cursor_coordinates.row -= 1
            }
        }
    }
    pub fn cursor_down(&mut self) {
        if !self.is_checkmate && !self.is_draw && !self.is_promotion {
            if self.is_cell_selected() {
                self.move_selected_piece_cursor(false, 1)
            } else if self.cursor_coordinates.row < 7 {
                self.cursor_coordinates.row += 1
            }
        }
    }
    pub fn cursor_left(&mut self) {
        // If we are doing a promotion the cursor is used for the popup
        if self.is_promotion {
            self.promotion_cursor = if self.promotion_cursor > 0 {
                self.promotion_cursor - 1
            } else {
                3
            };
        } else if !self.is_checkmate && !self.is_draw {
            if self.is_cell_selected() {
                self.move_selected_piece_cursor(false, -1)
            } else if self.cursor_coordinates.col > 0 {
                self.cursor_coordinates.col -= 1
            }
        }
    }
    pub fn cursor_right(&mut self) {
        // If we are doing a promotion the cursor is used for the popup
        if self.is_promotion {
            self.promotion_cursor = (self.promotion_cursor + 1) % 4;
        } else if !self.is_checkmate && !self.is_draw {
            if self.is_cell_selected() {
                self.move_selected_piece_cursor(false, 1)
            } else if self.cursor_coordinates.col < 7 {
                self.cursor_coordinates.col += 1
            }
        }
    }

    pub fn did_king_already_move(&self) -> bool {
        for (i, item) in self.move_history.iter().enumerate() {
            if item.0 == PieceType::King && self.player_turn as usize == i % 2 {
                return true;
            }
        }
        false
    }

    fn move_selected_piece_cursor(&mut self, first_time_moving: bool, direction: i8) {
        let piece_color = get_piece_color(self.board, &self.selected_coordinates);
        let piece_type = get_piece_type(self.board, &self.selected_coordinates);

        let mut authorized_positions =
            self.get_authorized_positions(piece_type, piece_color, &self.selected_coordinates);

        if !authorized_positions.is_empty() {
            self.selected_piece_cursor = if self.selected_piece_cursor == 0 && first_time_moving {
                0
            } else {
                let new_cursor =
                    (self.selected_piece_cursor + direction) % authorized_positions.len() as i8;
                if new_cursor == -1 {
                    authorized_positions.len() as i8 - 1
                } else {
                    new_cursor
                }
            };

            authorized_positions.sort();

            if let Some(position) = authorized_positions.get(self.selected_piece_cursor as usize) {
                self.cursor_coordinates = position.clone();
            }
        } else {
            self.cursor_coordinates = Coords::default();
        }
    }

    // Methods to select a cell on the board
    pub fn select_cell(&mut self) {
        self.export_fen_position();
        // If we are doing a promotion the cursor is used for the popup
        if self.is_promotion {
            self.promote_piece();
        } else if !self.is_checkmate && !self.is_draw {
            if !self.is_cell_selected() {
                // Check if the piece on the cell can move before selecting it
                let piece_color = get_piece_color(self.board, &self.cursor_coordinates);
                let piece_type = get_piece_type(self.board, &self.cursor_coordinates);

                let authorized_positions = self.get_authorized_positions(
                    piece_type,
                    piece_color,
                    &self.cursor_coordinates,
                );

                if authorized_positions.is_empty() {
                    return;
                }
                if let Some(piece_color) = get_piece_color(self.board, &self.cursor_coordinates) {
                    if piece_color == self.player_turn {
                        self.selected_coordinates = self.cursor_coordinates.clone();
                        self.old_cursor_position = self.cursor_coordinates.clone();
                        self.move_selected_piece_cursor(true, 1);
                    }
                }
            } else {
                // We already selected a piece
                if self.cursor_coordinates.is_valid() {
                    let selected_coords = &self.selected_coordinates.clone();
                    let cursor_coords = &self.cursor_coordinates.clone();
                    self.move_piece(selected_coords, cursor_coords);
                    self.unselect_cell();
                    self.switch_player_turn();
                    // If we play against a bot we will play his move and switch the player turn again
                    if self.is_game_against_bot {
                        self.is_promotion = self.is_latest_move_promotion();
                        if !self.is_promotion {
                            self.is_checkmate = self.is_checkmate();
                            self.is_promotion = self.is_latest_move_promotion();
                            if !self.is_checkmate {
                                self.bot_move();
                                self.switch_player_turn();
                            }
                        }
                    }
                    self.is_draw = self.is_draw();
                }
            }
        }
        self.is_checkmate = self.is_checkmate();
        self.is_promotion = self.is_latest_move_promotion();
    }

    pub fn bot_move(&mut self) {
        let engine = match &self.engine {
            Some(engine) => engine,
            None => panic!("Missing the chess engine"),
        };

        engine.set_position(&self.fen_position()).unwrap();

        let best_move = engine.bestmove();
        let movement = match best_move {
            Ok(movement) => movement,
            Err(_) => panic!("An error as occured"),
        };
        let converted_move = convert_notation_into_position(movement);

        let from_y = chtoi(converted_move.chars().next());
        let from_x = chtoi(converted_move.chars().nth(1));
        let from = Coords::new(from_y, from_x);
        let to_y = chtoi(converted_move.chars().nth(2));
        let to_x = chtoi(converted_move.chars().nth(3));
        let to = Coords::new(to_y, to_x);

        self.move_piece(&from, &to);
    }
    // Convert the history and game status to a FEN string
    pub fn fen_position(&self) -> String {
        let mut result = String::new();

        for i in 0..8i8 {
            for j in 0..8i8 {
                let (piece_type, piece_color) = (
                    get_piece_type(self.board, &Coords::new(i, j)),
                    get_piece_color(self.board, &Coords::new(i, j)),
                );
                match PieceType::piece_to_fen_enum(piece_type, piece_color) {
                    // Pattern match directly on the result of piece_to_fen_enum
                    "" => {
                        // Check if the string is not empty before using chars().last()
                        if let Some(last_char) = result.chars().last() {
                            if last_char.is_ascii_digit() {
                                let incremented_char =
                                    char::from_digit(last_char.to_digit(10).unwrap_or(0) + 1, 10)
                                        .unwrap_or_default();
                                // Remove the old number and add the new incremented one
                                result.pop();
                                result.push_str(incremented_char.to_string().as_str());
                            } else {
                                result.push('1');
                            }
                        } else {
                            result.push('1');
                        }
                    }
                    letter => {
                        // If the result is not an empty string, push '1'
                        result.push_str(letter);
                    }
                }
            }
            result.push('/')
        }

        // we remove the last / and specify the player turn (black)
        result.pop();

        // We say it is blacks turn to play
        result.push_str(" b");

        // We add the castles availabilities for black
        if !did_piece_already_move(&self.move_history, (PieceType::King, Coords::new(0, 4)))
            && !is_getting_checked(self.board, PieceColor::Black, &self.move_history)
        {
            // king side black castle availability
            if !did_piece_already_move(&self.move_history, (PieceType::Rook, Coords::new(0, 7))) {
                result.push_str(" k");
            }
            // queen side black castle availability
            if !did_piece_already_move(&self.move_history, (PieceType::Rook, Coords::new(0, 0))) {
                result.push('q');
            }
        } else {
            result.push_str(" -")
        }

        // We check if the latest move is a pawn moving 2 cells, meaning the next move can be en passant
        if self.did_pawn_move_two_cells() {
            // Use an if-let pattern for better readability
            if let Some((_, latest_move_string)) = self.move_history.last() {
                let mut converted_move: String = String::new();

                if let (Some(from_y_char), Some(from_x_char)) = (
                    latest_move_string.chars().nth(0),
                    latest_move_string.chars().nth(1),
                ) {
                    let from_y = chtoi(Some(from_y_char)) - 1;
                    let from_x = chtoi(Some(from_x_char));

                    converted_move += &col_to_letter(from_x);
                    converted_move += &format!("{}", 8 - from_y).to_string();

                    result.push(' ');
                    result.push_str(&converted_move);
                }
            }
        } else {
            result.push_str(" -");
        }

        result.push(' ');

        result.push_str(&self.consecutive_non_pawn_or_capture.to_string());
        result.push(' ');

        result.push_str(&(self.move_history.len() / 2).to_string());

        result
    }

    pub fn export_fen_position(&self) {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open("chess-tui.fen")
            .expect("could not open chess-tui.fen");

        writeln!(f, "{}", self.fen_position()).expect("could not save current fen position");
    }

    pub fn did_pawn_move_two_cells(&self) -> bool {
        match self.move_history.last() {
            Some((piece_type, move_string)) => {
                let from_y = chtoi(move_string.chars().next());
                let to_y = chtoi(move_string.chars().nth(2));

                let distance = (to_y - from_y).abs();

                if piece_type == &PieceType::Pawn && distance == 2 {
                    return true;
                }
                false
            }
            _ => false,
        }
    }
    pub fn promote_piece(&mut self) {
        if let Some(position) = self.move_history.last() {
            let to = Coords::new(
                chtoi(position.1.chars().nth(2)),
                chtoi(position.1.chars().nth(3)),
            );
            let new_piece = match self.promotion_cursor {
                0 => PieceType::Queen,
                1 => PieceType::Rook,
                2 => PieceType::Bishop,
                3 => PieceType::Knight,
                _ => unreachable!("Promotion cursor out of boundaries"),
            };

            let current_piece_color = get_piece_color(self.board, &to);
            if let Some(piece_color) = current_piece_color {
                // we replace the piece by the new piece type
                // self.board.set_coord(&to, Some((new_piece, piece_color)));
                self.set(&to, Some((new_piece, piece_color)));
            }
        }
        self.is_promotion = false;
        self.promotion_cursor = 0;
    }

    pub fn move_piece(&mut self, from: &Coords, to: &Coords) {
        if !from.is_valid() || !to.is_valid() {
            return;
        }
        let direction_y = if self.player_turn == PieceColor::White {
            -1
        } else {
            1
        };

        let piece_type_from = get_piece_type(self.board, from);
        let piece_type_to = get_piece_type(self.board, to);

        // We increment the consecutive_non_pawn_or_capture if the piece type is a pawn or if there is no capture
        match (piece_type_from, piece_type_to) {
            (Some(PieceType::Pawn), _) | (Some(_), Some(_)) => {
                self.consecutive_non_pawn_or_capture = 0;
            }
            _ => {
                self.consecutive_non_pawn_or_capture += 1;
            }
        }

        // We check for en passant as the latest move
        if self.is_latest_move_en_passant(from, to) {
            // we kill the pawn
            let row_index = to.row as i32 - direction_y;

            // self.board[row_index as usize][to.col as usize] = None;
            self.set(&Coords::new(row_index as i8, to.col), None);
        }

        let mut to_hist = Coords::new(to.row, to.col);

        // We check for castling as the latest move
        if self.is_latest_move_castling(from, to) {
            // we set the king 2 cells on where it came from

            let mut to_x: i32 = to.col as i32;

            let distance = from.col as i32 - to_x;
            let direction_x = if distance > 0 { -1 } else { 1 };

            let row_index_rook;

            let row_index = from.col + direction_x * 2;

            // We put move the king 2 cells
            self.set(&Coords::new(to.row, row_index), self.get(from));

            // We put the rook 3 cells from it's position if it's a big castling else 2 cells
            // If it is playing against a bot we will receive 4 -> 6  and 4 -> 2 for to_x instead of 4 -> 7 and 4 -> 0
            // big castling
            match distance.cmp(&0) {
                Ordering::Less => {
                    row_index_rook = 5;
                    if self.is_game_against_bot && self.player_turn == PieceColor::Black {
                        to_x = 7;
                    }
                }
                Ordering::Greater => {
                    row_index_rook = 3;
                    if self.is_game_against_bot && self.player_turn == PieceColor::Black {
                        to_x = 0;
                    }
                }
                Ordering::Equal => unreachable!("having castled, a king's x axis has changed"),
            }

            self.board[to.row as usize][row_index_rook as usize] =
                self.board[to.row as usize][to_x as usize];

            // We remove the latest rook
            self.board[to.row as usize][to_x as usize] = None;
            to_hist.col = row_index;
        } else {
            self.set(to, self.get(from));
        }

        self.set(from, None);

        let position_number: String = format!("{}{}", from.to_hist(), to_hist.to_hist());
        // We store it in the history
        if let Some(piece_type) = piece_type_from {
            let tuple = (piece_type, position_number);
            self.move_history.push(tuple.clone());
        }
    }

    /// move history of `self` contains this coordinate, either as moved to or from
    fn history_has(&self, coord: &Coords, to: bool) -> Option<(PieceType, usize)> {
        let hist = &self.move_history;
        if hist.is_empty() {
            return None;
        }

        let mut i = hist.len() - 1;
        while i > 0 {
            let hist_rec = &hist[i].1;
            if to {
                if hist_rec[2..4] == coord.to_hist() {
                    return Some((hist[i].0, i));
                }
            } else if hist_rec[0..2] == coord.to_hist() {
                return Some((hist[i].0, i));
            }
            i -= 1;
        }
        None
    }

    /// takeback
    pub fn takeback(&mut self) {
        if let Some((piece_type, prev_move)) = self.move_history.pop() {
            let to = Coords::from_hist(&prev_move[0..2]);
            let from = Coords::from_hist(&prev_move[2..4]);

            // check for castling
            if piece_type == PieceType::King && (from.col - to.col).abs() > 1 {
                // check all 4 rooks, place back the one that was involved in castling
                let right_rook = Coords::new(from.row, from.col - 1);
                let left_rook = Coords::new(from.row, from.col + 1);
                match self.player_turn {
                    PieceColor::Black => {
                        if self
                            .get(&right_rook)
                            .is_some_and(|piece| piece.0 == PieceType::Rook)
                        {
                            self.set(&right_rook, None);
                            self.set(
                                &Coords::new(7, 7),
                                Some((PieceType::Rook, PieceColor::White)),
                            );
                        } else {
                            self.set(&left_rook, None);
                            self.set(
                                &Coords::new(7, 0),
                                Some((PieceType::Rook, PieceColor::White)),
                            )
                        }
                    }
                    PieceColor::White => {
                        if self
                            .get(&right_rook)
                            .is_some_and(|piece| piece.0 == PieceType::Rook)
                        {
                            self.set(&right_rook, None);
                            self.set(
                                &Coords::new(0, 7),
                                Some((PieceType::Rook, PieceColor::Black)),
                            )
                        } else {
                            self.set(&left_rook, None);
                            self.set(
                                &Coords::new(0, 0),
                                Some((PieceType::Rook, PieceColor::Black)),
                            )
                        }
                    }
                }
            }
            // check for en-passant
            else if piece_type == PieceType::Pawn && to.row != from.row && to.col != from.col {
                if let Some((PieceType::Pawn, hist)) = self.move_history.last() {
                    let passant_from = Coords::from_hist(&hist[0..2]);
                    let passant_to = Coords::from_hist(&hist[2..4]);
                    if (passant_to.row - passant_from.row).abs() > 1
                        && (from.row - passant_to.row).abs() == 1
                    {
                        self.set(&passant_to, Some((PieceType::Pawn, self.player_turn)));
                    }
                }
            }
            // check for promotions
            if piece_type == PieceType::Pawn && (from.row == 0 || from.row == 7) {
                // todo!("promotion takeback");
                self.set(&to, Some((PieceType::Pawn, self.player_turn.opposite())));
            } else {
                // take last moved piece back to where it came from
                self.set(&to, self.get(&from));
            }

            // pseudo kind of code
            // if history.contains(board[from], Moved::To) && !history.contains(board[from], Moved::From) {
            //     board[from] = history[from]
            // }

            // optionally fill the cell that it moved to if something was taken off it
            self.set(
                &from,
                // check if there was anything on the cell where it was before takeback:
                // if anything has moved to this cell and not away from it, there probably was
                if (self.history_has(&from, true).is_some()
                    && self.history_has(&from, false).is_none())
                    || (self.history_has(&from, false).is_some()
                        && self.history_has(&from, true).is_some()
                        && (self.history_has(&from, true).unwrap().1
                            > self.history_has(&from, false).unwrap().1))
                {
                    let kicked_kind = self.history_has(&from, true).unwrap().0;
                    Some((kicked_kind, self.player_turn))
                // didn't move yet, but the default setup includes them, they're still there
                } else if let Some(piece) = Self::default().get(&from) {
                    if get_piece_color(Self::default().board, &from) == Some(self.player_turn)
                        && self.history_has(&from, false).is_none()
                    {
                        Some(piece)
                    } else {
                        None
                    }
                } else {
                    None
                },
            );

            self.switch_player_turn();
        }
    }

    pub fn unselect_cell(&mut self) {
        if self.is_cell_selected() {
            self.selected_coordinates = Coords::default();
            self.selected_piece_cursor = 0;
            self.cursor_coordinates = self.old_cursor_position.clone()
        }
    }

    pub fn number_of_authorized_positions(&self) -> usize {
        let mut possible_moves_count = 0;

        for i in 0..7 {
            for j in 0..7 {
                if let Some((piece_type, piece_color)) = self.board[i][j] {
                    if piece_color == self.player_turn {
                        possible_moves_count += self
                            .get_authorized_positions(
                                Some(piece_type),
                                Some(piece_color),
                                &Coords::new(i as i8, j as i8),
                            )
                            .len();
                    }
                }
            }
        }
        possible_moves_count
    }

    fn is_latest_move_en_passant(&self, from: &Coords, to: &Coords) -> bool {
        let piece_type_from = get_piece_type(self.board, from);
        let piece_type_to = get_piece_type(self.board, to);

        match (piece_type_from, piece_type_to) {
            (Some(PieceType::Pawn), _) => {
                // Check if it's a diagonal move, and the destination is an empty cell
                from.row != to.row && from.col != to.col && self.get(to).is_none()
            }
            _ => false,
        }
    }

    fn is_latest_move_castling(&self, from: &Coords, to: &Coords) -> bool {
        let piece_type_from = get_piece_type(self.board, from);
        let piece_type_to = get_piece_type(self.board, to);

        let distance = (from.col - to.col).abs();

        match (piece_type_from, piece_type_to) {
            (Some(PieceType::King), _) => distance > 1,
            _ => false,
        }
    }

    fn is_latest_move_promotion(&self) -> bool {
        if let Some(position) = self.move_history.last() {
            let to_y = chtoi(position.1.chars().nth(2));
            let to_x = chtoi(position.1.chars().nth(3));
            let to = Coords::new(to_y, to_x);

            if let Some(piece_type_from) = get_piece_type(self.board, &to) {
                if let Some(piece_color) = get_piece_color(self.board, &to) {
                    let last_row = if piece_color == PieceColor::White {
                        0
                    } else {
                        7
                    };

                    if to_y == last_row && piece_type_from == PieceType::Pawn {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_checkmate(&self) -> bool {
        if !is_getting_checked(self.board, self.player_turn, &self.move_history) {
            return false;
        }

        self.number_of_authorized_positions() == 0
    }

    pub fn draw_by_repetition(&self) -> bool {
        if self.move_history.len() >= 9 {
            let last_ten: Vec<HistRec> = self.move_history.iter().rev().take(9).cloned().collect();

            if (last_ten[0].clone(), last_ten[1].clone())
                == (last_ten[4].clone(), last_ten[5].clone())
                && last_ten[4].clone() == last_ten[8].clone()
                && (last_ten[2].clone(), last_ten[3].clone())
                    == (last_ten[6].clone(), last_ten[7].clone())
            {
                return true;
            }
        }
        false
    }

    pub fn is_draw(&self) -> bool {
        self.number_of_authorized_positions() == 0
            || self.consecutive_non_pawn_or_capture == 50
            || self.draw_by_repetition()
    }

    // Method to render the board
    pub fn board_render(&self, area: Rect, frame: &mut Frame) {
        let width = area.width / 8;
        let height = area.height / 8;
        let border_height = area.height / 2 - (4 * height);
        let border_width = area.width / 2 - (4 * width);
        // We have 8 vertical lines
        let columns = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    // spread the excess border
                    Constraint::Length(border_height),
                    Constraint::Length(height),
                    Constraint::Length(height),
                    Constraint::Length(height),
                    Constraint::Length(height),
                    Constraint::Length(height),
                    Constraint::Length(height),
                    Constraint::Length(height),
                    Constraint::Length(height),
                    Constraint::Length(border_height),
                ]
                .as_ref(),
            )
            .split(area);

        // For each line we set 8 layout
        for i in 0..8i8 {
            let lines = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(border_width),
                        Constraint::Length(width),
                        Constraint::Length(width),
                        Constraint::Length(width),
                        Constraint::Length(width),
                        Constraint::Length(width),
                        Constraint::Length(width),
                        Constraint::Length(width),
                        Constraint::Length(width),
                        Constraint::Length(border_width),
                    ]
                    .as_ref(),
                )
                .split(columns[i as usize + 1]);
            for j in 0..8i8 {
                // Color of the cell to draw the board
                let mut cell_color: Color = if (i + j) % 2 == 0 { WHITE } else { BLACK };

                // Draw the available moves for the selected piece
                if self.is_cell_selected() {
                    let selected_piece_type =
                        get_piece_type(self.board, &self.selected_coordinates);
                    let selected_piece_color: Option<PieceColor> =
                        get_piece_color(self.board, &self.selected_coordinates);
                    let positions = self.get_authorized_positions(
                        selected_piece_type,
                        selected_piece_color,
                        &self.selected_coordinates,
                    );

                    // Draw grey if the color is in the authorized positions
                    for coords in positions.clone() {
                        if i == coords.row && j == coords.col {
                            cell_color = Color::Rgb(100, 100, 100)
                        }
                    }
                }

                let square = lines[j as usize + 1];
                // Draw the cell blue if this is the current cursor cell
                if i == self.cursor_coordinates.row && j == self.cursor_coordinates.col {
                    let cell = Block::default().bg(Color::LightBlue);
                    frame.render_widget(cell.clone(), square);
                } else if is_getting_checked(self.board, self.player_turn, &self.move_history)
                    && Coords::new(i, j) == get_king_coordinates(self.board, self.player_turn)
                {
                    let cell = Block::default()
                        .bg(Color::Magenta)
                        .add_modifier(Modifier::SLOW_BLINK);
                    frame.render_widget(cell.clone(), square);
                }
                // Draw the cell green if this is the selected cell
                else if i == self.selected_coordinates.row && j == self.selected_coordinates.col {
                    let cell = Block::default().bg(Color::LightGreen);
                    frame.render_widget(cell.clone(), square);
                } else {
                    let cell = Block::default().bg(cell_color);
                    frame.render_widget(cell.clone(), square);
                }

                // We check if the current king is getting checked

                // Get piece and color
                let piece_color = get_piece_color(self.board, &Coords::new(i, j));
                let piece_type = get_piece_type(self.board, &Coords::new(i, j));

                let color_enum = color_to_ratatui_enum(piece_color);
                let piece_enum = PieceType::piece_type_to_string_enum(piece_type);

                // Place the pieces on the board
                let paragraph = Paragraph::new(piece_enum)
                    .alignment(Alignment::Center)
                    .fg(color_enum);
                frame.render_widget(paragraph, square);
            }
        }
    }

    pub fn history_render(&self, area: Rect, frame: &mut Frame) {
        // We write the history board on the side
        let history_block = Block::default()
            .title("History")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(WHITE))
            .border_type(BorderType::Rounded)
            .padding(Padding::new(5, 10, 1, 2));

        let mut lines: Vec<Line> = vec![];

        for i in (0..self.move_history.len()).step_by(2) {
            let piece_type_from = self.move_history[i].0;
            let number_move = &self.move_history[i].1;

            let utf_icon_white =
                PieceType::piece_to_utf_enum(piece_type_from, Some(PieceColor::White));
            let move_white = convert_position_into_notation(number_move.to_string());

            let mut utf_icon_black = "   ";
            let mut move_black: String = "   ".to_string();

            // If there is something for black
            if i + 1 < self.move_history.len() {
                let piece_type_to = self.move_history[i + 1].0;
                let number = &self.move_history[i + 1].1;

                move_black = convert_position_into_notation(number.to_string());
                utf_icon_black =
                    PieceType::piece_to_utf_enum(piece_type_to, Some(PieceColor::Black))
            }

            lines.push(Line::from(vec![
                Span::raw(format!("{}.  ", i / 2 + 1)), // line number
                Span::styled(format!("{} ", utf_icon_white), Style::default().fg(WHITE)), // white symbol
                Span::raw(move_white.to_string()), // white move
                Span::raw("     "),                // separator
                Span::styled(format!("{} ", utf_icon_black), Style::default().fg(WHITE)), // white symbol
                Span::raw(move_black.to_string()), // black move
            ]));
        }

        let history_paragraph = Paragraph::new(lines).alignment(Alignment::Center);

        let height = area.height;

        let right_panel_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(height - 1), Constraint::Length(1)].as_ref())
            .split(area);

        frame.render_widget(history_block.clone(), right_panel_layout[0]);
        frame.render_widget(
            history_paragraph,
            history_block.inner(right_panel_layout[0]),
        );

        // Bottom paragraph help text
        let text = vec![Line::from("Press ? for help").alignment(Alignment::Center)];

        let help_paragraph = Paragraph::new(text)
            .block(Block::new())
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, right_panel_layout[1]);
    }

    /// convert board from matrix (arrays) to vector
    fn mtov(&self) -> Vec<(PieceType, PieceColor, Coords)> {
        let mut pieces = Vec::new();
        for (i, row) in self.board.iter().enumerate() {
            for (j, piece) in row.iter().flatten().enumerate() {
                pieces.push((piece.0, piece.1, Coords::new(i as i8, j as i8)));
            }
        }
        pieces
    }

    fn can_move_to(
        &self,
        to: &Coords,
        be_color: PieceColor,
        be_type: Option<PieceType>,
        be_col: Option<char>,
        be_row: Option<i8>,
    ) -> Coords {
        if let Some(pt) = be_type {
            dbg!(pt);
        }
        let mut can_go_to = Vec::new();
        for piece in self.mtov() {
            if be_color == piece.1 && (be_type.is_some_and(|pt| pt == piece.0) || be_type.is_none())
            {
                dbg!(&piece);
            }
            if self
                .get_authorized_positions(Some(piece.0), Some(piece.1), &piece.2)
                .contains(to)
                && piece.1 == be_color
                && (be_type.is_some_and(|pt| pt == piece.0) || be_type.is_none())
                && (be_col.is_some_and(|c| letter_to_col(Some(c)) == piece.2.col)
                    || be_col.is_none())
                && (be_row.is_some_and(|r| r == piece.2.row) || be_row.is_none())
            {
                can_go_to.push(piece.2);
            }
        }
        assert_eq!(can_go_to.len(), 1);
        can_go_to[0].clone()
    }

    /// example
    /// 1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7
    pub fn pgn_import(pgn: &str) -> Result<Board, Box<dyn Error>> {
        let pgn_moves = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7";
        dbg!(&pgn_moves);

        let mut board = Board::default();

        let n = 10;
        for i in 1..n + 1 {
            dbg!(&board);
            let start_pos = pgn_moves
                .find(&format!("{}.", i))
                .expect("invalid round number");

            let round = pgn_moves.chars().skip(start_pos).collect::<String>();
            let mut round = round.split(' ');

            let round_n = round.next().expect("round does not contain num");
            dbg!(round_n);
            assert_eq!(round_n, format!("{}.", i));

            let mut w = round
                .next()
                .expect("round does not contain white's move")
                .to_owned();
            dbg!(&w);

            let (w_to, w_to_type, w_to_col, w_to_row) = if w.chars().count() == 2 {
                (Coords::from_basic_san(&w), None, None, None)
            } else if w.contains('O') {
                if w == "O-O" {
                    todo!("white castle kingside")
                } else if w == "O-O-O" {
                    todo!("white castle kingside")
                } else {
                    unreachable!("invalid white castle")
                }
            } else {
                w = w.replace('x', "");
                w = w.replace('+', "");
                (
                    Coords::from_basic_san(&w[w.len() - 2..w.len()]),
                    Some(PieceType::from_char(w.chars().next().unwrap()).unwrap().0),
                    None,
                    None,
                )
            };
            dbg!(&w_to);

            let w_from = board.can_move_to(&w_to, PieceColor::White, w_to_type, w_to_col, w_to_row);
            dbg!(&w_from);
            board.set(&w_to, board.get(&w_from));
            board.set(&w_from, None);
            // board.move_piece(&from_w, &w_to);

            dbg!(&board);
            let mut b = round
                .next()
                .expect("round does not contain black's move")
                .to_owned();
            dbg!(&b);
            let (b_to, b_to_type, b_to_col, b_to_row) = if b.chars().count() == 2 {
                (Coords::from_basic_san(&b), None, None, None)
            } else if b.contains('O') {
                if b == "O-O" {
                    todo!("black castle kingside")
                } else if b == "O-O-O" {
                    todo!("black castle kingside")
                } else {
                    unreachable!("invalid black castle")
                }
            } else {
                b = b.replace('x', "");
                b = b.replace('+', "");
                (
                    Coords::from_basic_san(&b[b.len() - 2..b.len()]),
                    Some(PieceType::from_char(b.chars().next().unwrap()).unwrap().0),
                    None,
                    None,
                )
            };
            dbg!(&b_to);

            let b_from = board.can_move_to(&b_to, PieceColor::Black, b_to_type, b_to_col, b_to_row);
            dbg!(&b_from);
            board.set(&b_to, board.get(&b_from));
            board.set(&b_from, None);
            // board.move_piece(&from_b, &b_to);

            // todo!("fn")
        }

        todo!("pgn import")
    }
}
impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for row in self.board {
            for piece in row {
                if let Some(p) = piece {
                    write!(f, "{}", PieceType::piece_to_utf_enum(p.0, Some(p.1)))?;
                } else {
                    write!(f, " ")?;
                }
                write!(f, " ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, Coords},
        constants::UNDEFINED_POSITION,
        pieces::{PieceColor, PieceType},
        utils::is_getting_checked,
    };

    #[test]
    fn is_getting_checked_true() {
        let custom_board = [
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Rook, PieceColor::Black)),
                None,
                None,
                None,
            ],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let mut board = Board::default();
        board.set_board(custom_board);

        assert!(is_getting_checked(custom_board, PieceColor::White, &[]));
    }

    #[test]
    fn is_getting_checked_false() {
        let custom_board = [
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Rook, PieceColor::Black)),
                None,
                None,
                None,
            ],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Pawn, PieceColor::White)),
                None,
                None,
                None,
            ],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let mut board = Board::default();
        board.set_board(custom_board);

        assert!(!is_getting_checked(custom_board, PieceColor::White, &[]));
    }

    #[test]
    fn is_getting_checked_piece_in_front_false() {
        let custom_board = [
            [
                Some((PieceType::Rook, PieceColor::Black)),
                Some((PieceType::Knight, PieceColor::Black)),
                Some((PieceType::Bishop, PieceColor::Black)),
                Some((PieceType::Queen, PieceColor::Black)),
                Some((PieceType::King, PieceColor::Black)),
                None,
                None,
                Some((PieceType::Rook, PieceColor::Black)),
            ],
            [
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                None,
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
            ],
            [
                Some((PieceType::Rook, PieceColor::White)),
                Some((PieceType::Knight, PieceColor::White)),
                Some((PieceType::Bishop, PieceColor::White)),
                Some((PieceType::Queen, PieceColor::White)),
                Some((PieceType::Rook, PieceColor::White)),
                Some((PieceType::Bishop, PieceColor::White)),
                None,
                Some((PieceType::King, PieceColor::White)),
            ],
        ];
        let mut board = Board::default();
        board.set_board(custom_board);

        assert!(!is_getting_checked(custom_board, PieceColor::Black, &[]));
    }

    #[test]
    fn is_getting_checked_piece_in_with_gap_false() {
        let custom_board = [
            [
                Some((PieceType::Rook, PieceColor::Black)),
                Some((PieceType::Knight, PieceColor::Black)),
                Some((PieceType::Bishop, PieceColor::Black)),
                Some((PieceType::Queen, PieceColor::Black)),
                Some((PieceType::King, PieceColor::Black)),
                None,
                None,
                Some((PieceType::Rook, PieceColor::Black)),
            ],
            [
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                None,
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
            ],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Pawn, PieceColor::Black)),
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                None,
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
            ],
            [
                Some((PieceType::Rook, PieceColor::White)),
                Some((PieceType::Knight, PieceColor::White)),
                Some((PieceType::Bishop, PieceColor::White)),
                Some((PieceType::Queen, PieceColor::White)),
                Some((PieceType::Rook, PieceColor::White)),
                Some((PieceType::Bishop, PieceColor::White)),
                None,
                Some((PieceType::King, PieceColor::White)),
            ],
        ];
        let mut board = Board::default();
        board.set_board(custom_board);

        assert!(!is_getting_checked(custom_board, PieceColor::Black, &[]));
    }

    #[test]
    fn is_checkmate_true() {
        let custom_board = [
            [
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [
                None,
                Some((PieceType::Rook, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                Some((PieceType::Queen, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let board = Board::new(custom_board, PieceColor::White, vec![]);

        assert!(board.is_checkmate());
    }

    #[test]
    fn is_checkmate_false() {
        let custom_board = [
            [
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [
                None,
                Some((PieceType::Rook, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                None,
                None,
                None,
                Some((PieceType::Queen, PieceColor::Black)),
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let board = Board::new(custom_board, PieceColor::White, vec![]);

        assert!(!board.is_checkmate());
    }

    #[test]
    fn is_checkmate_false_2() {
        let custom_board = [
            [
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                None,
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::Queen, PieceColor::White)),
                None,
            ],
            [
                None,
                Some((PieceType::Rook, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                Some((PieceType::Queen, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let board = Board::new(custom_board, PieceColor::White, vec![]);

        assert!(!board.is_checkmate());
    }

    #[test]
    fn is_draw_true() {
        let custom_board = [
            [
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                None,
                None,
                Some((PieceType::Queen, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                None,
                Some((PieceType::Rook, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let board = Board::new(custom_board, PieceColor::White, vec![]);

        assert!(board.is_draw());
    }

    #[test]
    fn is_draw_false() {
        let custom_board = [
            [
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Queen, PieceColor::Black)),
                None,
                None,
                None,
            ],
            [
                None,
                None,
                Some((PieceType::Rook, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let board = Board::new(custom_board, PieceColor::White, vec![]);

        assert!(!board.is_draw());
    }

    #[test]
    fn is_promote_false() {
        let custom_board = [
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::King, PieceColor::Black)),
            ],
            [
                None,
                None,
                None,
                Some((PieceType::Rook, PieceColor::White)),
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                Some((PieceType::Pawn, PieceColor::White)),
                None,
                None,
                None,
                None,
            ],
            [
                None,
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ];
        let board = Board::new(
            custom_board,
            PieceColor::Black,
            vec![(PieceType::Pawn, "7363".to_string())],
        );

        assert!(!board.is_latest_move_promotion());
    }
    #[test]
    fn is_promote_true() {
        let custom_board = [
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Pawn, PieceColor::White)),
                None,
                None,
                Some((PieceType::King, PieceColor::Black)),
            ],
            [
                None,
                None,
                None,
                Some((PieceType::Rook, PieceColor::White)),
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ];
        let board = Board::new(
            custom_board,
            PieceColor::Black,
            vec![(PieceType::Pawn, "1404".to_string())],
        );

        assert!(board.is_latest_move_promotion());
    }

    #[test]
    fn promote_and_checkmate() {
        let custom_board = [
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::King, PieceColor::Black)),
            ],
            [
                None,
                None,
                None,
                Some((PieceType::Rook, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ];
        // We setup the board
        let mut board = Board::new(custom_board, PieceColor::White, vec![]);
        assert!(!board.is_latest_move_promotion());

        // Move the pawn to a promote cell
        board.move_piece(&Coords::new(1, 4), &Coords::new(0, 4));
        assert!(board.is_latest_move_promotion());

        // Promote the pawn
        board.promote_piece();

        // The black king gets checkmated
        board.player_turn = PieceColor::Black;
        assert!(board.is_checkmate());
    }

    #[test]
    fn is_promote_true_black() {
        let custom_board = [
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                Some((PieceType::King, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                None,
                None,
                None,
                Some((PieceType::Rook, PieceColor::Black)),
                None,
                None,
                None,
                None,
            ],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Pawn, PieceColor::Black)),
                None,
                None,
                Some((PieceType::King, PieceColor::White)),
            ],
        ];
        let board = Board::new(
            custom_board,
            PieceColor::White,
            vec![(PieceType::Pawn, "6474".to_string())],
        );

        assert!(board.is_latest_move_promotion());
    }

    #[test]
    fn promote_and_draw() {
        let custom_board = [
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::Rook, PieceColor::Black)),
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                Some((PieceType::King, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::Pawn, PieceColor::Black)),
                None,
                Some((PieceType::King, PieceColor::White)),
            ],
            [None, None, None, None, None, None, None, None],
        ];
        // We setup the board
        let mut board = Board::new(custom_board, PieceColor::Black, vec![]);
        assert!(!board.is_latest_move_promotion());

        // Move the pawn to a promote cell
        board.move_piece(&Coords::new(6, 5), &Coords::new(7, 5));
        assert!(board.is_latest_move_promotion());

        // Promote the pawn
        board.promote_piece();

        // The black king gets checkmated
        board.player_turn = PieceColor::White;
        assert!(board.is_draw());
    }
    #[test]
    fn fifty_moves_draw() {
        let custom_board = [
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
                Some((PieceType::King, PieceColor::Black)),
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        // We setup the board
        let mut board = Board::new(
            custom_board,
            PieceColor::White,
            vec![
                // We don't use the history for a fifty draw
            ],
        );

        board.consecutive_non_pawn_or_capture = 49;
        assert!(!board.is_draw());

        // Move the pawn to a make the 50th move
        board.move_piece(&Coords::new(0, 6), &Coords::new(0, 5));
        assert!(board.is_draw());
    }

    #[test]
    fn consecutive_position_draw() {
        let custom_board = [
            [
                None,
                None,
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
                Some((PieceType::King, PieceColor::Black)),
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        // We setup the board
        let mut board = Board::new(
            custom_board,
            PieceColor::White,
            vec![
                (PieceType::King, "0201".to_string()),
                (PieceType::King, "0605".to_string()),
                (PieceType::King, "0102".to_string()),
                (PieceType::King, "0506".to_string()),
                (PieceType::King, "0201".to_string()),
                (PieceType::King, "0605".to_string()),
                (PieceType::King, "0102".to_string()),
                (PieceType::King, "0506".to_string()),
            ],
        );

        assert!(!board.is_draw());

        // Move the king to replicate a third time the same position
        board.move_piece(&Coords::new(0, 2), &Coords::new(0, 1));
        assert!(board.is_draw());
    }

    #[test]
    fn fen_converter_1() {
        let custom_board = [
            [
                None,
                None,
                Some((PieceType::King, PieceColor::Black)),
                None,
                None,
                None,
                None,
                Some((PieceType::Rook, PieceColor::White)),
            ],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        // We setup the board
        let board = Board::new(custom_board, PieceColor::White, vec![]);

        // Move the king to replicate a third time the same position
        assert_eq!(board.fen_position(), "2k4R/8/4K3/8/8/8/8/8 b - - 0 0");
    }

    #[test]
    fn fen_converter_en_passant() {
        let custom_board = [
            [
                None,
                None,
                Some((PieceType::King, PieceColor::Black)),
                None,
                None,
                None,
                None,
                Some((PieceType::Rook, PieceColor::White)),
            ],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                Some((PieceType::Pawn, PieceColor::White)),
                None,
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        // We setup the board
        let board = Board::new(
            custom_board,
            PieceColor::White,
            vec![(PieceType::Pawn, "6242".to_string())],
        );

        // Move the king to replicate a third time the same position
        assert_eq!(board.fen_position(), "2k4R/8/4K3/8/2P5/8/8/8 b - c3 0 0");
    }
    #[test]
    fn fen_converter_castling() {
        let custom_board = [
            [
                Some((PieceType::Rook, PieceColor::Black)),
                Some((PieceType::Knight, PieceColor::Black)),
                Some((PieceType::Bishop, PieceColor::Black)),
                Some((PieceType::Queen, PieceColor::Black)),
                Some((PieceType::King, PieceColor::Black)),
                Some((PieceType::Bishop, PieceColor::Black)),
                Some((PieceType::Knight, PieceColor::Black)),
                Some((PieceType::Rook, PieceColor::Black)),
            ],
            [
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
            ],
            [
                Some((PieceType::Rook, PieceColor::White)),
                Some((PieceType::Knight, PieceColor::White)),
                Some((PieceType::Bishop, PieceColor::White)),
                Some((PieceType::Queen, PieceColor::White)),
                Some((PieceType::King, PieceColor::White)),
                Some((PieceType::Bishop, PieceColor::White)),
                Some((PieceType::Knight, PieceColor::White)),
                Some((PieceType::Rook, PieceColor::White)),
            ],
        ];
        // We setup the board
        let board = Board::new(custom_board, PieceColor::White, vec![]);

        // Move the king to replicate a third time the same position
        assert_eq!(
            board.fen_position(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b kq - 0 0"
        );
    }

    #[test]
    fn takeback_basic() {
        let mut board = Board::default();
        board.move_piece(&Coords { col: 4, row: 6 }, &Coords { col: 4, row: 4 });
        assert_ne!(Board::default().board, board.board);
        board.takeback();
        assert_eq!(Board::default().board, board.board);
    }

    #[test]
    #[ignore]
    fn takeback_kick() {
        let mut board = Board::default();
        board.move_piece(&Coords { col: 4, row: 6 }, &Coords { col: 4, row: 4 });
        board.move_piece(&Coords { col: 3, row: 1 }, &Coords { col: 3, row: 3 });
        board.move_piece(&Coords { col: 4, row: 4 }, &Coords { col: 3, row: 3 });
        assert_ne!(Board::default().board, board.board);
        board.takeback();
        board.takeback();
        board.takeback();
        assert_eq!(Board::default().board, board.board);
    }

    #[test]
    #[ignore]
    fn takeback_en_passant() {
        let mut board = Board::default();
        board.move_piece(&Coords { col: 4, row: 6 }, &Coords { col: 4, row: 4 });
        board.move_piece(&Coords { col: 5, row: 1 }, &Coords { col: 5, row: 3 });
        board.move_piece(&Coords { col: 4, row: 4 }, &Coords { col: 4, row: 3 });
        board.move_piece(&Coords { col: 3, row: 1 }, &Coords { col: 3, row: 3 });
        board.move_piece(&Coords { col: 4, row: 4 }, &Coords { col: 3, row: 3 });
        assert_ne!(Board::default().board, board.board);
        board.takeback();
        board.takeback();
        board.takeback();
        board.takeback();
        board.takeback();
        assert_eq!(Board::default().board, board.board);
    }

    // #[test]
    // fn takeback_castle() {
    // }

    #[test]
    fn coords_new_min() {
        assert_eq!(Coords { col: 0, row: 0 }, Coords::new(0, 0));
    }

    #[test]
    fn coords_new_max() {
        assert_eq!(Coords { col: 7, row: 7 }, Coords::new(7, 7));
    }

    #[test]
    fn coords_new() {
        assert_eq!(Coords { col: 6, row: 1 }, Coords::new(1, 6));
    }

    #[test]
    #[ignore]
    #[should_panic]
    fn coords_new_too_big() {
        Coords::new(8, 8);
    }

    #[test]
    #[ignore]
    #[should_panic]
    fn coords_new_too_small() {
        Coords::new(-1, -1);
    }

    #[test]
    fn coords_default() {
        assert_eq!(
            Coords {
                col: UNDEFINED_POSITION,
                row: UNDEFINED_POSITION
            },
            Coords::default()
        );
    }

    #[test]
    fn notation_to_coords_0() {
        let nt = "a1";
        assert_eq!(Coords { col: 0, row: 7 }, Coords::from_basic_san(nt));
    }
    #[test]
    fn notation_to_coords_1() {
        let nt = "h1";
        assert_eq!(Coords { col: 7, row: 7 }, Coords::from_basic_san(nt));
    }
    #[test]
    fn notation_to_coords_2() {
        let nt = "a8";
        assert_eq!(Coords { col: 0, row: 0 }, Coords::from_basic_san(nt));
    }
    #[test]
    fn notation_to_coords_3() {
        let nt = "h8";
        assert_eq!(Coords { col: 7, row: 0 }, Coords::from_basic_san(nt));
    }
    #[test]
    #[should_panic]
    fn notation_to_coords_4() {
        let nt = "h9";
        Coords::from_basic_san(nt);
    }
    #[test]
    #[should_panic]
    fn notation_to_coords_5() {
        let nt = "i0";
        Coords::from_basic_san(nt);
    }
    #[test]
    fn notation_to_coords_6() {
        let nt = "e4";
        assert_eq!(Coords { col: 4, row: 4 }, Coords::from_basic_san(nt));
    }
    #[test]
    fn notation_to_coords_7() {
        let nt = "e5";
        assert_eq!(Coords { col: 4, row: 3 }, Coords::from_basic_san(nt));
    }

    #[test]
    fn can_move_to_0() {
        let board = Board::default();
        let to_nt = "e4";
        let to = Coords::from_basic_san(to_nt);

        let from = board.can_move_to(&to, PieceColor::White, None, None, None);

        let from_nt = "e2";
        assert_eq!(Coords::from_basic_san(from_nt), from);
    }
    #[test]
    fn can_move_to_1() {
        let board = Board::default();
        let to_nt = "d4";
        let to = Coords::from_basic_san(to_nt);

        let from = board.can_move_to(&to, PieceColor::White, None, None, None);

        let from_nt = "d2";
        assert_eq!(Coords::from_basic_san(from_nt), from);
    }
    #[test]
    fn can_move_to_2() {
        let board = Board::default();
        let to_nt = "Nc3";
        let to = Coords::from_basic_san(&to_nt[1..3]);

        let from = board.can_move_to(&to, PieceColor::White, Some(PieceType::Knight), None, None);

        let from_nt = "b1";
        assert_eq!(Coords::from_basic_san(from_nt), from);
    }

    #[test]
    fn auth_pos_0() {
        let board = Board::default();

        let auth_pos = board.get_authorized_positions(
            Some(PieceType::King),
            Some(PieceColor::White),
            &Coords { col: 4, row: 7 },
        );
        assert_eq!(0, auth_pos.len());
    }
}
