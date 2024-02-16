pub type Document = Vec<Box<dyn Node>>;
pub type ElementList = Vec<Element>;
pub type AttributeList = Vec<Attribute>;
pub type Title = String;
pub type Value = String;

pub trait Node {
    fn print(&self) -> String;
}

pub struct Element {
    pub title: Title,
    pub r#type: ElementType,
    pub attributes: AttributeList,
    pub elements: ElementList,
}

impl Node for Element {
    fn print(&self) -> String {
        let mut buf = String::new();

        buf += "<";
        buf += &self.title;
        buf += "\n";
        for at in &self.attributes {
            buf += "\t";
            buf += &at.print();
            buf += "\n";
        }

        if let ElementType::Single = self.r#type {
            buf += "/>";
            return buf;
        };

        buf += ">";

        for el in &self.elements {
            buf += "\t";
            buf += &el.print();
            buf += "\n";
        }

        buf += ">";
        buf
    }
}

pub struct Attribute {
    pub title: Title,
    pub value: Value,
}

impl Node for Attribute {
    fn print(&self) -> String {
        let mut buf = String::new();
        buf += &self.title;
        buf += r#"=""#;
        buf += &self.value;
        buf += r#"""#;
        buf
    }
}

pub struct ProcessingInstructions {
    pub content: String,
}

impl Node for ProcessingInstructions {
    fn print(&self) -> String {
        let mut buf = String::new();
        buf += "<?";
        buf += &self.content;
        buf += "?>";
        buf
    }
}

pub struct Comment {
    pub content: String,
}

impl Node for Comment {
    fn print(&self) -> String {
        let mut buf = String::new();
        buf += "<!-- ";
        buf += &self.content;
        buf += " -->";
        buf
    }
}

pub enum ElementType {
    Regular,
    Single,
}
