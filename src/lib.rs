#![feature(exclusive_range_pattern)]
pub mod cpu;
pub mod mmu;
pub mod ppu;
pub mod cartridge;
mod util;

pub use {
    cartridge::Cartridge,
};
use {
    util::*
};


#[derive(Debug, Default)]
pub struct Gameboy {
    pub mmu: mmu::MMU,
    pub cpu: cpu::CPU,
    pub ppu: ppu::PPU,
}

impl Gameboy {
    /// Create a new gameboy with a cartridge loaded
    pub fn new_with_cartridge(cartridge: cartridge::Cartridge) -> Self {
        Gameboy {
            mmu: mmu::MMU {
                cartridge: Some(Box::new(cartridge)),
                ..mmu::MMU::default()
            },
            ..Gameboy::default()
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.mmu.read(address)
    }

    /// Step the gameboy forward one instruction, returning the number of cycles the instruction took to execute
    pub fn step(&mut self) -> u8 {
        let n_cycles = self.cpu.step(&mut self.mmu);
        self.ppu.step(&mut self.mmu, n_cycles);
        n_cycles
    }

    pub fn display(&self) -> impl Iterator<Item=&u8> {
        self.mmu.vram()
    }
}
