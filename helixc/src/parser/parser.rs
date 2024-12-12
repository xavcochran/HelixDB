pub trait Parser {
    fn parse(&self, input: &str) -> Result<(), String>;
}
