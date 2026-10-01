use regex::Regex;
use std::collections::HashSet;
use std::error::Error;
use std::fs;

/*
Gaurav Sablok,
gsablok@proton.me
*/

pub fn plantoml_fetcher(plantomlfile: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut final_plant_ontology = Vec::new();
    let file = plantomlfile;
    let contents = fs::read_to_string(file).expect("Unable to read file");
    let plantoml_read: Vec<&str> = contents.lines().collect();
    let re = Regex::new(r"PO_\d{1,10}").unwrap();
    let mut extract_plant_ontology = HashSet::new();
    for line in plantoml_read {
        if line.contains("PO") {
            for cap in re.captures_iter(line) {
                let match_str = cap.get(0).unwrap().as_str();
                if match_str.len() > 3 {
                    extract_plant_ontology.insert(match_str.to_string());
                }
            }
        }
    }
    final_plant_ontology.extend(extract_plant_ontology);
    Ok(final_plant_ontology)
}
