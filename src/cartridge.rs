use {
  crate::util::*,
};

#[derive(Clone)]
pub enum Cartridge {
  RomOnly(Box<[u8; Self::ROM_ONLY_SIZE]>),
  MBC1 {},
  MBC2 {},
  MBC3 {},
  MBC5 {},
  Rumble {},
  HuC1 {},
}

impl Cartridge {
  const ROM_ONLY_SIZE: usize = 0xFFFF;

  pub fn maybe_from_bytes(bytes: &[u8]) -> Option<Self> {
    if bytes.len() <= Self::ROM_ONLY_SIZE {
      let mut buffer = [0; Self::ROM_ONLY_SIZE];
      let buffer_len = bytes.len();
      buffer[..buffer_len].clone_from_slice(&bytes[..buffer_len]);
      Some(Cartridge::RomOnly(Box::new(buffer)))
    } else {
      None
    }
  }
}

impl Memory for Cartridge {
  fn read(&self, address: u16) -> u8 {
    match self {
      Self::RomOnly(inner) => inner[address as usize],
      _ => unimplemented!()
    }
  }

  fn write(&mut self, _address: u16, _value: u8) {
    match self {
      Self::RomOnly(_) => { /* noop */ },
      _ => unimplemented!()
    }
  }
}
