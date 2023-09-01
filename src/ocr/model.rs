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
