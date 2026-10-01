#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Coordinate {
    pub id: String,
    pub start: i32,
    pub end: i32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct GffRecord {
    pub gene_type: String,
    pub start: i32,
    pub end: i32,
    pub gene_id: String,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct PathFastaFile {
    pub filepath: String,
    pub motif: String,
}
