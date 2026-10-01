use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/*
Gaurav Sablok,
gsablok@proton.me
*/

/// Converts TAIR to UniProt IDs using provided mapping and ID files.
/// # Arguments
/// * `tair_uniprot_file` - Path to TAIR2Uniprot mapping file
/// * `ids_file` - Path to file containing IDs to search for
/// # Returns
/// Returns a `Result` containing a `Vec` of tuples, where each tuple contains
/// a UniProt ID and a `Vec` of corresponding locus and AGI IDs
/// # Example
/// The mapping file can be downloaded from:
/// https://www.arabidopsis.org/download_files/Proteins/Id_conversions/TAIR2UniprotMapping-Jul2023.txt

pub fn uniprot_tair<P: AsRef<Path>>(
    tair_uniprot_file: P,
    ids_file: P,
) -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>> {
    let ids_file = File::open(ids_file)?;
    let reader = BufReader::new(ids_file);
    let final_ids: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.to_uppercase()
                .split('.')
                .next()
                .unwrap_or("")
                .to_string()
        })
        .collect();
    let mut tair_uniprot: HashMap<String, Vec<String>> = HashMap::new();
    let uni_file = File::open(tair_uniprot_file)?;
    let uni_reader = BufReader::new(uni_file);

    for line in uni_reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.trim().split('\t').collect();
        if parts.len() > 1 {
            let key = parts[0].to_string();
            let values = parts[1..].iter().map(|s| s.to_string()).collect();
            tair_uniprot.insert(key, values);
        }
    }
    let result: Vec<(String, Vec<String>)> = tair_uniprot
        .into_iter()
        .filter(|(_k, v)| v.get(1).map_or(false, |v1| final_ids.contains(v1)))
        .collect();

    Ok(result)
}
