use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/*
Gaurav Sablok,
gsablok@proton.me
*/

// Function to convert uniprot_agi
// Takes a uniprot_agi file and a file of IDs and returns the converted IDs
// or the links between the AGI and the uniprot
// Arguments:
//     agi_file: Path to uniprot_agi file (download from:
//         https://www.arabidopsis.org/download_files/Proteins/Id_conversions/Uniprot2AGI-JAN2023.txt)
//     ids_file: Path to file containing list of IDs to search for associations
// Returns:
//     Vec of tuples containing (uniprot_id, Vec of AGI IDs)

pub fn uniprot_agi<P: AsRef<Path>>(
    agi_file: P,
    ids_file: P,
) -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>> {
    let mut uniprot_ag: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut final_ids: Vec<String> = Vec::new();
    let ids_file = File::open(ids_file)?;
    let ids_reader = BufReader::new(ids_file);

    for line in ids_reader.lines() {
        let id = line?.trim().to_string();
        if !id.is_empty() {
            final_ids.push(id);
        }
    }
    let agi_file = File::open(agi_file)?;
    let agi_reader = BufReader::new(agi_file);
    for line in agi_reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.trim().split('\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let uniprot_id = parts[0].to_string();
        let agi_list: Vec<String> = parts[1..]
            .iter()
            .flat_map(|s| s.split(';').map(|s| s.to_string()))
            .collect();

        uniprot_ag.insert(uniprot_id, agi_list);
    }

    let result: Vec<(String, Vec<String>)> = uniprot_ag
        .into_iter()
        .filter_map(|(k, v)| {
            if v.iter().any(|agi| final_ids.contains(agi)) {
                Some((k, v))
            } else {
                None
            }
        })
        .collect();

    Ok(result)
}
