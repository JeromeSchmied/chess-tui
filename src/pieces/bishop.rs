use super::{Movable, PieceColor, PieceType, Position};
use crate::{
    board::Coord,
    utils::{
        cleaned_positions, get_piece_color, impossible_positions_king_checked, is_cell_color_ally,
        is_piece_opposite_king, is_valid,
    },
};
pub struct Bishop;

impl Movable for Bishop {
    fn piece_move(
        coordinates: Coord,
        color: PieceColor,
        board: [[Option<(PieceType, PieceColor)>; 8]; 8],
        allow_move_on_ally_positions: bool,
        _move_history: &[(Option<PieceType>, String)],
    ) -> Vec<Coord> {
        let mut positions: Vec<Coord> = vec![];

        let y = coordinates.row;
        let x = coordinates.col;

        // for diagonal from piece to top left
        for i in 1..8i8 {
            let new_x = x - i;
            let new_y = y - i;
            let new_coordinates = Coord::new(new_y, new_x);

            // Invalid coords
            if !is_valid(&new_coordinates) {
                break;
            }

            // Empty cell
            if get_piece_color(board, &new_coordinates).is_none() {
                positions.push(new_coordinates);
                continue;
            }
            // Ally cell
            if is_cell_color_ally(board, new_coordinates.clone(), color) {
                if !allow_move_on_ally_positions {
                    break;
                } else {
                    positions.push(new_coordinates);
                    break;
                }
            }

            // Enemy cell
            positions.push(new_coordinates.clone());
            if !allow_move_on_ally_positions
                || !is_piece_opposite_king(board[new_y as usize][new_x as usize], color)
            {
                break;
            }
        }

        // for diagonal from piece to bottom right
        for i in 1..8i8 {
            let new_x = x + i;
            let new_y = y + i;

            let new_coordinates = Coord::new(new_y, new_x);

            // Invalid coords
            if !is_valid(&new_coordinates) {
                break;
            }

            // Empty cell
            if get_piece_color(board, &new_coordinates).is_none() {
                positions.push(new_coordinates);
                continue;
            }
            // Ally cell
            if is_cell_color_ally(board, new_coordinates.clone(), color) {
                if !allow_move_on_ally_positions {
                    break;
                } else {
                    positions.push(new_coordinates);
                    break;
                }
            }

            // Enemy cell
            positions.push(new_coordinates.clone());
            if !allow_move_on_ally_positions
                || !is_piece_opposite_king(board[new_y as usize][new_x as usize], color)
            {
                break;
            }
        }

        // for diagonal from piece to bottom left
        for i in 1..8i8 {
            let new_x = x - i;
            let new_y = y + i;
            let new_coordinates = Coord::new(new_y, new_x);

            // Invalid coords
            if !is_valid(&new_coordinates) {
                break;
            }

            // Empty cell
            if get_piece_color(board, &new_coordinates).is_none() {
                positions.push(new_coordinates);
                continue;
            }
            // Ally cell
            if is_cell_color_ally(board, new_coordinates.clone(), color) {
                if !allow_move_on_ally_positions {
                    break;
                } else {
                    positions.push(new_coordinates);
                    break;
                }
            }

            // Enemy cell
            positions.push(new_coordinates);
            if !allow_move_on_ally_positions
                || !is_piece_opposite_king(board[new_y as usize][new_x as usize], color)
            {
                break;
            }
        }

        // for diagonal from piece to top right
        for i in 1..8i8 {
            let new_x = x + i;
            let new_y = y - i;
            let new_coordinates = Coord::new(new_y, new_x);

            // Invalid coords
            if !is_valid(&new_coordinates) {
                break;
            }

            // Empty cell
            if get_piece_color(board, &new_coordinates).is_none() {
                positions.push(new_coordinates);
                continue;
            }
            // Ally cell
            if is_cell_color_ally(board, new_coordinates.clone(), color) {
                if !allow_move_on_ally_positions {
                    break;
                } else {
                    positions.push(new_coordinates);
                    break;
                }
            }

            // Enemy cell
            positions.push(new_coordinates);
            if !allow_move_on_ally_positions
                || !is_piece_opposite_king(board[new_y as usize][new_x as usize], color)
            {
                break;
            }
        }
        cleaned_positions(positions)
    }
}

impl Position for Bishop {
    fn authorized_positions(
        coordinates: Coord,
        color: PieceColor,
        board: [[Option<(PieceType, PieceColor)>; 8]; 8],
        move_history: &[(Option<PieceType>, String)],
        _is_king_checked: bool,
    ) -> Vec<Coord> {
        // if the king is checked we clean all the position not resolving the check
        impossible_positions_king_checked(
            &coordinates,
            Self::piece_move(coordinates.clone(), color, board, false, move_history),
            board,
            color,
            move_history,
        )
    }
    fn protected_positions(
        coordinates: Coord,
        color: PieceColor,
        board: [[Option<(PieceType, PieceColor)>; 8]; 8],
        move_history: &[(Option<PieceType>, String)],
    ) -> Vec<Coord> {
        Self::piece_move(coordinates, color, board, true, move_history)
    }
}

