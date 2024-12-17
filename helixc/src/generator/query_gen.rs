use std::fmt::Write;
use std::marker::PhantomData;

// Phantom types to track traversal state
pub struct VertexState;
pub struct EdgeState;
pub struct NoState;

#[derive(Debug, Clone)]
pub enum TraversalStep<In, Out> {
    // Source steps
    V(PhantomData<(In, Out)>),
    E(PhantomData<(In, Out)>),
    AddV {
        label: String,
        _marker: PhantomData<(In, Out)>,
    },
    AddE {
        label: String,
        from_id: String,
        to_id: String,
        _marker: PhantomData<(In, Out)>,
    },

    // Traversal steps
    Out {
        label: String,
        _marker: PhantomData<(In, Out)>,
    },
    OutE {
        label: String,
        _marker: PhantomData<(In, Out)>,
    },
    In {
        label: String,
        _marker: PhantomData<(In, Out)>,
    },
    InE {
        label: String,
        _marker: PhantomData<(In, Out)>,
    },
}

/// ## Traversal Generator
/// Builds up a traversal based on what is in the AST. 
/// 
/// It uses phantom data allow specific traversal transitions to ensure incompatible traversal steps (e.g. `Edge->OutE`) are not allowed.
/// 
pub struct TraversalGenerator<CurrentState> {
    function_identifier: String,
    steps: Vec<Box<dyn TraversalStepGenerator>>,
    _marker: PhantomData<CurrentState>,
}

pub trait TraversalStepGenerator {
    fn generate_code(&self, f: &mut String) -> std::fmt::Result;
}

impl<In, Out> TraversalStepGenerator for TraversalStep<In, Out> {
    fn generate_code(&self, f: &mut String) -> std::fmt::Result {
        match self {
            TraversalStep::V(_) => writeln!(f, "    traversal.v(storage);"),
            TraversalStep::E(_) => writeln!(f, "    traversal.e(storage);"),
            TraversalStep::AddV { label, .. } => {
                writeln!(f, "    traversal.add_v(storage, \"{}\");", label)
            }
            TraversalStep::AddE {
                label,
                from_id,
                to_id,
                ..
            } => {
                writeln!(
                    f,
                    "    traversal.add_e(storage, \"{}\", \"{}\", \"{}\");",
                    label, from_id, to_id
                )
            }
            TraversalStep::Out { label, .. } => {
                writeln!(f, "    traversal.out(storage, \"{}\");", label)
            }
            TraversalStep::OutE { label, .. } => {
                writeln!(f, "    traversal.out_e(storage, \"{}\");", label)
            }
            TraversalStep::In { label, .. } => {
                writeln!(f, "    traversal.in_(storage, \"{}\");", label)
            }
            TraversalStep::InE { label, .. } => {
                writeln!(f, "    traversal.in_e(storage, \"{}\");", label)
            }
        }
    }
}

impl<T> TraversalGenerator<T> {
    pub fn generate_code(&self) -> Result<String, std::fmt::Error> {
        let mut code = String::new();
        
        writeln!(
            code,
            "pub fn {}(storage: &HelixGraphStorage) -> TraversalBuilder {{", self.function_identifier
        )?;
        writeln!(
            code,
            "    let mut traversal = TraversalBuilder::new(vec![]);"
        )?;

        for step in &self.steps {
            step.generate_code(&mut code)?;
        }

        writeln!(code, "    traversal")?;
        writeln!(code, "}}")?;

        Ok(code)
    }
}

impl TraversalGenerator<NoState> {
            
    pub fn new(function_identifier: &str) -> Self {
        Self {
            function_identifier: function_identifier.to_string(),
            steps: Vec::new(),
            _marker: PhantomData,
        }
    }

    // Source steps that start a traversal
    pub fn v(mut self) -> TraversalGenerator<VertexState> {
        self.steps
            .push(Box::new(TraversalStep::<NoState, VertexState>::V(
                PhantomData,
            )));
        TraversalGenerator {
            function_identifier: self.function_identifier,
            steps: self.steps,
            _marker: PhantomData,
        }
    }

    pub fn e(mut self) -> TraversalGenerator<EdgeState> {
        self.steps
            .push(Box::new(TraversalStep::<NoState, EdgeState>::E(
                PhantomData,
            )));
        TraversalGenerator {
            function_identifier: self.function_identifier,
            steps: self.steps,
            _marker: PhantomData,
        }
    }
}

impl TraversalGenerator<VertexState> {
    pub fn out(mut self, label: &str) -> TraversalGenerator<VertexState> {
        self.steps
            .push(Box::new(TraversalStep::<NoState, VertexState>::Out {
                label: label.to_string(),
                _marker: PhantomData,
            }));
        self
    }

    pub fn out_e(mut self, label: &str) -> TraversalGenerator<EdgeState> {
        self.steps
            .push(Box::new(TraversalStep::<NoState, EdgeState>::OutE {
                label: label.to_string(),
                _marker: PhantomData,
            }));
        TraversalGenerator {
            function_identifier: self.function_identifier,
            steps: self.steps,
            _marker: PhantomData,
        }
    }

    pub fn in_(mut self, label: &str) -> TraversalGenerator<VertexState> {
        self.steps
            .push(Box::new(TraversalStep::<NoState, VertexState>::In {
                label: label.to_string(),
                _marker: PhantomData,
            }));
        self
    }

    pub fn in_e(mut self, label: &str) -> TraversalGenerator<EdgeState> {
        self.steps
            .push(Box::new(TraversalStep::<NoState, EdgeState>::InE {
                label: label.to_string(),
                _marker: PhantomData,
            }));
        TraversalGenerator {
            function_identifier: self.function_identifier,
            steps: self.steps,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_safe_traversal() {
        let generator = TraversalGenerator::new("test_function")
            .v()
            .out("knows")
            .in_("follows")
            .out_e("likes");

        let code = generator.generate_code().unwrap();

        assert!(code.contains("pub fn test_function("));
        assert!(code.contains("traversal.v(storage);"));
        assert!(code.contains("traversal.out(storage, \"knows\");"));
        assert!(code.contains("traversal.in_(storage, \"follows\");"));
        assert!(code.contains("traversal.out_e(storage, \"likes\");"));
    }
}
