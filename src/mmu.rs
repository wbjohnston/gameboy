use {
  crate::{
    cartridge::Cartridge,
    util::Memory,
  }
};

pub struct MMU {
  pub cartridge: Option<Box<Cartridge>>,
  pub vram: [u8; Self::VRAM_SIZE],            // video ram
  pub oam: [u8; Self::OAM_SIZE],              // sprite attrib memory
  pub iom: [u8; Self::IO_SIZE],               // IO memory
  pub ram: [u8; Self::RAM_0_SIZE],                           // internal ram
  pub sram: [u8; Self::SRAM_SIZE],            // switchable ram
  pub ier: u8,                                // interrupt enable register
}

impl Default for MMU {
  fn default() -> Self {
    Self {
      cartridge: None,
      vram: [0; Self::VRAM_SIZE],            // video ram
      oam: [0; Self::OAM_SIZE],              // sprite attrib memory
      iom: [0; Self::IO_SIZE],               // IO memory
      ram: [0; Self::RAM_0_SIZE],                           // internal ram
      sram: [0; Self::SRAM_SIZE],            // switchable ram
      ier: 0,                                // interrupt enable register
    }
  }
}

impl std::fmt::Debug for MMU {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.cartridge.is_some() {
      write!(f, "MMU(..TODO..)")
    } else {
      write!(f, "MMU[..TODO..]")
    }
  }
}


impl MMU {
  //  interrupt Enable Register     --------------- FFFF
  pub const INTERRUPT_ENABLE_REG_ADDR: u16          = 0xFFFF;
  //  Internal RAM                  --------------- FF80 - FFFF
  pub const RAM_1_END_ADDR: u16                          = Self::INTERRUPT_ENABLE_REG_ADDR;
  pub const RAM_1_START_ADDR: u16                   = Self::EMPTY_1_END_ADDR;
  pub const RAM_1_SIZE: usize                       = (Self::RAM_1_END_ADDR - Self::RAM_1_START_ADDR + 1) as usize;
  // Empty but unusable for I/O     --------------- FF4C - FF80
  pub const EMPTY_1_END_ADDR: u16                   = 0xFF80;
  pub const EMPTY_1_START_ADDR: u16                 = Self::IO_END_ADDR;
  // I/O ports                     ----------------FF00 - FF4C
  pub const IO_END_ADDR: u16                        = 0xFF4C;
  pub const IO_START_ADDR: u16                      = Self::EMPTY_0_END_ADDR;
  pub const IO_SIZE: usize                          = (Self::IO_END_ADDR - Self::IO_START_ADDR + 1) as usize;
  //  Empty but unusable for I/O   ---------------- FEA0 - FF00
  pub const EMPTY_0_END_ADDR: u16                   = 0xFF00;
  pub const EMPTY_0_START_ADDR: u16                 = Self::OAM_END_ADDR;
  // Sprite Attrib Memory (OAM)    ----------------FE00 - FEA0
  pub const OAM_END_ADDR: u16                       = 0xFEA0;
  pub const OAM_START_ADDR: u16                     = Self::ERAM_END_ADDR;
  pub const OAM_SIZE: usize                         = (Self::OAM_END_ADDR - Self::OAM_START_ADDR + 1) as usize;
  // Echo of 8kB Internal RAM      ----------------E000 - FE00
  pub const ERAM_END_ADDR: u16                      = 0xFE00;
  pub const ERAM_START_ADDR: u16                    = Self::RAM_0_END_ADDR;
  // 8kB Internal RAM              ----------------C000 - E000
  pub const RAM_0_END_ADDR: u16                     = 0xE000;
  pub const RAM_0_START_ADDR: u16                   = Self::SRAM_END_ADDR;
  pub const RAM_0_SIZE: usize                       = (Self::RAM_0_END_ADDR - Self::RAM_0_START_ADDR + 1) as usize;
  // 8kB switchable RAM bank       ----------------A000 - C000
  pub const SRAM_END_ADDR: u16                      = 0xC000;
  pub const SRAM_START_ADDR: u16                    = Self::VRAM_END_ADDR;
  pub const SRAM_SIZE: usize                        = (Self::SRAM_END_ADDR - Self::SRAM_START_ADDR + 1) as usize;
  // 8kB Video RAM                 ----------------8000 - A000
  pub const VRAM_END_ADDR: u16                      = 0xA000;
  pub const VRAM_START_ADDR: u16                    = Self::CARTRIDGE_END_ADDR;
  pub const VRAM_SIZE: usize                        = (Self::VRAM_END_ADDR - Self::VRAM_START_ADDR + 1) as usize;
  // cartridge                     ----------------0000 - 8000
  pub const CARTRIDGE_END_ADDR: u16                 = 0x8000;
  pub const CARTRIDGE_START_ADDR: u16               = 0x0000;
  // unusable io block

}

impl MMU {
  pub fn vram(&self) -> impl Iterator<Item=&u8> {
    self.vram.iter()
  }
}

impl Memory for MMU {
  fn read(&self, address: u16) -> u8 {
    match address {
      Self::INTERRUPT_ENABLE_REG_ADDR                       => self.ier,
      Self::RAM_1_START_ADDR..Self::RAM_1_END_ADDR          => unimplemented!(),
      Self::EMPTY_1_START_ADDR..Self::EMPTY_1_END_ADDR      => 0,
      Self::IO_START_ADDR..Self::IO_END_ADDR                => self.iom[(address - Self::IO_START_ADDR) as usize],
      Self::EMPTY_0_START_ADDR..Self::EMPTY_0_END_ADDR      => 0,
      Self::OAM_START_ADDR..Self::OAM_END_ADDR              => self.oam[(address - Self::OAM_START_ADDR) as usize],
      Self::ERAM_START_ADDR..Self::ERAM_END_ADDR            => self.ram[(address - Self::ERAM_START_ADDR) as usize],
      Self::RAM_0_START_ADDR..Self::RAM_0_END_ADDR          => self.ram[(address - Self::RAM_0_START_ADDR) as usize],
      Self::SRAM_START_ADDR..Self::SRAM_END_ADDR            => self.sram[(address - Self::SRAM_START_ADDR) as usize],
      Self::VRAM_START_ADDR..Self::VRAM_END_ADDR            => self.vram[(address - Self::VRAM_START_ADDR) as usize],
      Self::CARTRIDGE_START_ADDR..Self::CARTRIDGE_END_ADDR  => self.cartridge
        .as_ref()
        .map(|x| x.read(address))
        .unwrap_or(0)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      _ => {}
    }
  }
}
