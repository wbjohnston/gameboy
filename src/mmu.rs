use {
  crate::{cartridge::Cartridge, util::Memory},
  derivative::Derivative,
};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct MMU {
  #[derivative(Debug = "ignore")]
  pub cartridge: Option<Box<Cartridge>>,
  #[derivative(Debug = "ignore")]
  pub bios: [u8; Self::BIOS_SIZE],
  #[derivative(Debug = "ignore")]
  pub vram: [u8; Self::VRAM_SIZE], // video ram
  #[derivative(Debug = "ignore")]
  pub oam: [u8; Self::OAM_SIZE], // sprite attrib memory
  #[derivative(Debug = "ignore")]
  pub iom: [u8; Self::IO_SIZE], // IO memory
  #[derivative(Debug = "ignore")]
  pub ram: [u8; Self::RAM_SIZE], // internal ram
  #[derivative(Debug = "ignore")]
  pub sram: [u8; Self::SRAM_SIZE], // switchable ram
  #[derivative(Debug = "ignore")]
  pub hram: [u8; Self::HRAM_SIZE],
  /// Interrupt enable register
  pub ie: u8,
}

impl Default for MMU {
  fn default() -> Self {
    Self {
      cartridge: None,
      bios: [0; Self::BIOS_SIZE],
      vram: [0; Self::VRAM_SIZE], // video ram
      oam: [0; Self::OAM_SIZE],   // sprite attrib memory
      iom: [0; Self::IO_SIZE],    // IO memory
      ram: [0; Self::RAM_SIZE],   // internal ram
      sram: [0; Self::SRAM_SIZE], // switchable ram
      hram: [0; Self::HRAM_SIZE],
      ie: 0, // interrupt enable register
    }
  }
}

impl MMU {
  // 0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
  //    0000-00FF bios
  pub const BIOS_START_ADDRESS: u16 = 0x0000;
  pub const BIOS_END_ADDRESS: u16 = 0x00FF;
  pub const BIOS_SIZE: usize = (Self::BIOS_END_ADDRESS - Self::BIOS_START_ADDRESS + 1) as usize;

  // 4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
  pub const CARTRIDGE_START_ADDRESS: u16 = 0x0000;
  pub const CARTRIDGE_END_ADDRESS: u16 = 0x7FFF;

  // 8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
  pub const VRAM_START_ADDRESS: u16 = 0x8000;
  pub const VRAM_END_ADDRESS: u16 = 0x9FFF;
  pub const VRAM_SIZE: usize = (Self::VRAM_END_ADDRESS - Self::VRAM_START_ADDRESS + 1) as usize;

  // A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
  pub const EXTRAM_START_ADDRESS: u16 = 0xA000;
  pub const EXTRAM_END_ADDRESS: u16 = 0xBFFF;

  // C000-CFFF   4KB Work RAM Bank 0 (WRAM)
  pub const RAM_START_ADDRESS: u16 = 0xC000;
  pub const RAM_END_ADDRESS: u16 = 0xCFFF;
  pub const RAM_SIZE: usize = (Self::RAM_END_ADDRESS - Self::RAM_START_ADDRESS + 1) as usize;

  // D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
  pub const SRAM_START_ADDRESS: u16 = 0xD000;
  pub const SRAM_END_ADDRESS: u16 = 0xDFFF;
  pub const SRAM_SIZE: usize = (Self::SRAM_END_ADDRESS - Self::SRAM_START_ADDRESS + 1) as usize;

  // E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
  pub const ERAM_START_ADDRESS: u16 = 0xE000;
  pub const ERAM_END_ADDRESS: u16 = 0xFDFF;

  // FE00-FE9F   Sprite Attribute Table (OAM)
  pub const OAM_START_ADDRESS: u16 = 0xFE00;
  pub const OAM_END_ADDRESS: u16 = 0xFE9F;
  pub const OAM_SIZE: usize = (Self::OAM_END_ADDRESS - Self::OAM_START_ADDRESS + 1) as usize;

