use super::{Movable, PieceColor, PieceType, Position};
use crate::{
    board::Coord,
    utils::{cleaned_positions, impossible_positions_king_checked, is_cell_color_ally, is_valid},
};
pub struct Knight;

impl Movable for Knight {
    fn piece_move(
        coordinates: Coord,
        color: PieceColor,
        board: [[Option<(PieceType, PieceColor)>; 8]; 8],
        allow_move_on_ally_positions: bool,
        _move_history: &[(Option<PieceType>, String)],
    ) -> Vec<Coord> {
        let mut positions: Vec<Coord> = Vec::new();

        let (y, x) = (coordinates.row, coordinates.col);

        // Generate knight positions in all eight possible L-shaped moves
        let piece_move = [
            Coord::new(-2, -1),
            Coord::new(-2, 1),
            Coord::new(-1, -2),
            Coord::new(-1, 2),
            Coord::new(1, -2),
            Coord::new(1, 2),
            Coord::new(2, -1),
            Coord::new(2, 1),
        ];

        for &Coord { col: dx, row: dy } in &piece_move {
            let new_coordinates = Coord::new(y + dy, x + dx);

            if !is_valid(&new_coordinates) {
                continue;
            }

            if is_cell_color_ally(board, new_coordinates.clone(), color)
                && !allow_move_on_ally_positions
            {
                continue;
            }

            positions.push(new_coordinates);
        }

        cleaned_positions(positions)
    }
}

impl Position for Knight {
    fn authorized_positions(
        coordinates: Coord,
        color: PieceColor,
        board: [[Option<(PieceType, PieceColor)>; 8]; 8],
        move_history: &[(Option<PieceType>, String)],
        _is_king_checked: bool,
    ) -> Vec<Coord> {
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
        _move_history: &[(Option<PieceType>, String)],
    ) -> Vec<Coord> {
        Self::piece_move(coordinates, color, board, true, _move_history)
    }
}

impl Knight {
    pub fn to_string() -> &'static str {
        "\
    \n\
    ▟▛██▙\n\
   ▟█████\n\
   ▀▀▟██▌\n\
    ▟████\n\
    "
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, Coord},
        pieces::{knight::Knight, PieceColor, PieceType, Position},
        utils::is_getting_checked,
    };

    #[test]
    fn no_enemies() {
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
                Some((PieceType::Knight, PieceColor::White)),
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
            Coord::new(2, 3),
            Coord::new(2, 5),
            Coord::new(3, 2),
            Coord::new(3, 6),
            Coord::new(5, 2),
            Coord::new(5, 6),
            Coord::new(6, 3),
            Coord::new(6, 5),
        ];
        right_positions.sort();

        let mut positions = Knight::authorized_positions(
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
    fn enemy_and_ally() {
        let custom_board = [
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::Pawn, PieceColor::White)),
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
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::Knight, PieceColor::White)),
            ],
        ];
        let mut board = Board::default();
        board.set_board(custom_board);

        let mut right_positions = vec![Coord::new(6, 5)];
        right_positions.sort();

        let mut positions = Knight::authorized_positions(
            Coord::new(7, 7),
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
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::King, PieceColor::White)),
                None,
            ],
            [
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::Knight, PieceColor::White)),
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
                None,
                Some((PieceType::Knight, PieceColor::Black)),
            ],
        ];
        let mut board = Board::new(custom_board, PieceColor::White, vec![]);
        board.set_board(custom_board);

        let is_king_checked =
            is_getting_checked(board.board, board.player_turn, &board.move_history);

        let mut right_positions = vec![Coord::new(7, 7)];
        right_positions.sort();

        let mut positions = Knight::authorized_positions(
            Coord::new(6, 5),
            PieceColor::White,
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
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                None,
                None,
                None,
                None,
                None,
                None,
                Some((PieceType::King, PieceColor::White)),
                None,
            ],
            [
                None,
                None,
                None,
                None,
                Some((PieceType::Knight, PieceColor::White)),
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
                None,
                Some((PieceType::Knight, PieceColor::Black)),
            ],
        ];
        let mut board = Board::new(custom_board, PieceColor::White, vec![]);
        board.set_board(custom_board);

        let is_king_checked =
            is_getting_checked(board.board, board.player_turn, &board.move_history);

        let mut right_positions: Vec<Coord> = vec![];
        right_positions.sort();

        let mut positions = Knight::authorized_positions(
            Coord::new(6, 4),
            PieceColor::White,
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
                Some((PieceType::Knight, PieceColor::Black)),
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
                Some((PieceType::Queen, PieceColor::White)),
                None,
                None,
                None,
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

        let mut right_positions: Vec<Coord> = vec![];
        right_positions.sort();

        let mut positions = Knight::authorized_positions(
            Coord::new(1, 4),
            PieceColor::Black,
            board.board,
            &[],
            is_king_checked,
        );
        positions.sort();

        assert_eq!(right_positions, positions);
    }
}
