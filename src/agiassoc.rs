use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/*
Gaurav Sablok,
gsablok@proton.me
*/

/// Retrieves description information for AGI IDs from an association file.
/// # Arguments
/// * `ids_file` - Path to file containing AGI IDs
/// * `association_file` - Path to gene association file (e.g., gene_association.tair)
/// # Returns
/// Returns a `Result` containing a `Vec` of tuples, where each tuple contains
/// an AGI ID and a corresponding description (e.g., `("AT5G27680", "AT5G27680|RECQSIM|RECQ helicase SIM|T1G16.10|T1G16_10")`).
/// # Example
/// The output format matches:
/// vec![
///     ("AT5G27680".to_string(), "AT5G27680|RECQSIM|RECQ helicase SIM|T1G16.10|T1G16_10".to_string()),
///     ("AT5G27680".to_string(), "AT5G27680|AT5G27680.1|T1G16.10|T1G16_10".to_string()),
///     ...

pub fn get_agi_description_information<P: AsRef<Path>>(
    ids_file: P,
    association_file: P,
) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let ids_file = File::open(&ids_file)?;
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

    let assoc_path = association_file.as_ref();
    let temp_file_path = assoc_path.with_file_name(
        assoc_path
            .file_name()
            .map(|f| format!("{}name", f.to_string_lossy()))
            .unwrap_or_default(),
    );

    let assoc_file = File::open(&association_file)?;
    let assoc_reader = BufReader::new(assoc_file);
    let temp_file = File::create(&temp_file_path)?;
    let mut temp_writer = BufWriter::new(temp_file);

    for line in assoc_reader.lines() {
        let line = line?;
        if !line.starts_with('!') {
            writeln!(temp_writer, "{}", line)?;
        }
    }
    temp_writer.flush()?;

    let temp_file = File::open(&temp_file_path)?;
    let temp_reader = BufReader::new(temp_file);
    let mut description_data: Vec<(String, String)> = Vec::new();

    for line in temp_reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 11 {
            let agi = parts[1].split('.').next().unwrap_or("").to_string();
            let description = parts[10].to_string();
            description_data.push((agi, description));
        }
    }

    let result: Vec<(String, String)> = description_data
        .into_iter()
        .filter(|(agi, _)| final_ids.contains(agi))
        .collect();

    std::fs::remove_file(temp_file_path)?;

    Ok(result)
}