  // FEA0-FEFF   Not Usable
  pub const UNUSABLE_START_ADDRESS: u16 = 0xFEA0;
  pub const UNUSABLE_END_ADDRESS: u16 = 0xFEFF;
  pub const UNUSABLE_READ_VALUE: u8 = 0xFF;

  // FF00-FF7F   I/O Ports
  pub const IO_START_ADDRESS: u16 = 0xFF00;
  pub const BIOS_DISABLE_REGISTER_ADDRESS: u16 = 0xFF50;
  pub const IO_END_ADDRESS: u16 = 0xFF7F;
  pub const IO_SIZE: usize = (Self::IO_END_ADDRESS - Self::IO_START_ADDRESS + 1) as usize;

  // FF80-FFFE   High RAM (HRAM)
  pub const HRAM_START_ADDRESS: u16 = 0xFF80;
  pub const HRAM_END_ADDRESS: u16 = 0xFFFE;
  pub const HRAM_SIZE: usize = (Self::HRAM_END_ADDRESS - Self::HRAM_START_ADDRESS + 1) as usize;

  pub const INTERRUPT_ENABLE_REG_ADDRESS: u16 = 0xFFFF;
}

impl MMU {
  pub fn vram(&self) -> impl Iterator<Item = &u8> {
    self.vram.iter()
  }

  fn bios_enabled(&self) -> bool {
    self.read(Self::BIOS_DISABLE_REGISTER_ADDRESS) == 0
  }
}

