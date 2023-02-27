use std::io::prelude::*;
use std::io::{Cursor, Write};
use std::collections::HashMap;
use zip::write::FileOptions;
use zip::read::ZipArchive;
use reed_solomon_erasure::ReedSolomon;
use zip::{result::ZipError, CompressionMethod, ZipWriter};

pub fn zip_and_expand_data(data: &[u8], num_duplications: usize) -> Result<Vec<u8>, String> {
  // Step 1: Compress the data using the zip format
  let mut cursor = Cursor::new(Vec::new());
  let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
  let mut zip = ZipWriter::new(&mut cursor);
  zip.start_file("data", options).map_err(|e| e.to_string())?;
  zip.write_all(data).map_err(|e| e.to_string())?;
  zip.finish().map_err(|e| e.to_string())?;

  // Step 2: Compute error correction codes for the zipped data
  let zip_data = cursor.into_inner();
  let parity_size = (zip_data.len() * num_duplications) / (num_duplications + 1);
  let data_size = zip_data.len() - parity_size;
  let mut rs = ReedSolomon::new(data_size, parity_size).unwrap();

  // Step 3: Put the zipped data and the error codes into an array
  let mut data_with_parity: Vec<&mut [u8]> = vec![&mut [0; 1 + data_size + parity_size]];
  data_with_parity[0][1..data_size+1].copy_from_slice(&zip_data);
  rs.encode(&mut data_with_parity[0]).unwrap();

  // Step 4: Copy the array num_duplications times
  for _ in 0..num_duplications {
      let index = data_with_parity.len() - 1;
      data_with_parity.push(&mut [0; 1 + data_size + parity_size]);
      data_with_parity[index+1].copy_from_slice(data_with_parity[index]);
      rs.encode(&mut data_with_parity[index+1]).unwrap();
  }

  Ok(data_with_parity.concat())
}




pub fn unzip_and_unexpand_data(data: &[u8], num_duplications: usize) -> Result<Vec<u8>, String> {
  /*
  // Step 1: Determine the original data by finding the most common value at each index.
  let data_size = data.len() / num_duplications;
  let mut original_data = Vec::with_capacity(data_size);
  for i in 0..data_size {
      let mut counts = HashMap::new();
      for j in 0..num_duplications {
          let index = j * data_size + i;
          let value = data[index];
          *counts.entry(value).or_insert(0) += 1;
      }
      let most_common_value = counts.iter().max_by_key(|&(_, count)| count).map(|(&value, _)| value);
      if let Some(value) = most_common_value {
          original_data.push(value);
      } else {
          return Err(format!("Unable to determine original data at index {}", i));
      }
  }

  // Step 2: Split the original data into zipped data and parity data.
  let parity_size = original_data.len() - data_size;
  let zipped_data = &original_data[0..data_size];
  let parity_data = &original_data[data_size..];

  // Step 3: Use the parity data to fix any errors in the zipped data.
  let mut rs = ReedSolomon::new(data_size, parity_size).map_err(|e| format!("Error creating Reed-Solomon encoder: {}", e))?;
  let mut data_with_parity = [&mut zipped_data.to_vec(), &mut parity_data.to_vec()].concat();
  rs.decode(&mut data_with_parity).map_err(|e| format!("Error decoding data: {}", e))?;

  // Step 4: Unzip the corrected data.
  let mut zip = ZipArchive::new(Cursor::new(&data_with_parity)).map_err(|e| format!("Error creating zip archive: {}", e))?;
  let mut file = zip.by_index(0).map_err(|e| format!("Error reading zip file: {}", e))?;
  let mut data = Vec::new();
  file.read_to_end(&mut data).map_err(|e| format!("Error reading zip data: {}", e))?;
  */
  Ok(data)
}


fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).expect("Missing mode argument");
    let file_path = args.get(2).expect("Missing file path argument");
    let output_path = args.get(3).unwrap_or(&format!("{}.out", file_path));
    let n = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
    let num_duplications = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(5);

    match mode.as_str() {
        "encode" => {
            // Read data from file
            let mut data = Vec::new();
            std::fs::File::open(&file_path)?.read_to_end(&mut data)?;

            // Zip and expand data n times
            let mut arf_data = zip_and_expand_data(&data, num_duplications)?;
            for _ in 1..n {
                arf_data = zip_and_expand_data(&arf_data, num_duplications)?;
            }

            // Save array to disk with .arf extension
            let mut arf_file = std::fs::File::create(output_path)?;
            arf_file.write_all(&arf_data)?;
        }
        "decode" => {
            // Read data from arf file
            let mut arf_data = Vec::new();
            std::fs::File::open(&file_path)?.read_to_end(&mut arf_data)?;

            // Unzip and unexpand data n times
            let mut data = unzip_and_unexpand_data(&arf_data, num_duplications).map_err(|e| format!("Failed to decode arf file: {}", e))?;
            for _ in 1..n {
                data = unzip_and_unexpand_data(&data, num_duplications).map_err(|e| format!("Failed to decode arf file: {}", e))?;
            }

            // Save data to file
            let mut output_file = std::fs::File::create(output_path)?;
            output_file.write_all(&data)?;
        }
        _ => {
            println!("Invalid mode: {}", mode);
        }
    }

    Ok(())
}
