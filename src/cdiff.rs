use regex::Regex;
use collapse::collapse;
use std::{fs, path::PathBuf};
use scraper::{Html, Selector};
use textdistance::nstr;
use lazy_static::lazy_static;

static MAX_DIST: f64 = 1.0;
static TEXT_SELECTORS: &str = "p, h1, h2, h3, h4, h5, h6, li, figcaption";

struct LabeledBlock {
    text: String,
    index: Option<usize>,
}

// Parse an HTML string and extract
// blocks of text content.
fn extract_blocks(html: &str) -> Vec<String> {
    lazy_static! {
        static ref SELECTOR: Selector = Selector::parse(TEXT_SELECTORS).unwrap();

        // Kind of hacky way to get footnotes to be consistent
        // When you download a Google Doc as HTML, the footnotes are turned into `[1]`.
        // In the reports I make, footnotes have the markup:
        //  <a something class="fn" something>1</a>
        // which just gives the text content `1`.
        // So this will convert these into `[1]` instead.
        // Note that this expects that footnotes use an `a` tag with e.g. `id="fn-1"`.
        static ref RE: Regex = Regex::new(r#"<a [^>]*id="fn-\d+"[^>]*>(?P<n>\d+)<\/a>"#).unwrap();
    }
    let html = RE.replace_all(html, "[$n]");

    let doc = Html::parse_document(&html);
    doc.select(&SELECTOR).filter_map(|el| {
        let texts: Vec<&str> = el.text().collect();
        if texts.is_empty() {
            None
        } else {
            // Clean up the text a bit
            let text = texts.join("");
            Some(collapse(&text).trim().to_string())
        }
    }).collect()
}

pub fn cdiff(source_path: PathBuf, current_path: PathBuf) {
    // Load the source (source-of-truth) file.
    let source_html = std::fs::read_to_string(source_path).expect("Couldn't read file");
    let source_blocks = extract_blocks(&source_html);

    let mut source_sequence = vec![];
    for (i, text) in source_blocks.into_iter().enumerate() {
        source_sequence.push(LabeledBlock {
            index: Some(i),
            text,
        });
    }

    // Load the current file.
    let current_html = std::fs::read_to_string(current_path).expect("Couldn't read file");
    let current_blocks = extract_blocks(&current_html);

    // Find the most similar line for each of the current lines
    let mut current_sequence = vec![];
    for text in current_blocks {
        let mut best: (Option<usize>, f64) = (None, f64::INFINITY);
        for block in &source_sequence {
            let dist = nstr::levenshtein(&text, &block.text);
            if dist <= MAX_DIST && dist < best.1 {
                best = (block.index, dist);
                if dist <= 1e-3 {
                    break
                }
            }
        }
        current_sequence.push(LabeledBlock {
            index: best.0,
            text,
        });
    }

    // Generate the synthetic files to diff.
    // Aligned so that lines from each file
    // match up.
    let mut source_parts = vec![];
    let mut current_parts = vec![];
    for block in &source_sequence {
        source_parts.push(block.text.as_str());
        let current_part = current_sequence.iter().find_map(|b| {
            if b.index == block.index {
                Some(b.text.as_str())
            } else {
                None
            }
        }).unwrap_or("");
        current_parts.push(current_part);
    }

    // Add the removed parts,
    // i.e. parts with no matches
    for block in &current_sequence {
        if block.index.is_none() {
            current_parts.push(block.text.as_str());
        }
    }

    // To keep line length consistent,
    // add empty lines for the removed lines from
    // the current version.
    while source_parts.len() < current_parts.len() {
        source_parts.push("");
    }

    let source = source_parts.join("\n\n");
    fs::write("/tmp/source", source).expect("Unable to write file");

    let current = current_parts.join("\n\n");
    fs::write("/tmp/current", current).expect("Unable to write file");

    // Open with vimdiff.
    // Current will be on the left side,
    // Source will be on the right
    subprocess::Exec::cmd("nvim")
        .arg("-d")
        .arg("/tmp/current")
        .arg("/tmp/source")
        .join().unwrap();
}