impl Memory for MMU {
  fn read(&self, address: u16) -> u8 {
    match address {
      // 0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
      //    0000-00FF bios
      Self::BIOS_START_ADDRESS..=Self::BIOS_END_ADDRESS if self.bios_enabled() => {
        self.bios[(address - Self::BIOS_START_ADDRESS) as usize]
      }
      // 4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
      Self::CARTRIDGE_START_ADDRESS..=Self::CARTRIDGE_END_ADDRESS => {
        self.cartridge.as_ref().map(|x| x.read(address)).unwrap()
      }
      // 8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
      Self::VRAM_START_ADDRESS..=Self::VRAM_END_ADDRESS => {
        self.vram[(address - Self::VRAM_START_ADDRESS) as usize]
      }
      // A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
      Self::EXTRAM_START_ADDRESS..=Self::EXTRAM_END_ADDRESS => self
        .cartridge
        .as_ref()
        .map(|x| x.read_ram(address - Self::EXTRAM_START_ADDRESS))
        .unwrap(),
      // C000-CFFF   4KB Work RAM Bank 0 (WRAM)
      Self::RAM_START_ADDRESS..=Self::RAM_END_ADDRESS => {
        self.ram[(address - Self::RAM_START_ADDRESS) as usize]
      }
      // D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
      Self::SRAM_START_ADDRESS..=Self::SRAM_END_ADDRESS => {
        self.sram[(address - Self::SRAM_START_ADDRESS) as usize]
      }
      // E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
      Self::ERAM_START_ADDRESS..=Self::ERAM_END_ADDRESS => {
        self.ram[(address - Self::ERAM_START_ADDRESS) as usize]
      }
      // FE00-FE9F   Sprite Attribute Table (OAM)
      Self::OAM_START_ADDRESS..=Self::OAM_END_ADDRESS => {
        self.oam[(address - Self::OAM_START_ADDRESS) as usize]
      }
      // FEA0-FEFF   Not Usable
      Self::UNUSABLE_START_ADDRESS..=Self::UNUSABLE_END_ADDRESS => Self::UNUSABLE_READ_VALUE,
      // FF00-FF7F   I/O Ports
      Self::IO_START_ADDRESS..=Self::IO_END_ADDRESS => {
        self.iom[(address - Self::IO_START_ADDRESS) as usize]
      }
      // FF80-FFFE   High RAM (HRAM)
      Self::HRAM_START_ADDRESS..=Self::HRAM_END_ADDRESS => {
        self.hram[(address - Self::HRAM_START_ADDRESS) as usize]
      }
      // FFFF        Interrupt Enable Register
      Self::INTERRUPT_ENABLE_REG_ADDRESS => self.ie,
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      // 0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
      //    0000-00FF bios
      Self::BIOS_START_ADDRESS..=Self::BIOS_END_ADDRESS if self.bios_enabled() => {}
      // 4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
      Self::CARTRIDGE_START_ADDRESS..=Self::CARTRIDGE_END_ADDRESS => {}
      // 8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
      Self::VRAM_START_ADDRESS..=Self::VRAM_END_ADDRESS => {
        self.vram[(address - Self::VRAM_START_ADDRESS) as usize] = value;
      }
      // A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
      Self::EXTRAM_START_ADDRESS..=Self::EXTRAM_END_ADDRESS => unimplemented!(),
      // C000-CFFF   4KB Work RAM Bank 0 (WRAM)
      Self::RAM_START_ADDRESS..=Self::RAM_END_ADDRESS => {
        self.ram[(address - Self::RAM_START_ADDRESS) as usize] = value;
      }
      // D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
      Self::SRAM_START_ADDRESS..=Self::SRAM_END_ADDRESS => {
        self.sram[(address - Self::SRAM_START_ADDRESS) as usize] = value;
      }
      // E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
      Self::ERAM_START_ADDRESS..=Self::ERAM_END_ADDRESS => {
        self.ram[(address - Self::ERAM_START_ADDRESS) as usize] = value;
      }
      // FE00-FE9F   Sprite Attribute Table (OAM)
      Self::OAM_START_ADDRESS..=Self::OAM_END_ADDRESS => {
        self.oam[(address - Self::OAM_START_ADDRESS) as usize] = value;
      }
      // FEA0-FEFF   Not Usable
      Self::UNUSABLE_START_ADDRESS..=Self::UNUSABLE_END_ADDRESS => {}
      // FF00-FF7F   I/O Ports
      Self::IO_START_ADDRESS..=Self::IO_END_ADDRESS => {
        self.iom[(address - Self::IO_START_ADDRESS) as usize] = value;
      }
      // FF80-FFFE   High RAM (HRAM)
      Self::HRAM_START_ADDRESS..=Self::HRAM_END_ADDRESS => {
        self.hram[(address - Self::HRAM_START_ADDRESS) as usize] = value;
      }
      // FFFF        Interrupt Enable Register
      Self::INTERRUPT_ENABLE_REG_ADDRESS => self.ie = value,
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  #[test]
  fn bios_disabling_works() {
    let cartridge_value = 0x01;
    let bios_value = 0x02;
    let cartridge = Cartridge::maybe_from_bytes(&[cartridge_value; 0x1000]).unwrap();
    let mut mmu = MMU {
      cartridge: Some(Box::new(cartridge)),
      bios: [bios_value; MMU::BIOS_SIZE],
      ..MMU::default()
    };

    // by default the mmu should read from the bios
    assert_eq!(mmu.read(MMU::BIOS_START_ADDRESS), bios_value);
    assert_eq!(mmu.read(MMU::BIOS_END_ADDRESS), bios_value);
    assert_eq!(mmu.read(MMU::BIOS_END_ADDRESS + 1), cartridge_value);

    // but after we disable the bios
    assert_eq!(mmu.bios_enabled(), true);
    assert_eq!(mmu.read(MMU::BIOS_DISABLE_REGISTER_ADDRESS), 0x00);
    mmu.write(MMU::BIOS_DISABLE_REGISTER_ADDRESS, 0x01);
    assert_eq!(mmu.read(MMU::BIOS_DISABLE_REGISTER_ADDRESS), 0x01);
    assert_eq!(mmu.bios_enabled(), false);

    // we should be reading from the cartridge
    assert_eq!(mmu.read(MMU::BIOS_START_ADDRESS), cartridge_value);
    assert_eq!(mmu.read(MMU::BIOS_END_ADDRESS), cartridge_value);
    assert_eq!(mmu.read(MMU::BIOS_END_ADDRESS + 1), cartridge_value);
  }
}
