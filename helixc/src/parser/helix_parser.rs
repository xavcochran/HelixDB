use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HelixParser;

#[derive(Debug)]
pub struct Source {
    pub schemas: Vec<(NodeSchema, EdgeSchema)>,
    pub queries: Vec<Query>,
}

#[derive(Debug)]
pub struct NodeSchema {
    pub name: String,
    pub properties: Vec<Field>,
}

#[derive(Debug)]
pub struct EdgeSchema {
    pub name: String,
    pub from: String,
    pub to: String,
    pub properties: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug)]
pub enum DataType {
    Number,
    String,
    Boolean,
}

#[derive(Debug)]
pub struct Query {
    pub name: String,
    pub body: query_body,
    pub return_type: String,
}

#[derive(Debug)]
pub enum ElementType {
    V,
    E,
}

#[derive(Debug)]
pub struct query_body {
    pub assignment: String,
    pub element_type: ElementType,
    pub traversal: Vec<String>,
}

impl HelixParser {}

impl Parser for HelixParser {
    fn parse(&self, input: &str) -> Result<(), String> {
        // ...
    }
}
