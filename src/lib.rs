#![feature(exclusive_range_pattern)]
pub mod cpu;
pub mod mmu;
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
}

impl Gameboy {
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

    pub fn step(&mut self) -> u8 {
        self.cpu.step(&mut self.mmu)
    }
}
