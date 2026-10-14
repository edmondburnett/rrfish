use crate::doc::{Link, Section};

#[derive(Default)]
pub struct SectionBuilder {
    sections: Vec<Section>,
    section_line: u32,
    current_text: String,
    first_h1: Option<String>,
    headings: Vec<(u8, String)>,
    in_heading: Option<(u8, String)>,
    links: Vec<Link>,
    code_langs: Vec<String>,
}

/// What a [`SectionBuilder`] produces, independent of the source format.
pub struct Built {
    pub sections: Vec<Section>,
    pub first_h1: Option<String>,
    pub links: Vec<Link>,
}

impl SectionBuilder {
    pub fn new() -> Self {
        Self {
            section_line: 1,
            ..Default::default()
        }
    }

    pub fn start_heading(&mut self, level: u8, line: u32) {
        self.close_section();
        self.section_line = line;
        self.in_heading = Some((level, String::new()));
    }

    pub fn end_heading(&mut self) {
        let Some((level, title)) = self.in_heading.take() else {
            return;
        };
        let title = title.trim().to_string();
        if level == 1 && self.first_h1.is_none() {
            self.first_h1 = Some(title.clone());
        }
        self.headings.retain(|(existing_level, _)| *existing_level < level);
        self.headings.push((level, title));
    }

    /// Text goes to the heading title if inside one, else to the section body.
    pub fn push_text(&mut self, text: &str) {
        if let Some((_level, title)) = &mut self.in_heading {
            title.push_str(text);
        } else {
            self.current_text.push_str(text);
        }
    }

    pub fn push_link(&mut self, link: Link) {
        self.links.push(link);
    }

    pub fn push_code_lang(&mut self, lang: &str) {
        let lang = lang.trim().to_lowercase();
        if lang.is_empty() || self.code_langs.contains(&lang) {
            return;
        }
        self.code_langs.push(lang);
    }

    fn close_section(&mut self) {
        let current_text = std::mem::take(&mut self.current_text);
        let code_langs = std::mem::take(&mut self.code_langs);
        if current_text.trim().is_empty() {
            return;
        }

        let mut heading_path = Vec::new();
        for (_, title) in &self.headings {
            heading_path.push(title.clone());
        }

        self.sections.push(Section {
            heading_path,
            line: self.section_line,
            text: current_text.trim().to_string(),
            code_langs,
        });
    }

    pub fn finish(mut self) -> Built {
        self.close_section();
        Built {
            sections: self.sections,
            first_h1: self.first_h1,
            links: self.links,
        }
    }
}
