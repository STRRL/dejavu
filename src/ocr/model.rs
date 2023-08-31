use std::thread::current;

use super::MarkupBox;

#[derive(Debug)]
pub struct BoundingBox {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl BoundingBox {
    pub fn new(left: u32, top: u32, right: u32, bottom: u32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn new_i32(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left: left as u32,
            top: top as u32,
            right: right as u32,
            bottom: bottom as u32,
        }
    }
}

impl Into<MarkupBox> for BoundingBox {
    fn into(self) -> MarkupBox {
        MarkupBox::new(
            self.left,
            self.top,
            self.right - self.left,
            self.bottom - self.top,
        )
    }
}

#[derive(Debug)]
pub struct Paragraph {
    pub id: String,
    pub bounding_box: BoundingBox,
    pub language: String,
    pub lines: Vec<Line>,
}

impl Paragraph {
    pub fn new(id: String, bounding_box: BoundingBox, language: String, lines: Vec<Line>) -> Self {
        Self {
            id,
            bounding_box,
            language,
            lines,
        }
    }

    pub fn text(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.text())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[derive(Debug)]
pub struct Line {
    pub id: String,
    pub bounding_box: BoundingBox,
    pub words: Vec<Word>,
}
impl Line {
    pub fn new(id: String, bounding_box: BoundingBox, words: Vec<Word>) -> Self {
        Self {
            id,
            bounding_box,
            words,
        }
    }

    pub fn text(&self) -> String {
        self.words
            .iter()
            .map(|w| w.text())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[derive(Debug)]
pub struct Word {
    pub id: String,
    pub bounding_box: BoundingBox,
    pub content: String,
}

impl Word {
    pub fn new(id: String, bounding_box: BoundingBox, content: String) -> Self {
        Self {
            id,
            bounding_box,
            content,
        }
    }

    pub fn text(&self) -> String {
        self.content.clone()
    }
}

use xml::reader::{EventReader, XmlEvent};
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
