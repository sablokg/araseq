use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

/*
Gaurav Sablok,
gsablok@proton.me
*/

pub fn ipr_network_plot(
    tree_file: &str,
    arg_ipr: Option<&str>,
    arg_domain: Option<&str>,
    arg_type: Option<&str>,
) -> Result<Vec<String>, io::Error> {
    let file = File::open(tree_file)?;
    let reader = io::BufReader::new(file);
    let mut domaindict = HashMap::new();
    let mut doname = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("IPR") {
            doname = line.trim().to_string();
            domaindict.insert(doname.clone(), String::new());
        } else {
            if let Some(entry) = domaindict.get_mut(&doname) {
                entry.push_str(&line.trim());
            }
        }
    }

    let normalize_domain: HashMap<String, Vec<String>> = domaindict
        .iter()
        .map(|(k, v)| {
            let normalized = v
                .split("::")
                .map(|i| i.trim().replace("--", ""))
                .filter(|i| i.starts_with("IPR"))
                .collect();
            (k.split("::").next().unwrap_or("").to_string(), normalized)
        })
        .collect();

    let normalize_categories: HashMap<String, Vec<String>> = domaindict
        .iter()
        .map(|(k, v)| {
            let normalized = v
                .split("::")
                .map(|i| i.trim().replace("--", ""))
                .filter(|i| !i.starts_with("IPR"))
                .filter(|i| !i.is_empty())
                .collect();
            (k.split("::").nth(1).unwrap_or("").to_string(), normalized)
        })
        .collect();

    if arg_type == Some("interpro") {
        if let Some(ipr) = arg_ipr {
            return Ok(normalize_domain.get(ipr).cloned().unwrap_or_default());
        }
    }

    if arg_type == Some("domain") {
        if let Some(domain) = arg_domain {
            return Ok(normalize_categories
                .get(domain)
                .cloned()
                .unwrap_or_default());
        }
    }

    Ok(vec![])
}
