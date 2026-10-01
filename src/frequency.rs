use crate::structfile::PathFastaFile;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

/*
Gaurav Sablok,
gsablok@proton.me
*/

impl PathFastaFile {
    pub fn frequencymotif(&self) -> Result<Vec<(String, usize)>, Box<dyn Error>> {
        let fileopen = File::open(self.filepath.clone()).expect("file not present");
        let fileread = BufReader::new(fileopen);
        let mut fastaseqvec: Vec<String> = Vec::new();
        let mut _fastaname: Vec<String> = Vec::new();
        for i in fileread.lines() {
            let line = i.expect("file not present");
            if line.starts_with(">") {
                _fastaname.push(line.strip_prefix(">").unwrap().to_string())
            }
            if !line.starts_with(">") {
                fastaseqvec.push(line);
            }
        }

        let motif_len = self.motif.len();
        if motif_len == 0 {
            return Err("motif cannot be empty".into());
        }

        let mut motifvec: Vec<Vec<String>> = Vec::new();
        for i in fastaseqvec.iter() {
            if i.len() < motif_len {
                continue;
            }
            let value = i
                .as_bytes()
                .windows(motif_len)
                .map(|x| std::str::from_utf8(x).unwrap())
                .map(|x| x.to_string())
                .collect::<Vec<_>>();
            motifvec.push(value);
        }

        // Count every k-mer window, across every sequence, that matches the
        // requested motif exactly (case-sensitive), instead of only looking
        // at the very first window.
        let finalmotif = motifvec.iter().flatten().collect::<Vec<&String>>();
        let count = finalmotif.iter().filter(|window| ***window == self.motif).count();

        Ok(vec![(self.motif.clone(), count)])
    }
}
