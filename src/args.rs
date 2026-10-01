use clap::{Parser, Subcommand};
#[derive(Debug, Parser)]
#[command(
    name = "araseq",
    version = "1.0",
    about = "araseq.
       ************************************************
       Author Gaurav Sablok,
       Email: gsablok@proton.me
      ************************************************"
)]
pub struct CommandParse {
    /// subcommands for the specific actions
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// UNIPROTAGI
    UNIPROTAGI {
        /// path to the agi file
        agifile: String,
        /// ids file
        idsfile: String,
        /// thread for the analysis
        thread: String,
    },
    /// AGI Go information
    AGIGO {
        /// path to the ids file
        idsfile: String,
        /// path to the association file
        associationfile: String,
        /// thread for the analysis
        thread: String,
    },
    /// AGI Coordinate
    AGICoordinate {
        /// ids file
        idsfile: String,
        /// gff file
        gfffile: String,
        /// genetype
        genetype: String,
        /// thread for the analysis
        thread: String,
    },
    /// TAIRUNIPROT
    TAIRUNIPROT {
        /// tairuniprot file
        taiuniprotfile: String,
        /// ids files
        idsfile: String,
        /// thread for the analysis
        thread: String,
    },
    /// AGIUNIPROT
    AGIUNIPROT {
        /// path to the agi uniprot file
        agiuniprotfile: String,
        /// path to the ids file
        idsfiles: String,
        /// thread for the analysis
        thread: String,
    },
    /// AGI TAIR cross link
    AGITAIR {
        /// path to the idsfile
        idsfile: String,
        /// path to the association file
        associationfile: String,
        /// thread for the analysis
        thread: String,
    },
    /// AGI to descrption association
    AGIDescription {
        /// idsfile
        idsfile: String,
        /// association file
        associaitonfile: String,
        /// thread for the analysis
        thread: String,
    },
    /// Pubmed abstract approach
    AGIPUBMDED {
        /// path to the GOSLIM
        goslim: String,
        /// path to the outputfile
        outputfile: String,
        /// locus information
        locus: String,
        /// thread for the analysis
        thread: String,
    },
    /// encodes the GFF
    GFFEncoder {
        /// path to the gff
        gffstring: String,
        genestring: bool,
        exonstring: bool,
        three_prime_utrstring: bool,
        five_prime_utrstring: bool,
        /// thread for the analysis
        thread: String,
    },
    /// Plant OML analysis
    PlantOnto {
        /// path to the ontology file
        pathontology: String,
        /// thread for the analysis
        thread: String,
    },
    /// Domain analyzer
    DomainAnalyzer {
        /// file path to the domain file
        domainfile: String,
        /// domain argument for the filter
        domainarg: String,
        /// thread for the analysis
        thread: String,
    },
    /// IPR analyzer
    IPRAnalyzer {
        /// IPR tree file
        iprfile: String,
        /// ipr for the search
        ipr: String,
        /// domain for the search
        domain: String,
        /// type of the domain
        typedomain: String,
        /// thread for the analysis
        thread: String,
    },
    /// Frequencyestimate
    FrequencyMotif {
        /// path to the fasta file
        pathfileinput: String,
        /// motif to be searched across all the sequences
        motifinput: String,
        /// threads for the analysis
        thread: String,
    },
    /// Launches an Axum + Bootstrap web frontend exposing every command above
    /// as an HTML form in the browser, so files can be analyzed without the CLI.
    Serve {
        /// port to bind the web server to
        #[arg(default_value = "3000")]
        port: String,
    },
}
