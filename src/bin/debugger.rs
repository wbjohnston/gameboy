use {
  std::{
    io::{
      self,
      BufRead,
      prelude::*,
    },
    fs::{File},
    env::{args},
  },
  gameboy::{
    Gameboy,
    Cartridge
  },
  failure::{
    Fail,
    Error
  }
};

#[derive(Debug, Fail)]
enum AppError {
  #[fail(display = "failed to parse cartridge")]
  FailedToParseCartridge,
  #[fail(display = "not enough arguments")]
  NotEnoughArguments,
}


fn main() -> Result<(), Error> {
  let args: Vec<_> = args().collect();

  if args.len() < 3 {
    println!("usage: {} <bios> <rom>", args[0]);
    return Err(AppError::NotEnoughArguments.into())
  }

  let bios = {
    let mut buffer = vec![];
    let mut bios = [0; gameboy::mmu::MMU::BIOS_SIZE];
    let mut file = File::open(&args[1])?;
    file.read_to_end(&mut buffer)?;
    bios.copy_from_slice(&mut buffer[0..gameboy::mmu::MMU::BIOS_SIZE]);
    bios
  };

  let cartridge = {
    let mut buffer = vec![];
    let mut file = File::open(&args[2])?;
    file.read_to_end(&mut buffer)?;
    match Cartridge::maybe_from_bytes(buffer.as_ref()) {
      Some(cartridge) => cartridge,
      _ => return Err(AppError::FailedToParseCartridge.into())
    }
  };

  let stdin = io::stdin();
  let stdin_lock = stdin.lock();
  let mut reader = io::BufReader::new(stdin_lock);
  let mut gameboy = Gameboy::new(bios);
  let mut buffer = String::new();

  loop {
    print!(">");
    io::stdout().flush()?;
    buffer.clear();
    reader.read_line(&mut buffer)?;
    let commands: Vec<_> = buffer.trim().split(" ").collect();
    if execute_command(commands.as_ref(), &mut gameboy)? {
      break;
    }
  }
  Ok(())
}


fn execute_command(commands: &[&str], gameboy: &mut Gameboy) -> Result<bool, Error> {
  match commands[0] {
    "s" => match & commands[1..]{
      [] => {
        gameboy.step();
        execute_command(&["p"], gameboy)?;
        execute_command(&["mpc"], gameboy)?;
        Ok(false)
      }
      [n] if n.chars().all(char::is_numeric) => {
        let n = n.parse().unwrap();
        for _ in 0..n {
          gameboy.step();
        }
        execute_command(&["p"], gameboy)?;
        execute_command(&["mpc"], gameboy)?;
        Ok(false)
      }
      _ => {
        Ok(false)
      }
    }
    "d" | "display" => {
      let d: Vec<_> = gameboy.display().collect();
      dbg!(d);
      Ok(false)
    }
    "mpc" => {
      execute_command(&["m", format!("{}", gameboy.cpu.pc).as_str()], gameboy)
    }
    "m" | "mem" => match &commands[1..] {
      [] => {
        Ok(false)
      }
      [address_str] => {
        let address = if address_str.chars().nth(0) == Some('0') && address_str.chars().nth(1) == Some('x') {
          u16::from_str_radix(&address_str[2..], 16)?
        } else {
          address_str.parse::<u16>()?
        };
        println!("0x{:x} = {:x}", address, gameboy.read(address));
        Ok(false)
      }
      [start_address_str, end_address_str] => {
        let start_address = if start_address_str.chars().nth(0) == Some('0') &&
          start_address_str.chars().nth(1) == Some('x')
        {
          u16::from_str_radix(&start_address_str[2..], 16)?
        } else {
          start_address_str.parse::<u16>()?
        };

        let end_address = if end_address_str.chars().nth(0) == Some('0') &&
          end_address_str.chars().nth(1) == Some('x')
        {
          u16::from_str_radix(&end_address_str[2..], 16)?
        } else {
          end_address_str.parse::<u16>()?
        };

        for address in start_address..end_address {
          execute_command(&["m", format!("{}", address).as_str()], gameboy)?;
        }
        Ok(false)
      }
      _ => unimplemented!()
    }
    "p" | "print" => {
      println!("{:#x?}", gameboy);
      Ok(false)
    }
    "e" | "exit" => {
      Ok(true)
    }
    cmd => {
      println!("command not implemented '{}'", cmd);
      Ok(false)
    }
  }
}
