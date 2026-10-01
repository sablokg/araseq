mod args;
use crate::args::CommandParse;
use crate::args::Commands;
use clap::Parser;
mod agi;
mod agiassoc;
mod agigff;
mod agigo;
mod api;
mod domain;
mod frequency;
mod gffencoder;
mod ipr;
mod plantoml;
mod structfile;
mod tair;
mod tairagi;
mod uniprot;
mod web;
use self::domain::domain_analyzer;
use crate::agi::agi_uniprot;
use crate::agiassoc::get_agi_description_information;
use crate::agigff::agi_coordinates;
use crate::agigo::get_agi_go_information;
use crate::api::read_tair_ncbi;
use crate::gffencoder::get_tair_coordinates;
use crate::ipr::ipr_network_plot;
use crate::plantoml::plantoml_fetcher;
use crate::structfile::PathFastaFile;
use crate::tair::uniprot_tair;
use crate::tairagi::get_agi_tair_information;
use crate::uniprot::uniprot_agi;
use crate::web::build_router;
use figlet_rs::FIGfont;

/*
Gaurav Sablok,
gsablok@proton.me
*/

fn main() {
    let fontgenerate = FIGfont::standard().unwrap();
    let repgenerate = fontgenerate.convert("AraPY");
    println!("{}", repgenerate.unwrap());
    let argparse = CommandParse::parse();
    match &argparse.command {
        Commands::UNIPROTAGI {
            agifile,
            idsfile,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let result = uniprot_agi(agifile, idsfile).unwrap();
                for (uniprot_id, agi_list) in result {
                    println!("Uniprot: {}, AGI: {:?}", uniprot_id, agi_list);
                }
            });
        }
        Commands::AGIGO {
            idsfile,
            associationfile,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let results = get_agi_go_information(idsfile, associationfile).unwrap();
                for (agi, go) in results {
                    println!("AGI: {}, GO: {}", agi, go);
                }
            });
        }
        Commands::AGICoordinate {
            idsfile,
            gfffile,
            genetype,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let results = agi_coordinates(idsfile, gfffile, Some(genetype)).unwrap();
                for record in results {
                    println!("{:?}", record);
                }
            });
        }
        Commands::TAIRUNIPROT {
            taiuniprotfile,
            idsfile,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let results = uniprot_tair(taiuniprotfile, idsfile).unwrap();
                for (uniprot, locus_agi) in results {
                    println!("UniProt: {}, Locus/AGI: {:?}", uniprot, locus_agi);
                }
            });
        }
        Commands::AGIUNIPROT {
            agiuniprotfile,
            idsfiles,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let results = agi_uniprot(agiuniprotfile, idsfiles).unwrap();
                for (agi, uniprots) in results {
                    println!("AGI: {}, UniProt: {:?}", agi, uniprots);
                }
            });
        }
        Commands::AGITAIR {
            idsfile,
            associationfile,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let results = get_agi_tair_information(idsfile, associationfile).unwrap();
                for (agi, tair) in results {
                    println!("AGI: {}, TAIR: {}", agi, tair);
                }
            });
        }
        Commands::AGIDescription {
            idsfile,
            associaitonfile,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let results = get_agi_description_information(idsfile, associaitonfile).unwrap();
                for (agi, description) in results {
                    println!("AGI: {}, Description: {}", agi, description);
                }
            });
        }
        Commands::AGIPUBMDED {
            goslim,
            outputfile,
            locus,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| match read_tair_ncbi(goslim, outputfile, locus) {
                Ok(results) => {
                    for (url, abstract_text) in results {
                        println!("URL: {}\nAbstract: {}\n", url, abstract_text);
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            });
        }
        Commands::GFFEncoder {
            gffstring,
            genestring,
            exonstring,
            three_prime_utrstring,
            five_prime_utrstring,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let command = get_tair_coordinates(
                    gffstring,
                    *genestring,
                    *exonstring,
                    *three_prime_utrstring,
                    *five_prime_utrstring,
                )
                .unwrap();
                println!("The command has finished:{:?}", command);
                for coord in command {
                    println!("{:?}", coord);
                }
            });
        }
        Commands::PlantOnto {
            pathontology,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let command = plantoml_fetcher(pathontology).unwrap();
                print!("The file has been written:{:?}", command);
            })
        }
        Commands::DomainAnalyzer {
            domainfile,
            domainarg,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let command = domain_analyzer(domainfile, Some(domainarg)).unwrap();
                match command {
                    Some(values) => {
                        for value in &values {
                            println!("{}", value);
                        }
                        println!("The analyzed domains: {} entries found", values.len());
                    }
                    None => println!("No domains found for the given arguments."),
                }
            });
        }
        Commands::IPRAnalyzer {
            iprfile,
            ipr,
            domain,
            typedomain,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let command =
                    ipr_network_plot(iprfile, Some(ipr), Some(domain), Some(typedomain)).unwrap();
                for entry in command.iter() {
                    println!("{:?}", entry);
                }
                println!("The command has finished, {} entries found", command.len());
            });
        }
        Commands::FrequencyMotif {
            pathfileinput,
            motifinput,
            thread,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread.parse::<usize>().unwrap())
                .build()
                .unwrap();
            pool.install(|| {
                let command = PathFastaFile {
                    filepath: pathfileinput.to_string(),
                    motif: motifinput.to_string(),
                };
                match command.frequencymotif() {
                    Ok(valueoutput) => {
                        if let Some((motif, count)) = valueoutput.first() {
                            println!("The motif and the count are: {}\t{}", motif, count);
                        } else {
                            println!("No sequences were found to search for the motif.");
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            });
        }
        Commands::Serve { port } => {
            let runtime = tokio::runtime::Runtime::new().expect("failed to start async runtime");
            runtime.block_on(async {
                let app = build_router();
                let address: std::net::SocketAddr = format!("0.0.0.0:{}", port)
                    .parse()
                    .expect("invalid port supplied to --port");
                println!("AraPY web frontend running at http://localhost:{}", port);
                axum::Server::bind(&address)
                    .serve(app.into_make_service())
                    .await
                    .expect("web server crashed");
            });
        }
    }
}
