use super::structure::Document;

pub fn parse(s: &String) -> Box<Document> {
    let mut document = Document::new();
    
    Box::new(document)
}