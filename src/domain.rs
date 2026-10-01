use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};

/*
Gaurav Sablok,
gsablok@proton.me
*/

pub fn domain_analyzer(
    file: &str,
    arg_type: Option<&str>,
) -> Result<Option<Vec<String>>, io::Error> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let data: Value = serde_json::from_reader(reader)?;
    if let Some(arg_type) = arg_type {
        match arg_type {
            "sequence" => {
                let sequence: String = data["results"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|i| i["sequence"].as_str().unwrap())
                    .collect();
                return Ok(Some(vec![sequence]));
            }
            _ => {
                let matches: Vec<Value> = data["results"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|i| i["matches"].as_array().unwrap())
                    .cloned()
                    .collect();

                let result: Vec<String> = match arg_type {
                    "interpro_normalize" => matches
                        .iter()
                        .map(|m| serde_json::to_string(m).unwrap())
                        .collect(),
                    "signature" => matches
                        .iter()
                        .filter_map(|m| m["signature"].as_str())
                        .map(String::from)
                        .collect(),
                    "location" => matches
                        .iter()
                        .filter_map(|m| m["locations"].as_str())
                        .map(String::from)
                        .collect(),
                    "evalue" => matches
                        .iter()
                        .filter_map(|m| m["evalue"].as_str())
                        .map(String::from)
                        .collect(),
                    "score" => matches
                        .iter()
                        .filter_map(|m| m["score"].as_str())
                        .map(String::from)
                        .collect(),
                    "modelac" => matches
                        .iter()
                        .filter_map(|m| m["model-ac"].as_str())
                        .map(String::from)
                        .collect(),
                    "scope" => matches
                        .iter()
                        .filter_map(|m| m["scope"].as_str())
                        .map(String::from)
                        .collect(),
                    "accession" => matches
                        .iter()
                        .filter_map(|m| m["accession"].as_str())
                        .map(String::from)
                        .collect(),
                    "name" => matches
                        .iter()
                        .filter_map(|m| m["name"].as_str())
                        .map(String::from)
                        .collect(),
                    "proteinClass" => matches
                        .iter()
                        .filter_map(|m| m["ProteinClass"].as_str())
                        .map(String::from)
                        .collect(),
                    "graftPoint" => matches
                        .iter()
                        .filter_map(|m| m["graftPoint"].as_str())
                        .map(String::from)
                        .collect(),
                    "goxRefs" => matches
                        .iter()
                        .filter_map(|m| m["goxRefs"].as_str())
                        .map(String::from)
                        .collect(),
                    "prediction_locations" => {
                        let signature_vec: Vec<HashMap<String, Value>> = matches
                            .iter()
                            .filter_map(|m| m["signature"].as_array())
                            .flat_map(|arr| {
                                arr.iter()
                                    .map(|sig| serde_json::from_value(sig.clone()).ok())
                                    .filter_map(|s| s)
                            })
                            .collect();
                        signature_vec
                            .into_iter()
                            .flat_map(|s| s.clone().into_values())
                            .map(|v| v.to_string())
                            .collect()
                    }
                    "getdomains" => {
                        let domain_matches: Vec<Vec<String>> = matches
                            .iter()
                            .filter_map(|m| m["signature"].as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|sig| sig.as_str().map(String::from))
                                    .collect()
                            })
                            .collect();
                        domain_matches.concat()
                    }
                    _ => Vec::new(),
                };
                return Ok(Some(result));
            }
        }
    }

    Ok(None)
}
