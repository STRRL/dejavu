
use xml::reader::{EventReader, XmlEvent};
use super::model::{Paragraph, Line, Word, BoundingBox};

pub fn parse_hocr_xml(hocr: &str) -> Vec<Paragraph> {
    let mut result = Vec::new();
    let reader = EventReader::new(hocr.as_bytes());

    let mut current_par: Option<Paragraph> = None;
    let mut current_line: Option<Line> = None;
    let mut current_word: Option<Word> = None;

    for event in reader {
        match event {
            Ok(XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            }) => {
                let par = attributes
                    .iter()
                    .find(|item| item.name.local_name == "class" && item.value.contains("ocr_par"));
                if par.is_some() {
                    let id = attributes
                        .iter()
                        .find(|item| item.name.local_name == "id")
                        .unwrap()
                        .value
                        .clone();
                    let title = attributes
                        .iter()
                        .find(|item| item.name.local_name == "title")
                        .unwrap()
                        .value
                        .clone();
                    let bbox = title
                        .split(";")
                        .take(1)
                        .collect::<String>()
                        .split_whitespace()
                        .skip(1)
                        .map(|item| item.parse::<i32>().unwrap())
                        .collect::<Vec<i32>>();
                    let bounding_box = BoundingBox::new_i32(bbox[0], bbox[1], bbox[2], bbox[3]);
                    let lang = attributes
                        .iter()
                        .find(|item| item.name.local_name == "lang")
                        .unwrap()
                        .value
                        .clone();
                    current_par = Some(Paragraph::new(id, bounding_box, lang, Vec::new()));
                }

                let line = attributes.iter().find(|item| {
                    item.name.local_name == "class" && item.value.contains("ocr_line")
                });
                if line.is_some() {
                    let id = attributes
                        .iter()
                        .find(|item| item.name.local_name == "id")
                        .unwrap()
                        .value
                        .clone();
                    let title = attributes
                        .iter()
                        .find(|item| item.name.local_name == "title")
                        .unwrap()
                        .value
                        .clone();
                    let bbox = title
                        .split(";")
                        .take(1)
                        .collect::<String>()
                        .split_whitespace()
                        .skip(1)
                        .map(|item| item.parse::<i32>().unwrap())
                        .collect::<Vec<i32>>();
                    let bounding_box = BoundingBox::new_i32(bbox[0], bbox[1], bbox[2], bbox[3]);
                    let words = Vec::new();
                    current_line = Some(Line::new(id, bounding_box, words))
                }
                let word = attributes.iter().find(|item| {
                    item.name.local_name == "class" && item.value.contains("ocrx_word")
                });
                if word.is_some() {
                    let id = attributes
                        .iter()
                        .find(|item| item.name.local_name == "id")
                        .unwrap()
                        .value
                        .clone();
                    let title = attributes
                        .iter()
                        .find(|item| item.name.local_name == "title")
                        .unwrap()
                        .value
                        .clone();
                    let bbox = title
                        .split(";")
                        .take(1)
                        .collect::<String>()
                        .split_whitespace()
                        .skip(1)
                        .map(|item| item.parse::<i32>().unwrap())
                        .collect::<Vec<i32>>();
                    let bounding_box = BoundingBox::new_i32(bbox[0], bbox[1], bbox[2], bbox[3]);
                    let content = String::new();
                    current_word = Some(Word::new(id, bounding_box, content))
                }
            }
            Ok(XmlEvent::Characters(content)) => {
                if let Some(word) = current_word.as_mut() {
                    word.content = content;
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.local_name == "p" {
                    if let Some(par) = current_par {
                        result.push(par);
                        current_par = None;
                    }
                }
                if name.local_name == "span" {
                    if let Some(word) = current_word {
                        // closing word
                        if let Some(line) = current_line.as_mut() {
                            line.words.push(word);
                        }
                        current_word = None;
                    } else if let Some(line) = current_line {
                        // closing line
                        if let Some(par) = current_par.as_mut() {
                            par.lines.push(line);
                        }
                        current_line = None;
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                panic!("Error: {}", e)
            }
        }
    }
    return result;
}

#[cfg(test)]
mod tests {
    use super::parse_hocr_xml;
    use std::fs::File;
    use std::io::{BufReader, Read};

    #[test]
    fn test_parse_xml() -> anyhow::Result<()> {
        let file = File::open("static/tesseract.out.hocr")?;
        let mut file = BufReader::new(file);
        let mut str = String::new();
        file.read_to_string(&mut str);
        let result = parse_hocr_xml(&str);

        for p in result {
            let text = p.text();
            println!("Paragraph: {:}", text);
        }
        Ok(())
    }
}
