use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
pub struct HelixParser;

#[derive(Parser)]
#[grammar = "grammar.pest"]

impl HelixParser {}

impl Parser for HelixParser {
    fn parse(&self, input: &str) -> Result<(), String> {
        // ...
    }
}
