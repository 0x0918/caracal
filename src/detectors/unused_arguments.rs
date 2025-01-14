use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::core_unit::CoreUnit;
use crate::utils::number_to_ordinal;
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::program::Statement as SierraStatement;

// Note: It doesn't work correctly when arguments are passed by reference

#[derive(Default)]
pub struct UnusedArguments {}

impl Detector for UnusedArguments {
    fn name(&self) -> &str {
        "unused-arguments"
    }

    fn description(&self) -> &str {
        "Detect unused arguments"
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn impact(&self) -> Impact {
        Impact::Low
    }

    fn run(&self, core: &CoreUnit) -> Vec<Result> {
        let mut results = Vec::new();
        let compilation_units = core.get_compilation_units();

        for compilation_unit in compilation_units {
            for f in compilation_unit.functions_user_defined() {
                // Calculate the offset to subtract from the paramter id. Builtins arguments are always before the user defined.
                let offset = f.params_all().count() - f.params().count();

                for stmt in f.get_statements() {
                    if let SierraStatement::Invocation(invoc) = stmt {
                        // Get the concrete libfunc called
                        let libfunc = compilation_unit
                            .registry()
                            .get_libfunc(&invoc.libfunc_id)
                            .expect("Library function not found in the registry");

                        // If an argument is unused there is a Drop as the first instruction
                        // When we don't have any more Drop instructions we are sure the others are used
                        if let CoreConcreteLibfunc::Drop(_) = libfunc {
                            results.push(Result {
                                name: self.name().to_string(),
                                impact: self.impact(),
                                confidence: self.confidence(),
                                message: format!(
                                    "The {} argument in {} is never used",
                                    number_to_ordinal(invoc.args[0].id - offset as u64 + 1),
                                    f.name()
                                ),
                            })
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        results
    }
}
