use axum::extract::Form;
use axum::response::Html;
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;

use crate::agi::agi_uniprot;
use crate::agiassoc::get_agi_description_information;
use crate::agigff::agi_coordinates;
use crate::agigo::get_agi_go_information;
use crate::api::read_tair_ncbi;
use crate::domain::domain_analyzer;
use crate::gffencoder::get_tair_coordinates;
use crate::ipr::ipr_network_plot;
use crate::plantoml::plantoml_fetcher;
use crate::structfile::PathFastaFile;
use crate::tair::uniprot_tair;
use crate::tairagi::get_agi_tair_information;
use crate::uniprot::uniprot_agi;

/*
Gaurav Sablok,
gsablok@proton.me
*/

/// Builds the Axum router for the AraPY web frontend: one landing page and
/// one POST endpoint per CLI subcommand, all rendered with Bootstrap 5.
pub fn build_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/run/uniprot-agi", post(run_uniprot_agi))
        .route("/run/agi-go", post(run_agi_go))
        .route("/run/agi-coordinate", post(run_agi_coordinate))
        .route("/run/tair-uniprot", post(run_tair_uniprot))
        .route("/run/agi-uniprot", post(run_agi_uniprot))
        .route("/run/agi-tair", post(run_agi_tair))
        .route("/run/agi-description", post(run_agi_description))
        .route("/run/agi-pubmed", post(run_agi_pubmed))
        .route("/run/gff-encoder", post(run_gff_encoder))
        .route("/run/plant-onto", post(run_plant_onto))
        .route("/run/domain-analyzer", post(run_domain_analyzer))
        .route("/run/ipr-analyzer", post(run_ipr_analyzer))
        .route("/run/frequency-motif", post(run_frequency_motif))
}

// ---------------------------------------------------------------------
// shared layout + helpers
// ---------------------------------------------------------------------

fn parse_threads(raw: &str) -> usize {
    raw.trim().parse::<usize>().unwrap_or(1).max(1)
}

/// Runs `f` inside a dedicated rayon thread pool sized by the "threads"
/// field of the submitted form, mirroring how the CLI drives every command
/// through `rayon::ThreadPoolBuilder`, and off the async executor thread.
async fn run_pooled<T, F>(threads: usize, f: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("failed to build thread pool")
            .install(f)
    })
    .await
    .expect("worker thread panicked")
}

