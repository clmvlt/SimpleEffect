//! Motion template definitions (ADR-0035): reading what a module's `meta` and `params` exports
//! declare ([`parse_definition`], a JSON Schema subset) and carrying item values over to a new
//! version of a template ([`migrate_params`]).
//!
//! The renderer runs the module in its sandbox and hands the two exports over as JSON; everything
//! about their meaning is decided here, so agents get one set of rules and messages.

mod definition;
mod migration;
mod param_schema;

pub use definition::{
    DefinitionProblem, TemplateDefinition, parse_definition, parse_definition_json,
};
pub use migration::{ParamMigration, migrate_params};
