use {
  crate::{
    mmu::MMU,
    util::*,
  }
};

#[derive(Debug, Clone, Default)]
pub struct CPU {
  pub af: u16,
  pub bc: u16,
  pub de: u16,
  pub hl: u16,
  // ^ general purpose registers
  pub sp: u16,
  pub pc: u16
}

impl CPU {
  const F_REGISTER_Z_FLAG_BIT_N: u8 = 7;
  const F_REGISTER_N_FLAG_BIT_N: u8 = 6;
  const F_REGISTER_H_FLAG_BIT_N: u8 = 5;
  const F_REGISTER_C_FLAG_BIT_N: u8 = 4;

  // TODO: what is this
  const CARRY_BIT: u8 = 0;
  const BORROW_BIT: u8 = 0;

  const HALF_CARRY_BIT: u8 = 8;
  const LOWER_HALF_CARRY_BIT: u8 = 3;
  const UPPER_HALF_CARRY_BIT: u8 = Self::LOWER_HALF_CARRY_BIT + 8;

  const HALF_BORROW_BIT: u8 = 8;
  const UPPER_HALF_BORROW_BIT: u8 = Self::LOWER_HALF_BORROW_BIT + 8;
  const LOWER_HALF_BORROW_BIT: u8 = 4;

  const UPPER_BORROW_BIT: u8 = Self::LOWER_BORROW_BIT + 8;
  const LOWER_BORROW_BIT: u8 = 0;

  pub fn step(&mut self, mmu: &mut MMU) -> u8 {
    let pc = self.pc;
    let opcode = mmu.read(pc);
    self.exec(opcode, mmu)
  }

