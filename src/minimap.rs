use crate::pieces::*;
use crate::Taken;
use bevy::ecs::{entity::Entity, query::Without, system::Query};

#[derive(Default)]
pub struct MiniMap([[PieceType; 11]; 11]);

impl std::fmt::Debug for MiniMap {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        // Draw the minimap, for debugging!

        // Nice line on top
        write!(fmt, "\n   {}\n", "_".repeat(23))?;

        // Draw a char for each piece
        // Iter in reverse because Bevy draws y=0 on the bottom
        for y_rev in 0..11 {
            let y = 10 - y_rev;

            write!(fmt, "{: >2} ", y)?;

            for line in self.0.iter() {
                // write!(fmt, "|")?;

                // for piece in line {
                let c = match line[y] {
                    PieceType::None => " ",
                    PieceType::King => "K",
                    PieceType::Defender => "D",
                    PieceType::Attacker => "A",
                };

                // Piece seperator
                write!(fmt, "|{}", c)?;
            }
            // Line seperator
            write!(fmt, "| {}\n", y)?;
        }

        // Nice line on bottom
        writeln!(fmt, "   {}", "\u{AF}".repeat(23))?;

        // Draw numbers on the bottom
        write!(fmt, "   ")?;

        for x in 0..11 {
            write!(fmt, " {}", x)?;
        }

        Ok(())
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

    fn get_neighbours(&self, loc: (i8, i8)) -> [(i8, i8); 4] {
        let (x, y) = loc;
        [(x, y - 1), (x, y + 1), (x - 1, y), (x + 1, y)]
    }

    pub fn detect_killings(&self, last_dest: (i8, i8)) -> Vec<(i8, i8)> {
        // Check all the easy pieces

        let mut retval = vec![];

        for neighbours in self.get_neighbours(last_dest) {
            let loc = self.get_piece(neighbours);
            let [up_loc, down_loc, left_loc, right_loc] = self.get_neighbours(neighbours);

            let up = self.get_piece(up_loc);
            let down = self.get_piece(down_loc);
            let left = self.get_piece(left_loc);
            let right = self.get_piece(right_loc);

            if (up.is_enemy(loc) && down.is_enemy(loc))
                || (left.is_enemy(loc) && right.is_enemy(loc))
            {
                retval.push(neighbours);
            }
        }

        retval
    }

    pub fn from_query(query: &Query<(Entity, &Piece), Without<Taken>>) -> Self {
        let mut map: MiniMap = Default::default();
        for (_entity, piece) in query.iter() {
            map.set_piece(piece);
        }
        map
    }
}

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use bevy::app::App;
    impl MiniMap {
        pub fn from_app(app: &mut App) -> Self {
            let mut map: MiniMap = Default::default();
            for (_entity, piece) in app.world.query::<(Entity, &Piece)>().iter(&app.world) {
                map.set_piece(piece);
            }

            map
        }
    }
}
