use crate::pieces::*;

#[derive(Default)]
pub struct MiniMap([[PieceType; 11]; 11]);

impl std::fmt::Debug for MiniMap {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        // Draw the minimap, for debugging!

        // Nice line on top
        write!(fmt, "\n{}\n", "_".repeat(23))?;

        // Draw a char for each piece
        // Iter in reverse because Bevy draws y=0 on the bottom
        for line in self.0.iter().rev() {
            write!(fmt, "|")?;

            for piece in line {
                let c = match piece {
                    PieceType::None => " ",
                    PieceType::King => "K",
                    PieceType::Defender => "D",
                    PieceType::Attacker => "A",
                };

                // Seperator
                write!(fmt, "{}|", c)?;
            }
            write!(fmt, "\n")?;
        }
        // Nice line on bottom
        write!(fmt, "{}\n", "\u{AF}".repeat(23))
    }
}

impl MiniMap {
    pub fn set_piece(&mut self, piece: &Piece) {
        if piece.is_king {}
        self.0[piece.x as usize][piece.y as usize] = piece.to_piecetype();
    }

    fn get_piece(&self, loc: (i8, i8)) -> PieceType {
        if loc.0 < 0 || loc.0 > 10 || loc.1 < 0 || loc.1 > 10 {
            PieceType::None
        } else {
            self.0[loc.0 as usize][loc.1 as usize]
        }
    }

    pub fn detect_killings(&self) -> Vec<(i8, i8)> {
        // Check all the easy pieces

        let mut retval = vec![];

        for x in 0..11 {
            for y in 0..11 {
                let loc = self.get_piece((x, y));

                let up = self.get_piece((x, y - 1));
                let down = self.get_piece((x, y + 1));

                let left = self.get_piece((x - 1, y));
                let right = self.get_piece((x + 1, y));

                if (up.is_enemy(loc) && down.is_enemy(loc))
                    || (left.is_enemy(loc) && right.is_enemy(loc))
                {
                    retval.push((x, y));
                }
            }
        }

        retval
    }
}
