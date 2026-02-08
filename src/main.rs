use anyhow::{Result, bail};
use std::fs::File;
use std::io::{SeekFrom, prelude::*};

#[allow(dead_code)]
struct Page {
    page_type: u8,
    cell_count: u16,
    cell_content_area: u16,
    cells: Vec<Cell>,
}

#[allow(dead_code)]
struct Cell{
    payload_size: u32,
    row_id: u32,
    payload: Vec<u8>,
}


impl Page {
    fn from_bytes(bytes: &[u8]) -> Self {
        let page_type = bytes[0];
        let cell_count = u16::from_be_bytes([bytes[3], bytes[4]]);
        let cell_content_area = u16::from_be_bytes([bytes[5], bytes[6]]);

        let mut cells = Vec::new();

        for cell in 0..cell_count {

            // 8 is used to skip the page header, and each byte after header is cell pointer to actual payload.
            let cell_offset = 8 + (cell as usize) * 2; // Each cell pointer is 2 bytes
            let cell_location = u16::from_be_bytes([bytes[cell_offset], bytes[cell_offset + 1]]) as usize;
            let cell_bytes = &bytes[cell_location..];
            let cell = Cell::from_bytes(cell_bytes);
            println!("{cell_offset:?}@{cell_location:?}");
            cells.push(cell);
        }

        Page {
            page_type,
            cell_count,
            cell_content_area,
            cells,
        }
    }
}



impl Cell {
    fn from_bytes(bytes: &[u8]) -> Self {
        let payload_size = u32::from_be_bytes([bytes[0]]);
        let row_id = u32::from_be_bytes([bytes[1]]);
        // let payload = bytes[8..(8 + payload_size as usize)].to_vec();
        let payload = Vec::new();

        println!("payload size: {bytes:?}@{payload_size}");

        Cell {
            payload_size,
            row_id,
            payload,
        }
    }
}

fn get_file_header(file: &mut File) -> Result<[u8; 100]> {
    let mut header = [0u8; 100];

    let _current_pos = file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut header)?;

    Ok(header)
}

fn get_page_bytes(file: &mut File, offset: usize, page_size: usize, contains_database_header: bool) -> Result<Vec<u8>> {
    
    let mut adjusted_page_size = page_size;
    if contains_database_header {
        // If the page contains the database header, we need to adjust the offset to account for the header
        adjusted_page_size = adjusted_page_size - 100; // Skip the 100-byte header
    }


    let mut page_bytes = vec![0u8; adjusted_page_size];
    let _current_pos = file.seek(SeekFrom::Start(offset as u64))?;
    println!("current position in file: {}", _current_pos);
    file.read_exact(&mut page_bytes)?;

    Ok(page_bytes)
}



fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            let mut file = File::open(&args[1])?;
            let header_bytes = get_file_header(&mut file)?;

            // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
            #[allow(unused_variables)]
            let page_size = u16::from_be_bytes([header_bytes[16], header_bytes[17]]);

            
            let page_bytes: Vec<u8>= get_page_bytes(&mut file, 100, page_size as usize, true)?;

            let page = Page::from_bytes(&page_bytes);

            println!("details: {:?}", &page_bytes[..=100]);

            // You can use print statements as follows for debugging, they'll be visible when running tests.
            eprintln!("Logs from your program will appear here!");

            println!("database page size: {}", page_size);
            println!("number of tables: {}", page.cell_count);
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
