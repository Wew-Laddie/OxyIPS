use std::path::Path;
use std::convert::TryInto;

const VERSION: &'static str = "1.0.0";

fn main() {
	let args: Vec<String> = std::env::args().collect();
	let patch: &Path;
	let rom: &Path;
	let output: &Path;

	if args.len() < 4 {
		println!("OxyIPS {} by Wew-Laddie", VERSION);
		println!("Usage: oxyips [patch] [rom] [output]");
		return;
	}

	patch = Path::new(&args[1]);
	rom = Path::new(&args[2]);
	output = Path::new(&args[3]);

	if !patch.exists() {
		println!("Error: IPS patch file '{}' does not exist!", patch.display());
		return;
	}
	if !rom.exists() {
		println!("Error: ROM file '{}' does not exist!", rom.display());
		return;
	}

	let patch_buffer: Vec<u8> = match std::fs::read(patch)
	{
		Ok(v) => {
			let signature = match String::from_utf8(v[0..5].to_vec()) {
				Ok(s) => s,
				Err(_e) => {
					eprintln!("Error: IPS patch file '{}' is invalid!", patch.display());
					return;		
				}			
			};
			if signature != "PATCH" {
				eprintln!("Error: IPS patch file '{}' is invalid!", patch.display());
				return;
			}
			v
		},
		Err(_e) => {
			eprintln!("Error: Failed to read IPS patch file '{}'!", patch.display());
			return;
		}
	};

	let mut rom_buffer: Vec<u8> = match std::fs::read(rom)
	{
		Ok(v) => v,
		Err(_e) => {
			eprintln!("Error: Failed to read ROM file '{}'!", rom.display());
			return;
		}
	};

	let mut record_n = 0;
	let mut patch_ptr: usize = 5;
	let mut offset: usize;
	let mut size: usize;
	let mut rle_byte: u8 = 0;
	let mut rle: bool;

	loop {
		let offset_slice: &[u8] = &patch_buffer[patch_ptr .. patch_ptr+3];
		match String::from_utf8(offset_slice.to_vec()) {
			Ok(s) => { if s == "EOF" { break; } },
			Err(_e) => ()
		};
		offset = (((offset_slice[0] as u32) << 16) | ((offset_slice[1] as u32) << 8) | (offset_slice[2] as u32)).try_into().unwrap();
		
		let size_slice: &[u8] = &patch_buffer[patch_ptr+3 .. patch_ptr+5];
		size = (((size_slice[0] as u16) << 8) | (size_slice[1] as u16)).try_into().unwrap();
		if size == 0 {
			let rle_slice: &[u8] = &patch_buffer[patch_ptr+5 .. patch_ptr+8];
			size = (((rle_slice[0] as u16) << 8) | (rle_slice[1] as u16)).try_into().unwrap();
			rle_byte = rle_slice[2];
			rle = true;
			patch_ptr += 8;
		} else {
			rle = false;
			patch_ptr += 5;
		}

		let rle_slice: [u8; 1] = [rle_byte];
		if offset >= rom_buffer.len().try_into().unwrap() {
			if rle {
				rom_buffer.extend_from_slice(&rle_slice.repeat(size));
			} else {
				rom_buffer.extend_from_slice(&patch_buffer[patch_ptr .. patch_ptr + size]);
			}
		} else {
			if rle {
				rom_buffer[offset .. offset + size].copy_from_slice(&rle_slice.repeat(size));
			} else {
				rom_buffer[offset .. offset + size].copy_from_slice(&patch_buffer[patch_ptr .. patch_ptr + size]);
			}
		}

		record_n += 1;
		if !rle {
			patch_ptr += size;
		}
	}

	match std::fs::write(output, rom_buffer) {
		Ok(_) => (),
		Err(_e) => {
			eprintln!("Error: Failed to write output file '{}'!", output.display());
			return;		
		}
	};
	
	println!("Wrote {} records to {}.", record_n, output.display());
}