  fn exec(&mut self, opcode: u8, mmu: &mut MMU) -> u8 {
    let n_cycles = match opcode {

      // NOP
      // 1  4
      // - - - -
      0x00 => {
        self.pc += 1;
        4
      }

      // LD BC,d16
      // 3  12
      // - - - -
      0x01 => {
        self.bc = mmu.read_double(self.pc + 1);
        self.pc += 3;
        12
      }

      // LD (BC),A
      // 1  8
      // - - - -
      0x02 => {
        mmu.write(self.bc, self.a());
        self.pc += 1;
        8
      }

      // INC BC
      // 1  8
      // - - - -
      0x03 => {
        self.bc = self.bc.overflowing_add(1).0;
        self.pc += 1;
        8
      }

      // DEC B
      // 1  4
      // Z 1 H -
      0x05 => {
        self.set_b(self.b().overflowing_sub(1).0);
        self.set_flags(Some(self.b() == 0), Some(true), Some(get_bit(self.bc, Self::HALF_BORROW_BIT)), None);
        self.pc += 1;
        4
      }

      // LD (a16),SP
      // 3  20
      // - - - -
      // Put Stack Pointer (SP) at address n.
      0x08 => {
        mmu.write_double(mmu.read_double(self.pc + 1), self.sp);
        self.pc += 3;
        20
      }

      // LD A,(BC)
      // 1  8
      // - - - -
      0x0A => {
        self.set_a(mmu.read(self.bc));
        self.pc += 1;
        8
      }

      // INC C
      // 1  4
      // Z 0 H -
      0x0C => {
        self.set_c(self.c().overflowing_add(1).0);
        self.set_flags(
          Some(self.c() == 0),
          Some(false),
          Some(get_bit(self.bc, Self::LOWER_HALF_CARRY_BIT)),
          None
        );
        self.pc += 1;
        4
      }

      // LD C,d8
      // 2  8
      // - - - -
      0x0E => {
        self.set_c(mmu.read(self.pc + 1));
        self.pc += 2;
        8
      }

      // LD DE,d16
      // 3  12
      // - - - -
      0x11 => {
        self.de = mmu.read_double(self.pc + 1);
        self.pc += 3;
        12
      }

      // LD (DE),A
      // 1  8
      // - - - -
      0x12 => {
        mmu.write(self.de, self.a());
        self.pc += 1;
        8
      }

      // DEC D
      // 1  4
      // Z 1 H -
      0x15 => {
        self.set_d(self.d().overflowing_sub(1).0);
        self.set_flags(
          Some(self.d() == 0),
          Some(true),
          Some(get_bit(self.de, Self::UPPER_HALF_BORROW_BIT)),
          None
        );
        self.pc += 1;
        4
      }

      // LD D,d8
      // 2  8
      // - - - -
      0x16 => {
        self.set_d(mmu.read(self.pc + 1));
        self.pc += 2;
        8
      }

      // JR r8
      // 2  12
      // - - - -
      0x18 => {
        let offset = mmu.read(self.pc + 1) as i8;
        self.pc = if offset < 0 {
          self.pc.overflowing_sub(-offset as u16).0
        } else {
          self.pc.overflowing_add(offset as u16).0
        };

        self.pc += 2;
        12
      }

      // ADD HL,DE
      // 1  8
      // - 0 H C
      0x19 => {

        self.set_flags(
          None,
          Some(false),
          Some(get_bit(self.hl, Self::HALF_CARRY_BIT)),
          Some(get_bit(self.hl, Self::CARRY_BIT))
        );
        self.pc += 1;
        8
      }

      // INC E
      // 1  4
      // Z 0 H -
      0x1C => {
        self.set_e(self.e() + 1);
        self.set_flags(
          Some(self.e() == 0),
          Some(false),
          Some(get_bit(self.de, Self::LOWER_HALF_CARRY_BIT)),
          None
        );
        self.pc += 1;
        4
      }

      // DEC E
      // 1  4
      // Z 1 H -
      0x1D => {
        self.set_e(self.e().overflowing_sub(1).0);
        self.pc += 1;
        self.set_flags(
          Some(self.e() == 0),
          Some(true),
          Some(get_bit(self.de, Self::LOWER_HALF_BORROW_BIT)),
          None
        );
        4
      }

      // JR NZ,r8
      // 2  12/8
      // - - - -
      0x20 => {
        let offset = mmu.read(self.pc + 1) as i8;
        if !self.get_z_flag() {
          self.pc = if offset < 0 {
            self.pc.overflowing_sub(-offset as u16).0
          } else {
            self.pc.overflowing_add(offset as u16).0
          };
          12
        } else {
          self.pc += 2;
          8
        }
      }

      // LD HL,d16
      // 3  12
      // - - - -
      0x21 => {
        self.hl = mmu.read_double(self.pc + 1);
        self.pc += 3;
        12
      }

      // LD (HL+),A
      // 1  8
      // - - - -
      0x22 => {
        mmu.write(self.hl, self.a());
        self.hl = self.hl.overflowing_add(1).0;
        self.pc += 1;
        8
      }

      // INC HL
      // 1  8
      // - - - -
      0x23 => {
        self.hl += 1;
        self.pc += 1;
        8
      }

      // DEC H
      // 1  4
      // Z 1 H -
      0x25 => {
        self.set_h(self.h().overflowing_sub(1).0);
        self.set_flags(
          Some(self.h() == 0),
          Some(true),
          Some(get_bit(self.hl, Self::UPPER_HALF_BORROW_BIT)),
          None
        );
        self.pc += 1;
        4
      }

      // INC L
      // 1  4
      // Z 0 H -
      0x2c => {
        self.set_l(self.l() + 1);

        self.set_flags(
          Some(self.l() == 0),
          Some(false),
          Some(get_bit(self.l() as u16, Self::LOWER_HALF_CARRY_BIT as u8)),
          None
        );
        self.pc += 1;
        4
      }

      // DEC L
      // 1  4
      // Z 1 H -
      0x2D => {
        self.set_l(self.l().overflowing_sub(1).0);
        self.pc += 1;
        4
      }

      // LD L,d8
      // 2  8
      // - - - -
      0x2E => {
        self.set_l(mmu.read(self.pc + 1));
        self.pc += 2;
        8
      }

      // CPL
      // 1  4
      // - 1 1 -
      0x2F => {
        self.set_a(!self.a());
        self.set_flags(None, Some(true), Some(true), None);
        self.pc += 1;
        4
      }

      // LD SP,d16
      // 3  12
      // - - - -
      0x31 => {
        self.sp = mmu.read_double(self.pc + 1);
        self.pc += 3;
        12
      }

      // LD (HL-),A
      // 1  8
      // - - - -
      0x32 => {
        mmu.write(self.hl, self.a());
        self.hl -= 1;
        self.pc += 1;
        8
      }

      // LD A,d8
      // 2  8
      // - - - -
      0x3E => {
        self.set_a(mmu.read(self.pc + 1));
        self.pc += 2;
        8
      }

      // LD B,A
      // 1  4
      // - - - -
      0x47 => {
        self.set_b(self.a());
        self.pc += 1;
        4
      }

      // LD C,B
      // 1  4
      // - - - -
      0x48 => {
        self.set_c(self.b());
        self.pc += 1;
        4
      }

      // LD C,C
      // 1  4
      // - - - -
      0x49 => {
        self.set_c(self.c());
        self.pc += 1;
        4
      }

      // LD C,D
      // 1  4
      // - - - -
      0x4A => {
        self.set_c(self.d());
        self.pc += 1;
        4
      }

      // LD C,E
      // 1  4
      // - - - -
      0x4b => {
        self.set_c(self.e());
        self.pc += 1;
        4
      }

      // CALL NZ,a16
      // 3  24/12
      // - - - -
      0x4C => {
        if self.get_z_flag() {
          let address = mmu.read_double(self.pc + 1);
          self.pc = address;
          self.pc += 3;
          24
        } else {
          self.pc += 3;
          12
        }
      }

      // LD C,L
      // 1  4
      // - - - -
      0x4D => {
        self.set_c(self.l());
        self.pc += 1;
        4
      }

      // LD C,(HL)
      // 1  8
      // - - - -
      0x4E => {
        self.set_c(mmu.read(self.hl));
        self.pc += 1;
        8
      }

      // LD C,A
      // 1  4
      // - - - -
      0x4F => {
        self.set_c(self.a());
        self.pc += 1;
        4
      }

      // LD D,B
      // 1  4
      // - - - -
      0x50 => {
        self.set_d(self.b());
        self.pc += 1;
        4
      }

      // LD D,C
      // 1  4
      // - - - -
      0x51 => {
        self.set_d(self.c());
        self.pc += 1;
        4
      }

      // LD D,D
      // 1  4
      // - - - -
      0x52 => {
        self.set_d(self.d());
        self.pc += 1;
        4
      }

      // LD D,E
      // 1  4
      // - - - -
      0x53 => {
        self.set_d(self.e());
        self.pc += 1;
        4
      }

      // LD D,H
      // 1  4
      // - - - -
      0x54 => {
        self.set_d(self.h());
        self.pc += 1;
        4
      }

      // LD D,L
      // 1  4
      // - - - -
      0x55 => {
        self.set_d(self.l());
        self.pc += 1;
        4
      }

      // LD D,(HL)
      // 1  8
      // - - - -
      0x56 => {
        self.set_d(mmu.read(self.hl));
        self.pc += 1;
        8
      }

      // LD D,A
      // 1  4
      // - - - -
      0x57 => {
        self.set_d(self.a());
        self.pc += 1;
        4
      }

      // LD E,B
      // 1  4
      // - - - -
      0x58 => {
        self.set_e(self.b());
        self.pc += 1;
        4
      }

      // LD E,C
      // 1  4
      // - - - -
      0x59 => {
        self.set_e(self.c());
        self.pc += 1;
        4
      }

      // LD E,D
      // 1  4
      // - - - -
      0x5A => {
        self.set_e(self.d());
        self.pc += 1;
        4
      }

      // LD E,E
      // 1  4
      // - - - -
      0x5B => {
        self.set_e(self.e());
        self.pc += 1;
        4
      }

      // LD E,H
      // 1  4
      // - - - -
      0x5C => {
        self.set_e(self.h());
        self.pc += 1;
        4
      }

      // LD H,B
      // 1  4
      // - - - -
      0x60 => {
        self.set_h(self.b());
        self.pc += 1;
        4
      }

      // LD H,(HL)
      // 1  8
      // - - - -
      0x66 => {
        self.set_h(mmu.read(self.hl));
        self.pc += 1;
        8
      }

      // LD L,E
      // 1  4
      // - - - -
      0x6B => {
        self.set_l(self.e());
        self.pc += 1;
        4
      }

      // LD L,H
      // 1  4
      // - - - -
      0x6c => {
        self.set_l(self.h());
        self.pc += 1;
        4
      }

      // LD L,(HL)
      // 1  8
      // - - - -
      0x6E => {
        self.set_l(mmu.read(self.hl));
        self.pc += 1;
        8
      }

      // LD L,L
      // 1  4
      // - - - -
      0x6D => {
        self.set_l(self.l());
        self.pc += 1;
        4
      }

      // LD L,A
      // 1  4
      // - - - -
      0x6F => {
        self.set_l(self.a());
        self.pc += 1;
        4
      }

      // LD (HL),C
      // 1  8
      // - - - -
      0x71 => {
        mmu.write(self.hl, self.c());
        self.pc += 1;
        8
      }

      // LD (HL),D
      // 1  8
      // - - - -
      0x72 => {
        mmu.write(self.hl, self.d());
        self.pc += 1;
        8
      }

      //LD (HL),E
      // 1  8
      // - - - -
      0x73 => {
        mmu.write(self.hl, self.e());
        self.pc += 1;
        8
      }

      // LD (HL),H
      // 1  8
      // - - - -
      0x74 => {
        mmu.write(self.hl, self.h());
        self.pc += 1;
        8
      }

      // LD (HL),L
      // 1  8
      // - - - -
      0x75 => {
        mmu.write(self.hl, self.l());
        self.pc += 1;
        8
      }

      // LD (HL),A
      // 1  8
      // - - - -
      0x77 => {
        mmu.write(self.hl, self.a());
        self.pc += 1;
        8
      }

      // LD A,B
      // 1  4
      // - - - -
      0x78 => {
        self.set_a(self.b());
        self.pc += 1;
        4
      }

      // LD A,C
      // 1  4
      // - - - -
      0x79 => {
        self.set_a(self.c());
        self.pc += 1;
        4
      }

      // LD A,D
      // 1  4
      // - - - -
      0x7A => {
        self.set_a(self.d());
        self.pc += 1;
        4
      }

      // LD A,(HL)
      // 1  8
      // - - - -
      0x7E => {
        self.set_a(mmu.read(self.hl));
        self.pc += 1;
        8
      }

      // LD A,A
      // 1  4
      // - - - -
      0x7F => {
        self.set_a(self.a());
        self.pc += 1;
        4
      }

      // ADD A,(HL)
      // 1  8
      // Z 0 H C
      0x86 => {
        self.set_a(self.a().overflowing_add(mmu.read(self.hl)).0);
        self.set_flags(
          Some(self.a() == 0),
          Some(false),
          Some(get_bit(self.af, Self::HALF_CARRY_BIT)),
          Some(get_bit(self.af, Self::CARRY_BIT))
        );
        self.pc += 1;
        8
      }

      // SUB B
      // 1  4
      // Z 1 H C
      0x90 => {
        self.set_a(self.a().overflowing_sub(self.b()).0);
        // FIXME: verify the correctness of this
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.bc, Self::UPPER_HALF_BORROW_BIT)),
          Some(get_bit(self.bc, Self::UPPER_BORROW_BIT))
        );
        self.pc += 1;
        4
      }

      // SUB C
      // 1  4
      // Z 1 H C
      0x91 => {
        self.set_a(self.a().overflowing_sub(self.c()).0);
        // FIXME: verify the correctness of this
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.bc, Self::LOWER_HALF_BORROW_BIT)),
          Some(get_bit(self.bc, Self::LOWER_BORROW_BIT))
        );
        self.pc += 1;
        4
      }

      // SUB D
      // 1  4
      // Z 1 H C
      0x92 => {
        self.set_a(self.a().overflowing_sub(self.d()).0);
        // FIXME: verify the correctness of this
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.de, Self::UPPER_HALF_BORROW_BIT)),
          Some(get_bit(self.de, Self::UPPER_BORROW_BIT))
        );
        self.pc += 1;
        4
      }

      // SUB E
      // 1  4
      // Z 1 H C
      0x93 => {
        self.set_a(self.a().overflowing_sub(self.e()).0);
        // FIXME: verify the correctness of this
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.de, Self::LOWER_HALF_BORROW_BIT)),
          Some(get_bit(self.de, Self::LOWER_BORROW_BIT))
        );
        self.pc += 1;
        4
      }

      // SUB H
      // 1  4
      // Z 1 H C
      0x94 => {
        self.set_a(self.a().overflowing_sub(self.h()).0);
        // FIXME: verify the correctness of this
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.hl, Self::UPPER_HALF_BORROW_BIT)),
          Some(get_bit(self.hl, Self::UPPER_BORROW_BIT))
        );
        self.pc += 1;
        4
      }

      // SUB L
      // 1  4
      // Z 1 H C
      0x95 => {
        self.set_a(self.a().overflowing_sub(self.l()).0);
        // FIXME: verify the correctness of this
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.hl, Self::LOWER_HALF_BORROW_BIT)),
          Some(get_bit(self.hl, Self::LOWER_BORROW_BIT))
        );
        self.pc += 1;
        4
      }

      // SUB (HL)
      // 1  8
      // Z 1 H C
      0x96 => {
        self.set_a(self.a().overflowing_sub(self.l()).0);
        // FIXME: verify the correctness of this
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.af, Self::LOWER_HALF_BORROW_BIT)),
          Some(get_bit(self.af, Self::LOWER_BORROW_BIT))
        );
        self.pc += 1;
        8
      }

      // SUB A
      // 1  4
      // Z 1 H C
      0x97 => {
        self.set_a(self.a().overflowing_sub(self.a()).0);
        // FIXME: verify the correctness of this
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.af, Self::LOWER_HALF_BORROW_BIT)),
          Some(get_bit(self.af, Self::LOWER_BORROW_BIT))
        );
        self.pc += 1;
        4
      }

      // SBC A,B
      // 1  4
      // Z 1 H C
      0x98 => {
        let (rhs, _) = self.b().overflowing_add(if self.c_flag() { 1 } else { 0 });
        self.set_a(self.a().overflowing_sub(rhs).0);
        // FIXME: verify the correctness of this
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.af, Self::LOWER_HALF_BORROW_BIT)),
          Some(get_bit(self.af, Self::LOWER_BORROW_BIT))
        );
        self.pc += 1;
        4
      }

      // SBC A,C
      // 1  4
      // Z 1 H C
      0x99 => {
        let (rhs, _) = self.c().overflowing_add(if self.c_flag() { 1 } else { 0 });
        self.set_a(self.a().overflowing_sub(rhs).0);
        // FIXME: verify the correctness of this
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.af, Self::LOWER_HALF_BORROW_BIT)),
          Some(get_bit(self.af, Self::LOWER_BORROW_BIT))
        );
        self.pc += 1;
        4
      }

      // SBC A,D
      // 1  4
      // Z 1 H C
      0x9A => {
        let (rhs, _) = self.d().overflowing_add(if self.c_flag() { 1 } else { 0 });
        self.set_a(self.a().overflowing_sub(rhs).0);
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.af, Self::UPPER_HALF_BORROW_BIT)),
          Some(get_bit(self.af, Self::BORROW_BIT))
        );
        self.pc += 1;
        4
      }

      // SBC A,E
      // 1  4
      // Z 1 H C
      0x9B => {
        let (rhs, _) = self.e().overflowing_add(if self.c_flag() { 1 } else { 0 });
        self.set_a(self.a().overflowing_sub(rhs).0);
        self.pc += 1;
        4
      }

      // SBC A,H
      // 1  4
      // Z 1 H C
      0x9C => {
        let (rhs, _) = self.h().overflowing_add(if self.c_flag() { 1 } else { 0 });
        self.set_a(self.a().overflowing_sub(rhs).0);
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.af, Self::UPPER_HALF_BORROW_BIT)),
          Some(get_bit(self.af, Self::BORROW_BIT))
        );
        self.pc += 1;
        4
      }

      // SBC A,L
      // 1  4
      // Z 1 H C
      0x9D => {
        let (rhs, _) = self.l().overflowing_add(if self.c_flag() { 1 } else { 0 });
        self.set_a(self.a().overflowing_sub(rhs).0);
        self.set_flags(
          Some(self.a() == 0),
          Some(true),
          Some(get_bit(self.af, Self::UPPER_HALF_CARRY_BIT)),
          Some(get_bit(self.af, Self::CARRY_BIT))
        );
        self.pc += 1;
        4
      }

      // XOR C
      // 1  4
      // Z 0 0 0
      0xA9 => {
        self.set_c(self.a() ^ self.c());
        self.set_flags(Some(self.a() == 0), Some(false), Some(false), Some(false));
        self.pc += 1;
        4
      }

      // XOR D
      // 1  4
      // Z 0 0 0
      0xAA => {
        self.set_a(self.a() ^ self.d());
        self.set_flags(Some(self.a() == 0), Some(false), Some(false), Some(false));
        self.pc += 1;
        4
      }

      // XOR E
      // 1  4
      // Z 0 0 0
      0xAB => {
        self.set_a(self.a() ^ self.e());
        self.set_flags(Some(self.a() == 0), Some(false), Some(false), Some(false));
        self.pc += 1;
        4
      }

      // XOR H
      // 1  4
      // Z 0 0 0
      0xAC => {
        self.set_a(self.a() ^ self.h());
        self.set_flags(Some(self.a() == 0), Some(false), Some(false), Some(false));
        self.pc += 1;
        4
      }

      // XOR A
      // 1  4
      // Z 0 0 0
      0xAF => {
        self.set_a(self.a() ^ self.a());
        self.set_flags(Some(self.a() == 0), Some(false), Some(false), Some(false));
        self.pc += 1;
        4
      }

      // JP a16
      // 3  16
      // - - - -
      0xC3 => {
        self.pc = mmu.read_double(self.pc + 1);
        16
      }

      0xCB => match mmu.read(self.pc + 1) {
        // BIT 7,H
        // 2  8
        // Z 0 1 -
        0x7C => {
          self.set_flags(Some(get_bit(self.bc, 7 + 8)), Some(false), Some(true), None);
          self.pc += 2;
          8
        }
        b => unimplemented!("0xCB prefixed command not implemented 0x{:x}", b)
      }

      // CALL Z,a16
      // 3  24/12
      // - - - -
      0xCC => {
        if self.get_z_flag() {
          mmu.write_double(self.sp, self.pc);
          self.sp -= 2;
          self.pc = mmu.read_double(self.pc + 1);
          24
        } else {
          self.pc += 3;
          12
        }
      }

      // ADC A,d8
      // 2  8
      // Z 0 H C
      0xCE => {
        // let immediate = mmu.read(pc + 1) as i8;
        // unsafe {
        //   if immediate > 0 {
        //     self.af.bytes.0 += immediate as u8
        //   } else {
        //     self.af.bytes.0 -= -immediate as u8
        //   }
        // }
        // self.pc += 2;
        // 8
        unimplemented!()
      },

      // LDH (a8),A
      // 2  12
      // - - - -
      0xE0 => {
        mmu.write(MMU::IO_START_ADDR + mmu.read(self.pc + 1) as u16, self.a());
        self.pc += 2;
        12
      }

      // LD (C),A
      // 2  8
      // - - - -
      0xE2 => {
        mmu.write(self.c() as u16, self.a());
        self.pc += 2;
        8
      }

      // PUSH HL
      // 1  16
      // - - - -
      0xE5 => {
        mmu.write_double(self.sp, self.hl);
        self.sp -= 2;
        self.pc += 1;
        16
      }

      // RST 38H
      // 1  16
      // - - - -
      0xFF => {
        mmu.write_double(self.sp, self.pc);
        self.sp -= 2;
        self.pc = 0x38;
        16
      }
      b => unimplemented!("command not implemented 0x{:x}", b)
    };

    n_cycles
  }

  fn set_f_bit_n(&mut self, n: u8, value: bool) {
    self.af = set_bit(self.af, n, value);
  }

  fn set_z_flag(&mut self, value: bool) {
    self.set_f_bit_n(Self::F_REGISTER_Z_FLAG_BIT_N, value)
  }

  fn set_n_flag(&mut self, value: bool) {
    self.set_f_bit_n(Self::F_REGISTER_N_FLAG_BIT_N, value)
  }

  fn set_h_flag(&mut self, value: bool) {
    self.set_f_bit_n(Self::F_REGISTER_H_FLAG_BIT_N, value)
  }

  fn set_c_flag(&mut self, value: bool) {
    self.set_f_bit_n(Self::F_REGISTER_C_FLAG_BIT_N, value)
  }

  /// This flag is set when the result of a math operation is zero or two values
  /// match when using the CP instruction.
  fn get_z_flag(&self) -> bool {
    self.get_f_bit_n(Self::F_REGISTER_Z_FLAG_BIT_N)
  }

  // This flag is set if a subtraction was performed in the last math instruction.
  fn get_n_flag(&self) -> bool {
    self.get_f_bit_n(Self::F_REGISTER_N_FLAG_BIT_N)
  }

  fn set_flags(&mut self, z: Option<bool>, n: Option<bool>, h: Option<bool>, c: Option<bool>) {
    match z {
      Some(z) => self.set_z_flag(z),
      _ => {}
    }
    match n {
      Some(n) => self.set_n_flag(n),
      _ => {}
    }
    match h {
      Some(h) => self.set_h_flag(h),
      _ => {}
    }
    match c {
      Some(c) => self.set_c_flag(c),
      _ => {}
    }
  }

  /// Return true if the `n`th bit of the `f` register is set
  fn get_f_bit_n(&self, n: u8) -> bool {
    let (_, f) = unpack_bytes_from_double(self.af);
    f & (1 << n) != 0
  }

  fn n_flag(&self) -> bool {
    self.get_f_bit_n(Self::F_REGISTER_N_FLAG_BIT_N)
  }

  fn h_flag(&self) -> bool {
    self.get_f_bit_n(Self::F_REGISTER_H_FLAG_BIT_N)
  }

  fn c_flag(&self) -> bool {
    self.get_f_bit_n(Self::F_REGISTER_C_FLAG_BIT_N)
  }


  fn c(&self) -> u8 {
    unpack_bytes_from_double(self.bc).1
  }

  fn d(&self) -> u8 {
    unpack_bytes_from_double(self.de).0
  }

  fn set_c(&mut self, value: u8) {
    self.bc = set_lower(self.bc, value);
  }

  fn a(&self) -> u8 {
    unpack_bytes_from_double(self.af).0
  }

  fn set_a(&mut self, value: u8) {
    self.af = set_upper(self.af, value);
  }

  fn b(&self) -> u8 {
    let (b, _) = unpack_bytes_from_double(self.bc);
    b
  }

  fn set_b(&mut self, value: u8) {
    self.bc = set_upper(self.bc, value);
  }

  fn h(&self) -> u8 {
    unpack_bytes_from_double(self.hl).0
  }

  fn set_h(&mut self, value: u8) {
    self.hl = set_upper(self.hl, value);
  }

  fn l(&self) -> u8 {
    unpack_bytes_from_double(self.hl).1
  }

  fn set_l(&mut self, value: u8) {
    self.hl = set_lower(self.hl, value);
  }

  fn set_d(&mut self, value: u8) {
    self.de = set_upper(self.de, value);
  }

  fn e(&self) -> u8 {
    unpack_bytes_from_double(self.de).1
  }

  fn set_e(&mut self, value: u8) {
    self.de = set_lower(self.de, value);
  }
}
