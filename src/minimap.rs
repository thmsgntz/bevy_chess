use crate::pieces::*;
use crate::Taken;
use bevy::ecs::{entity::Entity, query::Without, system::Query};

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
                    PieceType::Castle => "C",
                    PieceType::Wall => panic!("never format a wall!"),
                };

                // Piece seperator
                write!(fmt, "|{}", c)?;
            }
            // Line seperator
            writeln!(fmt, "| {}", y)?;
        }

        // Nice line on bottom
        writeln!(fmt, "   {}", "\u{AF}".repeat(23))?;

        // Draw numbers on the bottom
        write!(fmt, "   ")?;

        for x in 0..11 {
            write!(fmt, " {}", x)?;
        }

        writeln!(fmt, "\n")
    }
}

impl Default for MiniMap {
    fn default() -> Self {
        let mut retval: MiniMap = MiniMap {
            0: [[PieceType::None; 11]; 11],
        };
        retval.0[5][5] = PieceType::Castle;

        retval
    }
}
impl MiniMap {
    pub fn set_piece(&mut self, piece: &Piece) {
        self.0[piece.x as usize][piece.y as usize] = piece.to_piecetype();
    }

    fn get_piece(&self, loc: (i8, i8)) -> PieceType {
        if loc.0 < 0 || loc.0 > 10 || loc.1 < 0 || loc.1 > 10 {
            PieceType::Wall
        } else {
            self.0[loc.0 as usize][loc.1 as usize]
        }
    }

    fn get_neighbours(&self, loc: (i8, i8)) -> [(i8, i8); 4] {
        let (x, y) = loc;
        [(x, y - 1), (x, y + 1), (x - 1, y), (x + 1, y)]
    }

    pub fn detect_killings(&self, last_dest: (i8, i8)) -> Vec<(i8, i8)> {
        let retval = self.detect_multikill(&last_dest);

        if retval.is_empty() {
            // Check all the easy kills
            self.detect_simple_kill(last_dest)
        } else {
            retval
        }
    }

    fn detect_multikill(&self, last_dest: &(i8, i8)) -> Vec<(i8, i8)> {
        let mut retval: Vec<(i8, i8)> = vec![];

        let just_moved_piece = self.get_piece(*last_dest);

        dbg!(&last_dest);

        // For each neighbour of the just moved piece
        for neighbour_loc in self.get_neighbours(*last_dest) {
            let neighbour = self.get_piece(neighbour_loc);

            if neighbour.is_friendly(just_moved_piece) {
                // Ignore friendly neighbours
                continue;
            }

            println!("Found an enemy neighbour! {:?}", neighbour_loc);

            let mut done_group: Vec<(i8, i8)> = vec![];
            let mut todo_group: Vec<(i8, i8)> = vec![];
            let mut new_group: Vec<(i8, i8)> = vec![];

            let mut found_gap: bool = false;

            todo_group.push(neighbour_loc);

            while !found_gap {
                println!("Starting loop");
                for p in &todo_group {
                    for p_neighbour in self.get_neighbours(*p) {
                        // Check if we already did that neighbour
                        if done_group.contains(&p_neighbour) {
                            continue;
                        }

                        let p_neighbour_piece = self.get_piece(p_neighbour);

                        // If we find a gap, it's not closed yet!
                        if p_neighbour_piece == PieceType::None {
                            println!("Gap found! {:?}", p_neighbour);
                            found_gap = true;
                            break;
                        } else if p_neighbour_piece.is_enemy(just_moved_piece) {
                            new_group.push(p_neighbour);
                        }
                    }
                }

                println!("Found new group: {:?}", new_group);

                done_group.append(&mut todo_group);
                if new_group.is_empty() {
                    break;
                }
                todo_group.append(&mut new_group);
            }

            if !found_gap {
                retval.append(&mut done_group);
            }

            //

            // println!("{:?} is an enemy of {:?}", neighbour_loc, last_dest);
        }

        println!("Found retval: {:?}\n\n", retval);

        // vec![]
        retval
    }

    fn detect_simple_kill(&self, last_dest: (i8, i8)) -> Vec<(i8, i8)> {
        let mut retval = vec![];
        for neighbours in self.get_neighbours(last_dest) {
            let loc = self.get_piece(neighbours);

            let [up_loc, down_loc, left_loc, right_loc] = self.get_neighbours(neighbours);

            if up_loc == last_dest || down_loc == last_dest {
                let up = self.get_piece(up_loc);
                let down = self.get_piece(down_loc);

                if up.is_enemy(loc) && down.is_enemy(loc) {
                    retval.push(neighbours);
                }
            } else if left_loc == last_dest || right_loc == last_dest {
                let left = self.get_piece(left_loc);
                let right = self.get_piece(right_loc);

                if left.is_enemy(loc) && right.is_enemy(loc) {
                    retval.push(neighbours);
                }
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
