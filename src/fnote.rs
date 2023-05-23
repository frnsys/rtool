use lazy_static::lazy_static;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::path::PathBuf;
use lol_html::html_content::ContentType;
use lol_html::{rewrite_str, element, RewriteStrSettings};

fn extract_footnotes(html: &str) -> Vec<String> {
    lazy_static! {
        static ref SELECTOR: Selector = Selector::parse(".footnotes li").unwrap();
    }

    let doc = Html::parse_document(&html);
    doc.select(&SELECTOR).map(|el| {
        el.html()
    }).collect()
}

pub fn update_footnotes(path: PathBuf) {
    let html = std::fs::read_to_string(path)
        .expect("Couldn't read file");

    // First extract the footnote details.
    // Can't extract inner HTML using `lol_html` so have to do it this way.
    let footnotes = extract_footnotes(&html);

    // Then do a first pass to figure out
    // mapping of old footnote ids to new ids.
    let mut index: usize = 1; // 1-indexed
    let mut positions: HashMap<String, usize> = HashMap::default();
    let html = rewrite_str(
        &html,
        RewriteStrSettings {
            element_content_handlers: vec![
                element!(".fn", |el| {
                    let href = el.get_attribute("href").unwrap();
                    let id: String = href.chars().skip(1).collect();

                    el.set_inner_content(&index.to_string(), ContentType::Text);
                    el.set_attribute("href", &format!("#{}", index))?;
                    el.set_attribute("id", &format!("fn-{}", index))?;

                    positions.insert(id, index);

                    index += 1;

                    Ok(())
                }),

                // Remove old footnote details
                element!(".footnotes li", |el| {
                    el.remove();
                    Ok(())
                })
            ],
            ..RewriteStrSettings::default()
        }
    ).unwrap();

    // Update footnotes
    let mut footnotes: Vec<(String, usize)> = footnotes.iter().map(|footnote_html| {
        let mut new_id = 0;
        let html = rewrite_str(&footnote_html, RewriteStrSettings {
            element_content_handlers: vec![
                // <a name="1">1</a>
                element!("a[name]", |el| {
                    let id = el.get_attribute("name").unwrap();
                    new_id = *positions.get(&id).unwrap();

                    let new_id_str = new_id.to_string();
                    el.set_inner_content(&new_id_str, ContentType::Text);
                    el.set_attribute("name", &new_id_str)?;
                    Ok(())
                }),

                // <a href="#fn-1">⮌ </a>
                element!("a[href^='#fn-']", |el| {
                    let id: String = el.get_attribute("href").unwrap()
                        .chars().skip(4).collect();
                    let new_id = *positions.get(&id).unwrap();

                    el.set_attribute("href", &format!("#fn-{}", &new_id))?;
                    Ok(())
                })
            ],
            ..RewriteStrSettings::default()
        }).unwrap();

        (html, new_id)
    }).collect();

    // Then sort to match the new order
    footnotes.sort_by(|a, b| a.1.cmp(&b.1));

    // Insert the updated footnotes
    let html = rewrite_str(
        &html,
        RewriteStrSettings {
            element_content_handlers: vec![
                // Remove old footnote details
                element!(".footnotes ul", |el| {
                    let footnotes_html = footnotes.iter()
                        .map(|(html, _)| html.as_str())
                        .collect::<Vec<&str>>().join("\n");
                    el.set_inner_content(&footnotes_html, ContentType::Html);
                    Ok(())
                })
            ],
            ..RewriteStrSettings::default()
        }
    ).unwrap();

    // Print the whole document
    println!("{}", html);
}