impl Bishop {
    pub fn to_string() -> &'static str {
        "\
    \n\
       ⭘\n\
      █✝█\n\
      ███\n\
    ▗█████▖\n\
    "
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, Coord},
        pieces::{bishop::Bishop, PieceColor, PieceType, Position},
        utils::is_getting_checked,
    };

    #[test]
    fn piece_move_no_enemies() {
        let custom_board = [
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Bishop, PieceColor::White)),
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

        let mut right_positions = vec![
            Coord::new(0, 0),
            Coord::new(1, 1),
            Coord::new(2, 2),
            Coord::new(3, 3),
            Coord::new(5, 5),
            Coord::new(6, 6),
            Coord::new(7, 7),
            Coord::new(1, 7),
            Coord::new(2, 6),
            Coord::new(3, 5),
            Coord::new(5, 3),
            Coord::new(6, 2),
            Coord::new(7, 1),
        ];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            Coord::new(4, 4),
            PieceColor::White,
            board.board,
            &[],
            false,
        );
        positions.sort();

        assert_eq!(right_positions, positions);
    }

    #[test]
    fn piece_move_one_enemies_top_right() {
        let custom_board = [
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::Pawn, PieceColor::Black)),
                None,
                None,
            ],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Rook, PieceColor::White)),
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

        let mut right_positions = vec![
            Coord::new(0, 0),
            Coord::new(1, 1),
            Coord::new(2, 2),
            Coord::new(3, 3),
            Coord::new(5, 5),
            Coord::new(6, 6),
            Coord::new(7, 7),
            Coord::new(3, 5),
            Coord::new(5, 3),
            Coord::new(6, 2),
            Coord::new(7, 1),
        ];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            Coord::new(4, 4),
            PieceColor::White,
            board.board,
            &[],
            false,
        );
        positions.sort();

        assert_eq!(right_positions, positions);
    }

    #[test]
    fn piece_move_multiple_enemies_and_ally_front() {
        let custom_board = [
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                Some((PieceType::Pawn, PieceColor::Black)),
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
                Some((PieceType::Bishop, PieceColor::White)),
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
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [
                None,
                Some((PieceType::Bishop, PieceColor::White)),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ];
        let mut board = Board::default();
        board.set_board(custom_board);

        let mut right_positions = vec![
            Coord::new(3, 3),
            Coord::new(5, 5),
            Coord::new(3, 5),
            Coord::new(2, 6),
            Coord::new(1, 7),
            Coord::new(5, 3),
            Coord::new(6, 2),
        ];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            Coord::new(4, 4),
            PieceColor::White,
            board.board,
            &[],
            false,
        );
        positions.sort();

        assert_eq!(right_positions, positions);
    }

    #[test]
    fn king_checked_can_resolve() {
        let custom_board = [
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                Some((PieceType::King, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Bishop, PieceColor::White)),
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
                Some((PieceType::Bishop, PieceColor::Black)),
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let mut board = Board::new(custom_board, PieceColor::Black, vec![]);
        board.set_board(custom_board);

        let is_king_checked =
            is_getting_checked(board.board, board.player_turn, &board.move_history);

        let mut right_positions = vec![Coord::new(4, 4)];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            Coord::new(5, 5),
            PieceColor::Black,
            board.board,
            &[],
            is_king_checked,
        );
        positions.sort();

        assert_eq!(right_positions, positions);
    }

    #[test]
    fn king_checked_cant_resolve() {
        let custom_board = [
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                Some((PieceType::King, PieceColor::Black)),
                None,
                None,
                None,
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Bishop, PieceColor::White)),
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
                Some((PieceType::Bishop, PieceColor::Black)),
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let mut board = Board::new(custom_board, PieceColor::Black, vec![]);
        board.set_board(custom_board);

        let is_king_checked =
            is_getting_checked(board.board, board.player_turn, &board.move_history);

        let mut right_positions: Vec<Coord> = vec![];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            Coord::new(5, 6),
            PieceColor::Black,
            board.board,
            &[],
            is_king_checked,
        );
        positions.sort();

        assert_eq!(right_positions, positions);
    }

    #[test]
    fn nailing() {
        let custom_board = [
            [
                None,
                None,
                None,
                None,
                Some((PieceType::King, PieceColor::Black)),
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
                Some((PieceType::Bishop, PieceColor::Black)),
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::Queen, PieceColor::White)),
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let mut board = Board::new(custom_board, PieceColor::Black, vec![]);
        board.set_board(custom_board);

        let is_king_checked =
            is_getting_checked(board.board, board.player_turn, &board.move_history);

        let mut right_positions: Vec<Coord> = vec![Coord::new(2, 6), Coord::new(3, 7)];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            Coord::new(1, 5),
            PieceColor::Black,
            board.board,
            &[],
            is_king_checked,
        );
        positions.sort();

        assert_eq!(right_positions, positions);
    }
}
