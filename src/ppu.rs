
use {
  crate::mmu::MMU,
};


/// A pixel processing unit
#[derive(Debug, Clone, Default)]
pub struct PPU {

}


impl PPU {
  pub fn step(&mut self, mmu: &mut MMU, n_cycles: u8) {

  }
}
