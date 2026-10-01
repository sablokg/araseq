use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/*
Gaurav Sablok,
gsablok@proton.me
*/

/// Converts AGI to UniProt IDs using provided mapping and ID files.
/// # Arguments
/// * `agi_uniprot_file` - Path to Arabidopsis2Uniprot conversion file
/// * `ids_file` - Path to file containing AGI IDs to search for
/// # Returns
/// Returns a `Result` containing a `Vec` of tuples, where each tuple contains
/// an AGI ID and a `Vec` of corresponding UniProt IDs
/// # Example
/// The conversion file can be downloaded from:
/// https://www.arabidopsis.org/download_files/Proteins/Id_conversions/AGI2uniprot-Jul2023.txt

pub fn agi_uniprot<P: AsRef<Path>>(
    agi_uniprot_file: P,
    ids_file: P,
) -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>> {
    let ids_file = File::open(ids_file)?;
    let reader = BufReader::new(ids_file);
    let agi_ids: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    let mut ag_uniprot: HashMap<String, Vec<String>> = HashMap::new();
    let uni_file = File::open(agi_uniprot_file)?;
    let uni_reader = BufReader::new(uni_file);

    for line in uni_reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.trim().split('\t').collect();
        if parts.len() > 1 {
            let key = parts[0].to_string();
            let values = parts[1..].iter().map(|s| s.to_string()).collect();
            ag_uniprot.insert(key, values);
        }
    }
    let result: Vec<(String, Vec<String>)> = ag_uniprot
        .into_iter()
        .filter(|(k, _)| agi_ids.contains(k))
        .collect();

    Ok(result)
}
