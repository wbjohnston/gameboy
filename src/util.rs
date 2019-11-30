
/// extract the upper and lower bytes from a double
///
/// # Returns
/// the upper and lower byte as `(upper, lower)`
pub fn unpack_bytes_from_double(double: u16) -> (u8, u8) {
  let lower = double as u8;
  let upper = (double >> 8) as u8;
  (upper, lower)
}

pub fn pack_bytes_into_double(upper: u8, lower: u8) -> u16 {
  ((upper as u16) << 8) | lower as u16
}

pub fn set_lower(target: u16, value: u8) -> u16 {
  (target & 0xFF00) | value as u16
}

pub fn set_upper(target: u16, value: u8) -> u16 {
  (target & 0x00FF) | (value as u16) << 8
}

pub fn set_bit(target: u16, n: u8, value: bool) -> u16 {
  target | (1 << n)
}

pub fn get_bit(target: u16, n: u8) -> bool {
  (target & (1 << n)) != 0
}

pub trait Memory {
  fn read(&self, address: u16) -> u8;

  fn read_double(&self, address: u16) -> u16 {
    pack_bytes_into_double(self.read(address), self.read(address + 1))
  }

  fn write(&mut self, address: u16, value: u8);

  fn write_double(&mut self, address: u16, value: u16) {
    let (upper, lower) = unpack_bytes_from_double(value);
    self.write(address, upper);
    self.write(address + 1, lower);
  }
}


#[cfg(test)]
mod test {
  use super::*;
  use quickcheck_macros::quickcheck;

  #[test]
  fn set_lower_works() {
    let val = 0xDEAD;
    let sampled = set_lower(val, 0xED);
    assert_eq!(val, 0xDEED);
  }

  #[test]
  fn set_upper_works() {
    let val = 0xDEAD;
    let sampled = set_upper(val, 0xED);
    assert_eq!(val, 0xEDAD);
  }

  #[test]
  fn unpack_returns_bytes_in_correct_order() {
    let val = 0xDEAD;
    let (upper, lower) = unpack_bytes_from_double(val);
    assert_eq!(upper, 0xDE);
    assert_eq!(lower, 0xAD);
  }


  #[quickcheck]
  fn pack_and_unpack_is_equal(double: u16) -> bool {
    let (upper, lower) = unpack_bytes_from_double(double);
    double == pack_bytes_into_double(upper, lower)
  }
}