fn layout(title: &str, body: String) -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en" data-bs-theme="light">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>AraPY &middot; {title}</title>
  <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" rel="stylesheet">
  <style>
    body {{ background-color: #f5f7f4; }}
    .navbar-brand {{ font-weight: 700; letter-spacing: .02em; }}
    .tool-card {{ transition: box-shadow .15s ease-in-out; }}
    .tool-card:hover {{ box-shadow: 0 .5rem 1rem rgba(0,0,0,.1); }}
    pre.result-box {{ background:#0f172a; color:#e2e8f0; padding:1rem; border-radius:.5rem; max-height:480px; overflow:auto; }}
    footer {{ color:#6c757d; font-size:.875rem; }}
  </style>
</head>
<body>
  <nav class="navbar navbar-expand-lg navbar-dark bg-success">
    <div class="container">
      <a class="navbar-brand" href="/">&#127793; AraPY</a>
      <span class="navbar-text text-white-50">Arabidopsis genomics toolkit &mdash; web frontend</span>
    </div>
  </nav>
  <main class="container py-4">
    {body}
  </main>
  <footer class="container pb-4 text-center">
    AraPY &middot; Gaurav Sablok &middot; served locally over Axum &mdash; file paths must be readable on this machine.
  </footer>
  <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js"></script>
</body>
</html>"#,
        title = title,
        body = body
    ))
}

fn thread_field() -> &'static str {
    r#"<div class="col-md-3">
        <label class="form-label">Threads</label>
        <input type="number" min="1" class="form-control" name="thread" value="4" required>
      </div>"#
}

fn text_field(label: &str, name: &str, placeholder: &str) -> String {
    format!(
        r#"<div class="col-md-6">
            <label class="form-label">{label}</label>
            <input type="text" class="form-control" name="{name}" placeholder="{placeholder}" required>
          </div>"#
    )
}

fn tool_card(id: &str, title: &str, description: &str, action: &str, fields_html: String) -> String {
    format!(
        r#"<div class="card tool-card shadow-sm mb-4" id="{id}">
      <div class="card-body">
        <h5 class="card-title">{title}</h5>
        <p class="card-text text-muted">{description}</p>
        <form method="post" action="{action}">
          <div class="row g-3 align-items-end">
            {fields_html}
            <div class="col-12">
              <button type="submit" class="btn btn-success">Run</button>
            </div>
          </div>
        </form>
      </div>
    </div>"#
    )
}

fn result_page(title: &str, ok: bool, body_html: String) -> Html<String> {
    let badge = if ok {
        r#"<span class="badge text-bg-success">Success</span>"#
    } else {
        r#"<span class="badge text-bg-danger">Error</span>"#
    };
    let page = format!(
        r#"<div class="d-flex justify-content-between align-items-center mb-3">
      <h2 class="mb-0">{title}</h2>
      {badge}
    </div>
    {body_html}
    <a href="/" class="btn btn-outline-secondary mt-3">&larr; Back to tools</a>"#
    );
    layout(title, page)
}

fn error_html(err: impl std::fmt::Display) -> String {
    format!(
        r#"<div class="alert alert-danger" role="alert">{}</div>"#,
        html_escape(&err.to_string())
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Renders a two-column table where the second column's values are shown
/// with their `Display` representation (plain strings, no debug quoting).
fn table_from_pairs<A: std::fmt::Display, B: std::fmt::Display>(
    col_a: &str,
    col_b: &str,
    rows: Vec<(A, B)>,
) -> String {
    table_rows(col_a, col_b, rows, |b| b.to_string())
}

/// Renders a two-column table where the second column's values are shown
/// with their `Debug` representation (useful for `Vec<String>` cells).
fn table_from_pairs_debug<A: std::fmt::Display, B: std::fmt::Debug>(
    col_a: &str,
    col_b: &str,
    rows: Vec<(A, B)>,
) -> String {
    table_rows(col_a, col_b, rows, |b| format!("{:?}", b))
}

fn table_rows<A: std::fmt::Display, B>(
    col_a: &str,
    col_b: &str,
    rows: Vec<(A, B)>,
    render_b: impl Fn(&B) -> String,
) -> String {
    if rows.is_empty() {
        return r#"<div class="alert alert-warning">No matching records were found.</div>"#.to_string();
    }
    let mut out = format!(
        r#"<div class="table-responsive"><table class="table table-striped table-sm">
        <thead><tr><th>{col_a}</th><th>{col_b}</th></tr></thead><tbody>"#
    );
    for (a, b) in &rows {
        out.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>",
            html_escape(&a.to_string()),
            html_escape(&render_b(b))
        ));
    }
    out.push_str("</tbody></table></div>");
    out
}

// ---------------------------------------------------------------------
// index page
// ---------------------------------------------------------------------

async fn index() -> Html<String> {
    let mut body = String::new();
    body.push_str(r#"<div class="mb-4"><p class="lead">Run any AraPY analysis directly from the browser. Every field below maps 1:1 to the equivalent <code>araseq</code> CLI subcommand; paths are read from this server's filesystem.</p></div>"#);

    body.push_str(&tool_card(
        "uniprot-agi",
        "UniProt &rarr; AGI",
        "Cross-reference a UniProt2AGI mapping file against a list of IDs.",
        "/run/uniprot-agi",
        format!(
            "{}{}{}",
            text_field("AGI/UniProt mapping file", "agifile", "Uniprot2AGI-Jul2023.txt"),
            text_field("IDs file", "idsfile", "all_ids.txt"),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "agi-go",
        "AGI &rarr; GO information",
        "Look up GO terms for a set of AGI IDs from a gene_association.tair-style file.",
        "/run/agi-go",
        format!(
            "{}{}{}",
            text_field("IDs file", "idsfile", "gene_id.txt"),
            text_field("Association file", "associationfile", "gene_association.tair"),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "agi-coordinate",
        "AGI coordinates",
        "Fetch gene/exon/UTR coordinates for AGI IDs from a GFF3 file.",
        "/run/agi-coordinate",
        format!(
            "{}{}{}{}",
            text_field("IDs file", "idsfile", "gene_id.txt"),
            text_field("GFF file", "gfffile", "TAIR10_GFF3_genes.gff"),
            format!(
                r#"<div class="col-md-6">
                  <label class="form-label">Feature type</label>
                  <select class="form-select" name="genetype">
                    <option value="gene">gene</option>
                    <option value="exon">exon</option>
                    <option value="three_prime_UTR">three_prime_UTR</option>
                    <option value="five_prime_UTR">five_prime_UTR</option>
                  </select>
                </div>"#
            ),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "tair-uniprot",
        "TAIR &rarr; UniProt",
        "Cross-reference a TAIR2Uniprot mapping file against a list of locus/AGI IDs.",
        "/run/tair-uniprot",
        format!(
            "{}{}{}",
            text_field("TAIR2Uniprot mapping file", "taiuniprotfile", "TAIR2UniprotMapping-Jul2023.txt"),
            text_field("IDs file", "idsfile", "all_ids.txt"),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "agi-uniprot",
        "AGI &rarr; UniProt",
        "Cross-reference an AGI2Uniprot mapping file against a list of IDs.",
        "/run/agi-uniprot",
        format!(
            "{}{}{}",
            text_field("AGI2Uniprot mapping file", "agiuniprotfile", "AGI2uniprot-Jul2023.txt"),
            text_field("IDs file", "idsfiles", "all_ids.txt"),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "agi-tair",
        "AGI &rarr; TAIR communication",
        "Resolve TAIR communication IDs for a set of AGI IDs.",
        "/run/agi-tair",
        format!(
            "{}{}{}",
            text_field("IDs file", "idsfile", "gene_id.txt"),
            text_field("Association file", "associationfile", "gene_association.tair"),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "agi-description",
        "AGI &rarr; description",
        "Resolve gene descriptions for a set of AGI IDs.",
        "/run/agi-description",
        format!(
            "{}{}{}",
            text_field("IDs file", "idsfile", "gene_id.txt"),
            text_field("Association file", "associaitonfile", "gene_association.tair"),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "agi-pubmed",
        "PubMed abstracts",
        "Fetch PubMed abstracts linked to a locus via a GOSLIM association file (makes live HTTP requests).",
        "/run/agi-pubmed",
        format!(
            "{}{}{}{}",
            text_field("GOSLIM file", "goslim", "ATH_GO_GOSLIM.txt"),
            text_field("Output file (intermediate, written on server)", "outputfile", "goslim_clean.txt"),
            text_field("Locus", "locus", "AT5G27680"),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "gff-encoder",
        "GFF encoder",
        "Extract gene / exon / UTR coordinates from a GFF3 file.",
        "/run/gff-encoder",
        format!(
            "{}{}{}",
            text_field("GFF file", "gffstring", "TAIR10_GFF3_genes.gff"),
            format!(
                r#"<div class="col-md-6">
                  <label class="form-label">Feature type</label>
                  <select class="form-select" name="feature">
                    <option value="gene">gene</option>
                    <option value="exon">exon</option>
                    <option value="three_prime_utr">three_prime_UTR</option>
                    <option value="five_prime_utr">five_prime_UTR</option>
                  </select>
                </div>"#
            ),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "plant-onto",
        "Plant Ontology extractor",
        "Extract unique PO_XXXXXXX ontology terms referenced in a file.",
        "/run/plant-onto",
        format!(
            "{}{}",
            text_field("Ontology file", "pathontology", "po.owl"),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "domain-analyzer",
        "InterProScan domain analyzer",
        "Extract fields (sequence, signature, evalue, score, ...) from an InterProScan JSON report.",
        "/run/domain-analyzer",
        format!(
            "{}{}{}",
            text_field("InterProScan JSON file", "domainfile", "ipr_scan.json"),
            format!(
                r#"<div class="col-md-6">
                  <label class="form-label">Field to extract</label>
                  <select class="form-select" name="domainarg">
                    <option value="sequence">sequence</option>
                    <option value="interpro_normalize">interpro_normalize</option>
                    <option value="signature">signature</option>
                    <option value="location">location</option>
                    <option value="evalue">evalue</option>
                    <option value="score">score</option>
                    <option value="modelac">modelac</option>
                    <option value="scope">scope</option>
                    <option value="accession">accession</option>
                    <option value="name">name</option>
                  </select>
                </div>"#
            ),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "ipr-analyzer",
        "IPR tree analyzer",
        "Query an InterPro tree file for either InterPro IDs under an entry or category labels under a domain.",
        "/run/ipr-analyzer",
        format!(
            "{}{}{}{}{}",
            text_field("IPR tree file", "iprfile", "interpro_tree.txt"),
            text_field("InterPro ID", "ipr", "IPR000001"),
            text_field("Domain", "domain", "IPR000001"),
            format!(
                r#"<div class="col-md-6">
                  <label class="form-label">Query type</label>
                  <select class="form-select" name="typedomain">
                    <option value="interpro">interpro</option>
                    <option value="domain">domain</option>
                  </select>
                </div>"#
            ),
            thread_field()
        ),
    ));

    body.push_str(&tool_card(
        "frequency-motif",
        "Motif frequency",
        "Count how many times an exact motif occurs across every sliding window of a FASTA file's sequences.",
        "/run/frequency-motif",
        format!(
            "{}{}{}",
            text_field("FASTA file", "pathfileinput", "sequences.fasta"),
            text_field("Motif", "motifinput", "ATG"),
            thread_field()
        ),
    ));

    layout("Tools", body)
}

// ---------------------------------------------------------------------
// form payloads
// ---------------------------------------------------------------------

#[derive(Deserialize)]
struct TwoFileForm {
    #[serde(alias = "agifile", alias = "taiuniprotfile", alias = "agiuniprotfile")]
    file_a: String,
    #[serde(alias = "idsfile", alias = "idsfiles")]
    file_b: String,
    thread: String,
}

#[derive(Deserialize)]
struct AssocForm {
    idsfile: String,
    #[serde(alias = "associationfile", alias = "associaitonfile")]
    associationfile: String,
    thread: String,
}

#[derive(Deserialize)]
struct CoordinateForm {
    idsfile: String,
    gfffile: String,
    genetype: String,
    thread: String,
}

#[derive(Deserialize)]
struct PubmedForm {
    goslim: String,
    outputfile: String,
    locus: String,
    thread: String,
}

#[derive(Deserialize)]
struct GffEncoderForm {
    gffstring: String,
    feature: String,
    thread: String,
}

#[derive(Deserialize)]
struct SingleFileForm {
    #[serde(alias = "pathontology", alias = "pathfileinput")]
    path: String,
    thread: String,
}

#[derive(Deserialize)]
struct DomainForm {
    domainfile: String,
    domainarg: String,
    thread: String,
}

#[derive(Deserialize)]
struct IprForm {
    iprfile: String,
    ipr: String,
    domain: String,
    typedomain: String,
    thread: String,
}

#[derive(Deserialize)]
struct MotifForm {
    pathfileinput: String,
    motifinput: String,
    thread: String,
}

// ---------------------------------------------------------------------
// handlers
// ---------------------------------------------------------------------

async fn run_uniprot_agi(Form(input): Form<TwoFileForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let (a, b) = (input.file_a, input.file_b);
    let result = run_pooled(threads, move || uniprot_agi(&a, &b).map_err(|e| e.to_string())).await;
    match result {
        Ok(rows) => result_page(
            "UniProt \u{2192} AGI",
            true,
            table_from_pairs_debug("UniProt ID", "AGI list", rows),
        ),
        Err(e) => result_page("UniProt \u{2192} AGI", false, error_html(e)),
    }
}

async fn run_agi_go(Form(input): Form<AssocForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let (idsfile, associationfile) = (input.idsfile, input.associationfile);
    let result =
        run_pooled(threads, move || {
            get_agi_go_information(&idsfile, &associationfile).map_err(|e| e.to_string())
        })
        .await;
    match result {
        Ok(rows) => result_page("AGI \u{2192} GO", true, table_from_pairs("AGI", "GO term", rows)),
        Err(e) => result_page("AGI \u{2192} GO", false, error_html(e)),
    }
}

async fn run_agi_coordinate(Form(input): Form<CoordinateForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let (idsfile, gfffile, genetype) = (input.idsfile, input.gfffile, input.genetype);
    let result = run_pooled(threads, move || {
        agi_coordinates(&idsfile, &gfffile, Some(genetype.as_str())).map_err(|e| e.to_string())
    })
    .await;
    match result {
        Ok(rows) => {
            let body = if rows.is_empty() {
                r#"<div class="alert alert-warning">No matching records were found.</div>"#.to_string()
            } else {
                let mut out = String::from(
                    r#"<div class="table-responsive"><table class="table table-striped table-sm"><thead><tr><th>AGI</th><th>Start</th><th>End</th></tr></thead><tbody>"#,
                );
                for row in rows {
                    out.push_str("<tr>");
                    for cell in row {
                        out.push_str(&format!("<td>{}</td>", html_escape(&cell)));
                    }
                    out.push_str("</tr>");
                }
                out.push_str("</tbody></table></div>");
                out
            };
            result_page("AGI coordinates", true, body)
        }
        Err(e) => result_page("AGI coordinates", false, error_html(e)),
    }
}

async fn run_tair_uniprot(Form(input): Form<TwoFileForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let (a, b) = (input.file_a, input.file_b);
    let result = run_pooled(threads, move || uniprot_tair(&a, &b).map_err(|e| e.to_string())).await;
    match result {
        Ok(rows) => result_page(
            "TAIR \u{2192} UniProt",
            true,
            table_from_pairs_debug("UniProt ID", "Locus/AGI", rows),
        ),
        Err(e) => result_page("TAIR \u{2192} UniProt", false, error_html(e)),
    }
}

async fn run_agi_uniprot(Form(input): Form<TwoFileForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let (a, b) = (input.file_a, input.file_b);
    let result = run_pooled(threads, move || agi_uniprot(&a, &b).map_err(|e| e.to_string())).await;
    match result {
        Ok(rows) => result_page(
            "AGI \u{2192} UniProt",
            true,
            table_from_pairs_debug("AGI", "UniProt list", rows),
        ),
        Err(e) => result_page("AGI \u{2192} UniProt", false, error_html(e)),
    }
}

async fn run_agi_tair(Form(input): Form<AssocForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let (idsfile, associationfile) = (input.idsfile, input.associationfile);
    let result =
        run_pooled(threads, move || {
            get_agi_tair_information(&idsfile, &associationfile).map_err(|e| e.to_string())
        })
        .await;
    match result {
        Ok(rows) => result_page(
            "AGI \u{2192} TAIR communication",
            true,
            table_from_pairs("AGI", "TAIR communication ID", rows),
        ),
        Err(e) => result_page("AGI \u{2192} TAIR communication", false, error_html(e)),
    }
}

async fn run_agi_description(Form(input): Form<AssocForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let (idsfile, associationfile) = (input.idsfile, input.associationfile);
    let result = run_pooled(threads, move || {
        get_agi_description_information(&idsfile, &associationfile).map_err(|e| e.to_string())
    })
    .await;
    match result {
        Ok(rows) => result_page(
            "AGI \u{2192} description",
            true,
            table_from_pairs("AGI", "Description", rows),
        ),
        Err(e) => result_page("AGI \u{2192} description", false, error_html(e)),
    }
}

async fn run_agi_pubmed(Form(input): Form<PubmedForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let (goslim, outputfile, locus) = (input.goslim, input.outputfile, input.locus);
    let result = run_pooled(threads, move || {
        read_tair_ncbi(&goslim, &outputfile, &locus).map_err(|e| e.to_string())
    })
    .await;
    match result {
        Ok(rows) => result_page(
            "PubMed abstracts",
            true,
            table_from_pairs("URL", "Abstract", rows),
        ),
        Err(e) => result_page("PubMed abstracts", false, error_html(e)),
    }
}

async fn run_gff_encoder(Form(input): Form<GffEncoderForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let gene = input.feature == "gene";
    let exon = input.feature == "exon";
    let three_prime = input.feature == "three_prime_utr";
    let five_prime = input.feature == "five_prime_utr";
    let gffstring = input.gffstring;
    let result = run_pooled(threads, move || {
        get_tair_coordinates(&gffstring, gene, exon, three_prime, five_prime)
    })
    .await;
    match result {
        Ok(rows) => {
            let flattened: Vec<(String, i32, i32)> = rows.into_iter().flatten().collect();
            let body = if flattened.is_empty() {
                r#"<div class="alert alert-warning">No matching records were found.</div>"#.to_string()
            } else {
                let mut out = String::from(
                    r#"<div class="table-responsive"><table class="table table-striped table-sm"><thead><tr><th>ID</th><th>Start</th><th>End</th></tr></thead><tbody>"#,
                );
                for (id, start, end) in flattened {
                    out.push_str(&format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                        html_escape(&id),
                        start,
                        end
                    ));
                }
                out.push_str("</tbody></table></div>");
                out
            };
            result_page("GFF encoder", true, body)
        }
        Err(e) => result_page("GFF encoder", false, error_html(e)),
    }
}

async fn run_plant_onto(Form(input): Form<SingleFileForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let path = input.path;
    let result = run_pooled(threads, move || plantoml_fetcher(&path).map_err(|e| e.to_string())).await;
    match result {
        Ok(terms) => {
            let body = if terms.is_empty() {
                r#"<div class="alert alert-warning">No PO terms found.</div>"#.to_string()
            } else {
                let items: String = terms
                    .iter()
                    .map(|t| format!("<li class=\"list-group-item\">{}</li>", html_escape(t)))
                    .collect();
                format!(r#"<ul class="list-group">{}</ul>"#, items)
            };
            result_page("Plant Ontology terms", true, body)
        }
        Err(e) => result_page("Plant Ontology terms", false, error_html(e)),
    }
}

async fn run_domain_analyzer(Form(input): Form<DomainForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let (domainfile, domainarg) = (input.domainfile, input.domainarg);
    let result = run_pooled(threads, move || domain_analyzer(&domainfile, Some(domainarg.as_str())))
        .await;
    match result {
        Ok(Some(values)) => {
            let body = if values.is_empty() {
                r#"<div class="alert alert-warning">No matching values found.</div>"#.to_string()
            } else {
                let items: String = values
                    .iter()
                    .map(|v| format!("<li class=\"list-group-item\">{}</li>", html_escape(v)))
                    .collect();
                format!(r#"<ul class="list-group">{}</ul>"#, items)
            };
            result_page("Domain analyzer", true, body)
        }
        Ok(None) => result_page(
            "Domain analyzer",
            true,
            r#"<div class="alert alert-warning">No result for that field.</div>"#.to_string(),
        ),
        Err(e) => result_page("Domain analyzer", false, error_html(e)),
    }
}

async fn run_ipr_analyzer(Form(input): Form<IprForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let (iprfile, ipr, domain, typedomain) =
        (input.iprfile, input.ipr, input.domain, input.typedomain);
    let result = run_pooled(threads, move || {
        ipr_network_plot(
            &iprfile,
            Some(ipr.as_str()),
            Some(domain.as_str()),
            Some(typedomain.as_str()),
        )
    })
    .await;
    match result {
        Ok(values) => {
            let body = if values.is_empty() {
                r#"<div class="alert alert-warning">No matching entries found.</div>"#.to_string()
            } else {
                let items: String = values
                    .iter()
                    .map(|v| format!("<li class=\"list-group-item\">{}</li>", html_escape(v)))
                    .collect();
                format!(r#"<ul class="list-group">{}</ul>"#, items)
            };
            result_page("IPR tree analyzer", true, body)
        }
        Err(e) => result_page("IPR tree analyzer", false, error_html(e)),
    }
}

async fn run_frequency_motif(Form(input): Form<MotifForm>) -> Html<String> {
    let threads = parse_threads(&input.thread);
    let target = PathFastaFile {
        filepath: input.pathfileinput,
        motif: input.motifinput,
    };
    let result =
        run_pooled(threads, move || target.frequencymotif().map_err(|e| e.to_string())).await;
    match result {
        Ok(rows) => {
            let body = if let Some((motif, count)) = rows.first() {
                format!(
                    r#"<pre class="result-box">motif: {}
count: {}</pre>"#,
                    html_escape(motif),
                    count
                )
            } else {
                r#"<div class="alert alert-warning">No sequences were found to search.</div>"#.to_string()
            };
            result_page("Motif frequency", true, body)
        }
        Err(e) => result_page("Motif frequency", false, error_html(e)),
    }
}
