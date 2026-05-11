pub mod constants;
pub mod knights;
pub mod pawn;
pub mod king;
pub mod sliders;
pub mod magics;
pub mod masks;
pub mod rooks;
pub mod bishops;
pub mod queen;
pub mod moves;
pub mod generator;
pub mod notation;

pub fn init_all() {
    king::get_king_attacks(types::square::Square::new(0));
    knights::get_knight_attacks(types::square::Square::new(0));
    pawn::get_pawn_attacks(types::color::Color::White, types::square::Square::new(0));
    
    rooks::init_rook_magic_table();
    bishops::init_bishop_magic_table();
}
