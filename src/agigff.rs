use crate::structfile::GffRecord;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/*
Gaurav Sablok,
gsablok@proton.me
*/

/// Retrieves coordinates for AGI IDs from a GFF file based on the specified gene type.
/// # Arguments
/// * `ids_file` - Path to file containing AGI IDs
/// * `gff_file` - Path to GFF3 file (e.g., TAIR10_GFF3_genes.gff)
/// * `gene_type` - Optional gene type filter ("gene", "exon", "three_prime_UTR", "five_prime_UTR")
/// # Returns
/// Returns a `Result` containing a `Vec` of `Vec`s, where each inner `Vec` contains:
/// - For "gene": [[AGI_ID], [start, end]]
/// - For others: [AGI_ID, start, end]
/// # Example
/// The GFF file can be downloaded from:
/// https://www.arabidopsis.org/download_files/Genes/TAIR10_genome_release/TAIR10_gff3/TAIR10_GFF3_genes.gff

pub fn agi_coordinates<P: AsRef<Path>>(
    ids_file: P,
    gff_file: P,
    gene_type: Option<&str>,
) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
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
    let gff_file = File::open(gff_file)?;
    let gff_reader = BufReader::new(gff_file);
    let mut records: Vec<GffRecord> = Vec::new();

    for line in gff_reader.lines() {
        let line = line?;
        if line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 9 {
            continue;
        }

        let gene_type = parts[2].to_string();
        let start: i32 = parts[3].parse().unwrap_or(0);
        let end: i32 = parts[4].parse().unwrap_or(0);
        let gene_id = parts[8]
            .split(';')
            .next()
            .unwrap_or("")
            .split('.')
            .next()
            .unwrap_or("")
            .replace("Parent=", "")
            .replace("ID=", "");

        records.push(GffRecord {
            gene_type,
            start,
            end,
            gene_id,
        });
    }
    let selected_type = match gene_type {
        Some("gene") => Some("gene"),
        Some("exon") => Some("exon"),
        Some("three_prime_UTR") => Some("three_prime_UTR"),
        Some("five_prime_UTR") => Some("five_prime_UTR"),
        _ => None,
    };

    let filtered_records: Vec<GffRecord> = if let Some(gt) = selected_type {
        records
            .into_iter()
            .filter(|r| r.gene_type == gt && final_ids.contains(&r.gene_id))
            .collect()
    } else {
        return Ok(Vec::new());
    };

    let result: Vec<Vec<String>> = match selected_type {
        Some("gene") => filtered_records
            .into_iter()
            .map(|r| {
                vec![
                    r.gene_id.to_string(),
                    r.start.to_string(),
                    r.end.to_string(),
                ]
            })
            .collect::<Vec<Vec<String>>>(),
        Some(_) => filtered_records
            .into_iter()
            .map(|r| vec![r.gene_id, r.start.to_string(), r.end.to_string()])
            .collect(),
        None => Vec::new(),
    };

    Ok(result)
}
