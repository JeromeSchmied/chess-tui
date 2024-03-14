use super::{Movable, PieceColor, PieceKind, Position};
use crate::{
    board::GameBoard,
    notations::Coords,
    utils::{
        cleaned_positions, get_piece_color, impossible_positions_king_checked, is_cell_color_ally,
        is_piece_opposite_king, is_valid,
    },
};
pub struct Bishop;

impl Movable for Bishop {
    fn piece_move(
        coordinates: &Coords,
        color: PieceColor,
        board: GameBoard,
        allow_move_on_ally_positions: bool,
        _move_history: &[(Option<PieceKind>, String)],
    ) -> Vec<Coords> {
        let mut positions: Vec<Coords> = vec![];

        let y = coordinates.row;
        let x = coordinates.col;

        // for diagonal from piece to top left
        for i in 1..8i8 {
            let new_x = x - i;
            let new_y = y - i;
            let new_coordinates = Coords::new(new_y, new_x);

            // Invalid coords
            if !is_valid(&new_coordinates) {
                break;
            }

            // Empty cell
            if get_piece_color(&board, &new_coordinates).is_none() {
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

            let new_coordinates = Coords::new(new_y, new_x);

            // Invalid coords
            if !is_valid(&new_coordinates) {
                break;
            }

            // Empty cell
            if get_piece_color(&board, &new_coordinates).is_none() {
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
            let new_coordinates = Coords::new(new_y, new_x);

            // Invalid coords
            if !is_valid(&new_coordinates) {
                break;
            }

            // Empty cell
            if get_piece_color(&board, &new_coordinates).is_none() {
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
            let new_coordinates = Coords::new(new_y, new_x);

            // Invalid coords
            if !is_valid(&new_coordinates) {
                break;
            }

            // Empty cell
            if get_piece_color(&board, &new_coordinates).is_none() {
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
        coordinates: &Coords,
        color: PieceColor,
        board: GameBoard,
        move_history: &[(Option<PieceKind>, String)],
        _is_king_checked: bool,
    ) -> Vec<Coords> {
        // if the king is checked we clean all the position not resolving the check
        impossible_positions_king_checked(
            coordinates,
            Self::piece_move(coordinates, color, board, false, move_history),
            board,
            color,
            move_history,
        )
    }
    fn protected_positions(
        coordinates: &Coords,
        color: PieceColor,
        board: GameBoard,
        move_history: &[(Option<PieceKind>, String)],
    ) -> Vec<Coords> {
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
        board::Board,
        notations::Coords,
        pieces::{bishop::Bishop, Piece, PieceColor, PieceKind, Position},
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
                Some(Piece::new(PieceKind::Bishop, PieceColor::White)),
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
            Coords::new(0, 0),
            Coords::new(1, 1),
            Coords::new(2, 2),
            Coords::new(3, 3),
            Coords::new(5, 5),
            Coords::new(6, 6),
            Coords::new(7, 7),
            Coords::new(1, 7),
            Coords::new(2, 6),
            Coords::new(3, 5),
            Coords::new(5, 3),
            Coords::new(6, 2),
            Coords::new(7, 1),
        ];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            &Coords::new(4, 4),
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
                Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
                None,
                None,
            ],
            [
                None,
                None,
                None,
                None,
                Some(Piece::new(PieceKind::Rook, PieceColor::White)),
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
            Coords::new(0, 0),
            Coords::new(1, 1),
            Coords::new(2, 2),
            Coords::new(3, 3),
            Coords::new(5, 5),
            Coords::new(6, 6),
            Coords::new(7, 7),
            Coords::new(3, 5),
            Coords::new(5, 3),
            Coords::new(6, 2),
            Coords::new(7, 1),
        ];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            &Coords::new(4, 4),
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
                Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
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
                Some(Piece::new(PieceKind::Bishop, PieceColor::White)),
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
                Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [
                None,
                Some(Piece::new(PieceKind::Bishop, PieceColor::White)),
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
            Coords::new(3, 3),
            Coords::new(5, 5),
            Coords::new(3, 5),
            Coords::new(2, 6),
            Coords::new(1, 7),
            Coords::new(5, 3),
            Coords::new(6, 2),
        ];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            &Coords::new(4, 4),
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
                Some(Piece::new(PieceKind::King, PieceColor::Black)),
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
                Some(Piece::new(PieceKind::Bishop, PieceColor::White)),
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
                Some(Piece::new(PieceKind::Bishop, PieceColor::Black)),
                None,
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let mut board = Board::new(&custom_board, PieceColor::Black, vec![]);
        board.set_board(custom_board);

        let is_king_checked =
            is_getting_checked(board.board, board.player_turn, &board.move_history);

        let mut right_positions = vec![Coords::new(4, 4)];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            &Coords::new(5, 5),
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
                Some(Piece::new(PieceKind::King, PieceColor::Black)),
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
                Some(Piece::new(PieceKind::Bishop, PieceColor::White)),
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
                Some(Piece::new(PieceKind::Bishop, PieceColor::Black)),
                None,
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let mut board = Board::new(&custom_board, PieceColor::Black, vec![]);
        board.set_board(custom_board);

        let is_king_checked =
            is_getting_checked(board.board, board.player_turn, &board.move_history);

        let mut right_positions: Vec<Coords> = vec![];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            &Coords::new(5, 6),
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
                Some(Piece::new(PieceKind::King, PieceColor::Black)),
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
                Some(Piece::new(PieceKind::Bishop, PieceColor::Black)),
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
                Some(Piece::new(PieceKind::Queen, PieceColor::White)),
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];
        let mut board = Board::new(&custom_board, PieceColor::Black, vec![]);
        board.set_board(custom_board);

        let is_king_checked =
            is_getting_checked(board.board, board.player_turn, &board.move_history);

        let mut right_positions: Vec<Coords> = vec![Coords::new(2, 6), Coords::new(3, 7)];
        right_positions.sort();

        let mut positions = Bishop::authorized_positions(
            &Coords::new(1, 5),
            PieceColor::Black,
            board.board,
            &[],
            is_king_checked,
        );
        positions.sort();

        assert_eq!(right_positions, positions);
    }
}
