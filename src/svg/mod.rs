#![allow(clippy::unused_io_amount)]
// We need to allow this because clippy doesn't seem to get that the writer is flushed by FileDocument

// This is how it's gonna work:
// svg file -> token stream -> syntax tree

//! This has all the types for constructing a syntax tree and for converting it to a string

pub mod lexer;
pub mod parser;

use std::io::{BufWriter, Write};

pub type Result = std::result::Result<(), std::io::Error>;
pub type Title = String;

#[derive(Debug, Clone)]
pub struct FileDocument(Vec<Node>);

impl FileDocument {
    /// Writes to a file
    pub fn output(&self, path: String) -> Result {
        let file = std::fs::File::create(path).unwrap();

        let mut writer = BufWriter::new(file);
        for node in self.0.iter() {
            node.print(&mut writer, 0usize)?;
        }
        writer.flush()?;
        Ok(())
    }
    /// Writes to a Vec<u8>
    pub fn print(&self, buf: &mut Vec<u8>) -> Result {
        let mut writer = BufWriter::new(VecBuffer::new(buf));
        for node in self.0.iter() {
            node.print(&mut writer, 0usize)?;
        }
        writer.flush()?;
        Ok(())
    }
}

impl std::ops::Deref for FileDocument {
    type Target = Vec<Node>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for FileDocument {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct VecBuffer<'a>(&'a mut Vec<u8>);

impl<'a> VecBuffer<'a> {
    pub fn new(vec: &'a mut Vec<u8>) -> Self {
        Self(vec)
    }
}

impl<'a> Write for VecBuffer<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for byte in buf.iter() {
            self.0.push(*byte);
        }
        Ok(buf.len())
    }

    /// This does nothing
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Don't forget to flush the writer!
pub trait Printable<T: std::io::Write> {
    fn print(&self, writer: &mut BufWriter<T>, depth: usize) -> Result;
}

#[derive(Debug, Clone)]
pub enum Node {
    Empty,
    Element(Element),
}

impl<T: Write> Printable<T> for Node {
    fn print(&self, writer: &mut BufWriter<T>, depth: usize) -> Result {
        match self {
            Self::Element(element) => element.print(writer, depth),
            _ => panic!("Don't use this enum variant!"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    title: Title,
    attributes: Vec<Attribute>,
    nodes: Vec<Node>,
}

impl Element {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            attributes: Vec::<Attribute>::new(),
            nodes: Vec::<Node>::new(),
        }
    }
}

impl<T: Write> Printable<T> for Element {
    fn print(&self, writer: &mut BufWriter<T>, depth: usize) -> Result {
        // write opening
        add_tabs(writer, depth)?;
        writer.write(format!("<{}", self.title).as_bytes())?;

        // write attributes
        for attr in self.attributes.iter() {
            writer.write("\n".as_bytes())?;
            attr.print(writer, depth + 1)?;
        }
        if !self.attributes.is_empty() {
            add_tabs(writer, depth)?;
            writer.write("\n".as_bytes())?;
        }
        if self.nodes.is_empty() {
            writer.write("/>".as_bytes())?;
            return Ok(());
        }
        writer.write(">".as_bytes())?;

        // write elements
        for el in self.nodes.iter() {
            writer.write("\n".as_bytes())?;
            el.print(writer, depth + 1)?;
        }
        if !self.nodes.is_empty() {
            writer.write("\n".as_bytes())?;
        }

        // write closing
        add_tabs(writer, depth)?;
        writer.write(format!("<{0}/>", self.title).as_bytes())?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    title: Title,
    value: Value,
}

impl Attribute {
    pub fn new(title: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            title: title.into(),
            value: value.into(),
        }
    }
}

impl<T: Write> Printable<T> for Attribute {
    fn print(&self, writer: &mut BufWriter<T>, depth: usize) -> Result {
        add_tabs(writer, depth)?;
        writer.write(format!("{}=", self.title).as_bytes())?;
        self.value.print(writer, depth)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Empty,
    String(String),

    // Non px units are converted in the parsing step.
    F32(f32),
    U32(u32),
    I32(i32),
}

impl<T: Write> Printable<T> for Value {
    fn print(&self, writer: &mut BufWriter<T>, depth: usize) -> Result {
        match self {
            Value::String(s) => writer.write(format!("\"{}\"", s).as_bytes())?,
            Value::F32(f) => writer.write(format!("\"{}\"", f).as_bytes())?,
            Value::U32(u) => writer.write(format!("\"{}\"", u).as_bytes())?,
            Value::I32(i) => writer.write(format!("\"{}\"", i).as_bytes())?,
            _ => panic!("Don't use this enum variant!"),
        };
        Ok(())
    }
}

fn add_tabs<T: Write>(writer: &mut BufWriter<T>, depth: usize) -> Result {
    for _ in 0..depth {
        writer.write("\t".as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    pub fn writer_test() {
        use crate::svg::{Attribute, Element, FileDocument, Node, Value};
        let mut doc = FileDocument(Vec::<Node>::new());
        let mut el = Node::Element(Element::new("MyRootElement"));
        if let Node::Element(el) = &mut el {
            el.attributes.push(Attribute::new(
                "MyAttributeName",
                Value::String("MyAttributeValue".to_owned()),
            ));
            el.attributes.push(Attribute::new(
                "MyAttributeName2",
                Value::String("MyAttributeValue2".to_owned()),
            ));
            el.nodes.push(Node::Element(Element::new("MyElementName")));
            el.nodes.push(Node::Element(Element::new("MyElementName2")));
            if let Node::Element(el) = &mut el.nodes.last_mut().unwrap() {
                el.nodes.push(Node::Element(Element::new("MyElementName3")));
            }
        }
        doc.push(el);
        let mut buf = Vec::<u8>::new();
        doc.print(&mut buf).unwrap();
        assert_eq!(
            "<MyRootElement
\tMyAttributeName=\"MyAttributeValue\"
\tMyAttributeName2=\"MyAttributeValue2\"
>
\t<MyElementName/>
\t<MyElementName2>
\t\t<MyElementName3/>
\t<MyElementName2/>
<MyRootElement/>"
                .to_string(),
            String::from_utf8(buf).unwrap()
        );
        //println!("{:?}", doc);
        //doc.output("./out.svg".to_owned()).unwrap();
    }
}
