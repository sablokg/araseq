use csv::ReaderBuilder;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

/*
Gaurav Sablok,
gsablok@proton.me
*/

pub fn read_tair_ncbi(
    input: &str,
    output: &str,
    arg_id: &str,
) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let input_file = File::open(input)?;
    let mut output_file = File::create(output)?;
    let reader = BufReader::new(input_file);

    for line in reader.lines() {
        let line = line?;
        if !line.starts_with('!') {
            writeln!(output_file, "{}", line)?;
        }
    }

    let file = File::open(output)?;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(file);

    let mut gene_ids = Vec::new();
    let mut correspondence = Vec::new();

    for result in rdr.records() {
        let record = result?;
        if record.len() > 12 {
            let gene_id = record[1].replace("gene:", "").replace("locus:", "");
            let refs: Vec<String> = record[12].split('|').map(|s| s.to_string()).collect();
            gene_ids.push(gene_id);
            correspondence.push(refs);
        }
    }

    let pubmed: Vec<(String, Vec<String>)> = gene_ids.into_iter().zip(correspondence).collect();

    let format_id: HashSet<String> = pubmed
        .into_iter()
        .filter(|(gene, _)| gene == arg_id)
        .flat_map(|(_, refs)| {
            refs.into_iter()
                .filter(|r| r.contains("PMID"))
                .flat_map(|r| r.split_whitespace().next().map(|x| x.to_string()))
                .filter(|s| s.contains("PMID"))
                .map(|s| s.replace("PMID:", ""))
        })
        .collect();

    let format_check: Vec<String> = format_id
        .into_iter()
        .map(|id| format!("https://pubmed.ncbi.nlm.nih.gov/{}/", id))
        .collect();

    let mut ncbi_derive_information = HashMap::new();
    let selector = Selector::parse("div.abstract-content.selected").unwrap();

    for url in format_check.iter() {
        let response = get(url).expect("string not found");
        let document = Html::parse_document(&response.text().expect("text not found"));

        let abstract_text = document
            .select(&selector)
            .map(|element| {
                element
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string()
            })
            .collect::<Vec<String>>()
            .join("");

        ncbi_derive_information.insert(url.clone(), abstract_text);
    }

    Ok(ncbi_derive_information
        .into_iter()
        .filter(|(k, v)| !k.is_empty() && !v.is_empty())
        .collect())
}
