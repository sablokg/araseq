use crate::structfile::Coordinate;
use csv::ReaderBuilder;
use std::fs::File;
use std::io::{self, BufReader};

/*
Gaurav Sablok,
gsablok@proton.me
*/

pub fn get_tair_coordinates(
    gff: &str,
    arabidopsis_gene: bool,
    arabidopsis_exon: bool,
    arabidopsis_three_prime_utr: bool,
    arabidopsis_five_prime_utr: bool,
) -> io::Result<Vec<Vec<(String, i32, i32)>>> {
    let file = File::open(gff)?;
    let reader = BufReader::new(file);
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(reader);
    let mut gene_coords = Vec::new();
    let mut exon_coords = Vec::new();
    let mut three_prime_utr_coords = Vec::new();
    let mut five_prime_utr_coords = Vec::new();
    for result in rdr.records() {
        let record = result?;
        if record.len() < 9 {
            continue; // Skip invalid rows
        }

        let gene_type = &record[2];
        let start: i32 = record[3].parse().unwrap_or(0);
        let end: i32 = record[4].parse().unwrap_or(0);
        let attributes = &record[8];
        let id = attributes
            .split(';')
            .find(|s| s.starts_with("ID=") || s.starts_with("Parent="))
            .map(|s| {
                s.replace("ID=", "")
                    .replace("Parent=", "")
                    .split('.')
                    .next()
                    .unwrap_or("")
                    .to_string()
            })
            .unwrap_or_default();
        match gene_type {
            "gene" => gene_coords.push(Coordinate { id, start, end }),
            "exon" => exon_coords.push(Coordinate { id, start, end }),
            "three_prime_UTR" => three_prime_utr_coords.push(Coordinate { id, start, end }),
            "five_prime_UTR" => five_prime_utr_coords.push(Coordinate { id, start, end }),
            _ => {}
        }
    }
    let gene_output: Vec<Vec<(String, i32, i32)>> = gene_coords
        .into_iter()
        .map(|coord| vec![(coord.id, coord.start, coord.end)])
        .collect();
    let exon_output: Vec<Vec<(String, i32, i32)>> = exon_coords
        .into_iter()
        .map(|coord| vec![(coord.id, coord.start, coord.end)])
        .collect();
    let three_prime_utr_output: Vec<Vec<(String, i32, i32)>> = three_prime_utr_coords
        .into_iter()
        .map(|coord| vec![(coord.id, coord.start, coord.end)])
        .collect();
    let five_prime_utr_output: Vec<Vec<(String, i32, i32)>> = five_prime_utr_coords
        .into_iter()
        .map(|coord| vec![(coord.id, coord.start, coord.end)])
        .collect();
    if arabidopsis_gene {
        Ok(gene_output)
    } else if arabidopsis_exon {
        Ok(exon_output)
    } else if arabidopsis_three_prime_utr {
        Ok(three_prime_utr_output)
    } else if arabidopsis_five_prime_utr {
        Ok(five_prime_utr_output)
    } else {
        Ok(Vec::new())
    }
}
