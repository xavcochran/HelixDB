use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use super::parser_methods::ParserError;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HelixParser;

#[derive(Debug)]
pub struct Source {
    pub node_schemas: Vec<NodeSchema>,
    pub edge_schemas: Vec<EdgeSchema>,
    pub queries: Vec<Query>,
}

//Schema stuff
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

#[derive(Debug, Clone)]
pub enum DataType {
    Number,
    String,
    Boolean,
}

//Query stuff
#[derive(Debug)]
pub struct Query {
    pub name: String,
    pub parameter: String,
    pub body: QueryBody,
    pub return_type: String,
}

#[derive(Debug)]
pub struct QueryBody {
    pub assignment: Option<String>,
    pub element_type: ElementType,
    pub traversal: TraversalStep,
}

#[derive(Debug)]
pub struct TraversalStep {
    pub node: String,
    pub filter: Option<String>,
    pub children: Vec<TraversalStep>,
}

#[derive(Debug, Clone)]
pub enum ElementType {
    V,
    E,
}

impl HelixParser {
    pub fn parse_source(input: &str) -> Result<Source, ParserError> {
        // assert!(false, "string: {:?}", input);

        let pairs = HelixParser::parse(Rule::source, input)
            .expect("unsuccessful parse")
            .next()
            .unwrap();
        let mut source = Source {
            node_schemas: Vec::new(),
            edge_schemas: Vec::new(),
            queries: Vec::new(),
        };
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::node_schema => {
                    source.node_schemas.push(Self::parse_node_schema(pair));
                }
                Rule::edge_schema => {
                    source.edge_schemas.push(Self::parse_edge_schema(pair));
                }
                Rule::query => {
                    source.queries.push(Self::parse_query(pair));
                }
                Rule::EOI => (),
                _ => {
                    // Print out unexpected rule for debugging
                    eprintln!("Unexpected rule: {:?}", pair.as_rule());
                    panic!("Unexpected rule encountered during parsing");
                }
            }
        }

        Ok(source)
    }

    fn parse_node_schema(pair: Pair<Rule>) -> NodeSchema {
        let mut name = String::new();
        let mut properties = Vec::new();
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::identifier => name = inner_pair.as_str().to_string(),
                Rule::schema_properties => {
                    properties = Self::parse_schema_properties(inner_pair);
                }
                _ => {}
            }
        }

        NodeSchema { name, properties }
    }

    fn parse_edge_schema(pair: Pair<Rule>) -> EdgeSchema {
        let mut name = String::new();
        let mut from = String::new();
        let mut to = String::new();
        let mut properties = Vec::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::identifier => name = inner_pair.as_str().to_string(),
                Rule::edge_property => {
                    for prop_pair in inner_pair.into_inner() {
                        match prop_pair.as_rule() {
                            Rule::identifier => {
                                if from.is_empty() {
                                    from = prop_pair.as_str().to_string();
                                } else {
                                    to = prop_pair.as_str().to_string();
                                }
                            }
                            Rule::schema_properties => {
                                properties = Self::parse_schema_properties(prop_pair);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        EdgeSchema {
            name,
            from,
            to,
            properties,
        }
    }

    fn parse_schema_properties(pair: Pair<Rule>) -> Vec<Field> {
        pair.into_inner()
            .filter_map(|prop_pair| {
                if prop_pair.as_rule() == Rule::schema_property {
                    Some(Self::parse_schema_property(prop_pair))
                } else {
                    None
                }
            })
            .collect()
    }

    fn parse_schema_property(pair: Pair<Rule>) -> Field {
        let mut name = String::new();
        let mut data_type = DataType::String; // default

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::identifier => name = inner_pair.as_str().to_string(),
                Rule::type_def => {
                    data_type = match inner_pair.as_str() {
                        "Number" => DataType::Number,
                        "Boolean" => DataType::Boolean,
                        _ => DataType::String,
                    };
                }
                _ => {}
            }
        }

        Field { name, data_type }
    }

    fn parse_query(pair: Pair<Rule>) -> Query {
        let mut name = String::new();
        let mut parameter = String::new();
        let mut body = QueryBody {
            assignment: None,
            element_type: ElementType::V,
            traversal: TraversalStep {
                node: String::new(),
                filter: None,
                children: Vec::new(),
            },
        };
        let mut return_type = String::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::identifier => name = inner_pair.as_str().to_string(),
                Rule::parameters => {
                    parameter = inner_pair
                        .into_inner()
                        .filter_map(|param_pair| {
                            if param_pair.as_rule() == Rule::parameter {
                                Some(Self::parse_parameter(param_pair))
                            } else {
                                None
                            }
                        })
                        .next()
                        .map(|field| field.name)
                        .unwrap_or_default();
                }
                Rule::query_body => body = Self::parse_query_body(inner_pair),
                Rule::return_clause => {
                    return_type = inner_pair
                        .into_inner()
                        .filter_map(|item_pair| {
                            if item_pair.as_rule() == Rule::identifier {
                                Some(item_pair.as_str().to_string())
                            } else {
                                None
                            }
                        })
                        .next()
                        .unwrap_or_default();
                }
                _ => {}
            }
        }

        Query {
            name,
            parameter,
            body,
            return_type,
        }
    }

    fn parse_parameter(pair: Pair<Rule>) -> Field {
        let mut name = String::new();
        let mut data_type = DataType::String; // default

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::identifier => name = inner_pair.as_str().to_string(),
                Rule::type_def => {
                    data_type = match inner_pair.as_str() {
                        "Number" => DataType::Number,
                        "Boolean" => DataType::Boolean,
                        _ => DataType::String,
                    };
                }
                _ => {}
            }
        }

        Field { name, data_type }
    }

    fn parse_query_body(pair: Pair<Rule>) -> QueryBody {
        let mut assignment = None;
        let mut element_type = ElementType::V;
        let mut traversal = TraversalStep {
            node: String::new(),
            filter: None,
            children: Vec::new(),
        };

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::get_clause => {
                    for get_inner in inner_pair.into_inner() {
                        match get_inner.as_rule() {
                            Rule::traversal_assignment => {
                                assignment = Some(
                                    get_inner.into_inner().next().unwrap().as_str().to_string(),
                                );
                            }
                            Rule::source_traversal => {
                                element_type =
                                    match get_inner.clone().into_inner().next().unwrap().as_str() {
                                        "V" => ElementType::V,
                                        "E" => ElementType::E,
                                        _ => ElementType::V,
                                    };
                                traversal = Self::parse_traversal_expression(get_inner);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        QueryBody {
            assignment,
            element_type,
            traversal,
        }
    }

    fn parse_traversal_expression(pair: Pair<Rule>) -> TraversalStep {
        let mut current_step = TraversalStep {
            node: String::new(),
            filter: None,
            children: Vec::new(),
        };

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::identifier => {
                    if current_step.node.is_empty() {
                        current_step.node = inner_pair.as_str().to_string();
                    }
                }
                Rule::child_expression => {
                    let child_node = inner_pair.into_inner().next().unwrap().as_str().to_string();
                    current_step.children.push(TraversalStep {
                        node: child_node,
                        filter: None,
                        children: Vec::new(),
                    });
                }
                _ => {}
            }
        }

        current_step
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::helix_parser::HelixParser;

    #[test]
    fn test_input() {
        let input = r#"
    V::Person {
        name: String,
        age: Number
    }
    
    E::Knows {
        From: Person,
        To: Person,
        Properties {
            since: Number
        }
    }
    
    QUERY findFriends => 
        GET V::Person 
        RETURN name
    "#;

        let parsed = HelixParser::parse_source(input);

        // Output results
        match parsed {
            Ok(source) => {
                println!("Parsed Source:");

                println!("Node Schemas:");
                for schema in &source.node_schemas {
                    println!("  Name: {}", schema.name);
                    println!("  Properties:");
                    for prop in &schema.properties {
                        println!("    - {}: {:?}", prop.name, prop.data_type);
                    }
                }

                println!("\nEdge Schemas:");
                for schema in &source.edge_schemas {
                    println!("  Name: {}", schema.name);
                    println!("  From: {}", schema.from);
                    println!("  To: {}", schema.to);
                    println!("  Properties:");
                    for prop in &schema.properties {
                        println!("    - {}: {:?}", prop.name, prop.data_type);
                    }
                }

                println!("\nQueries:");
                for query in &source.queries {
                    println!("  Name: {}", query.name);
                    println!("  Parameter: {}", query.parameter);
                    println!("  Return Type: {}", query.return_type);
                    println!("  Body:");
                    println!("    Assignment: {:?}", query.body.assignment);
                    println!("    Element Type: {:?}", query.body.element_type);
                    println!("    Traversal:");
                    println!("      Node: {}", query.body.traversal.node);
                    println!("      Filter: {:?}", query.body.traversal.filter);
                }

                // Standard assertions can remain
                assert_eq!(source.node_schemas.len(), 1);
                assert_eq!(source.node_schemas[0].name, "Person");
                assert_eq!(source.node_schemas[0].properties.len(), 2);

                assert_eq!(source.edge_schemas.len(), 1);
                assert_eq!(source.edge_schemas[0].name, "Knows");

                assert_eq!(source.queries.len(), 1);
                assert_eq!(source.queries[0].name, "findFriends");
            }
            Err(e) => {
                panic!("Parsing failed: {:?}", e);
            }
        }
    }
}
