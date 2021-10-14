use crate::lang::Expression;
use crate::tactics::TacticsError;
use crate::lang::TacticsExpression;
use crate::context::Context;
use crate::error::Result;
use crate::lang::ParameterDefinition;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser, Token};
use crate::tactics::TacticsResult;
use core::fmt::Formatter;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct TacticsDict(Arc<HashMap<String, TacticsDefinition>>);

impl Display for TacticsDict {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        for t in self.0.values() {
            t.format(fmt, db)?;
        }
        Ok(())
    }
}

impl From<Vec<TacticsDefinition>> for TacticsDict {
    fn from(tactics_list: Vec<TacticsDefinition>) -> Self {
        let mut tactics_definitions = HashMap::new();
        for tactics_definition in tactics_list {
            tactics_definitions.insert(
                tactics_definition.name.as_str().to_string(),
                tactics_definition,
            );
        }
        Self(Arc::new(tactics_definitions))
    }
}

impl TacticsDict {
    pub fn get(&self, name: String) -> Option<&TacticsDefinition> {
        self.0.get(&name)
    }
}

pub struct TacticsDefinition {
    pub name: String,
    pub description: String,
    pub parameter_definition: Vec<ParameterDefinition>,
    tactics: TacticsExpression,
}

impl Display for TacticsDefinition {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("/**")?;
        fmt.write_str(&self.description)?;
        fmt.write_str("*/")?;
        fmt.write_str("tactics ")?;
        fmt.write_str(&self.name)?;
        self.tactics.format(fmt, db)?;
        Ok(())
    }
}

impl Parse for TacticsDefinition {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let name = parser.parse_identifier()?;
        let description = parser
            .last_description()
            .unwrap_or_else(|| "* no description provided *".to_string());
        let mut parameter_definition = Vec::new();
        parser.parse_token(Token::ParensOpen)?;
        while let Some(parameter) = parser.parse_parameter_definition()? {
            parameter_definition.push(parameter);
        }
        // parse_parameter_definition returns None when it encounters the closing parens
        let tactics = parser.parse_tactics()?;
        Ok(TacticsDefinition {
            name,
            description,
            parameter_definition,
            tactics,
        })
    }
}

impl TacticsDefinition {
    //	fn get_name(&self) -> String {
    //		self.name.clone()
    //	}
    //
    //	//fn arg_types(&self) ->
    //	fn get_desc(&self) -> String {
    //		self.description.clone()
    //	}
    //
    pub fn execute(&self, context: &mut Context) -> TacticsResult {
        self.tactics.execute(context)
    }

    pub fn add_variables(&self, context: &mut Context, parameters: &Vec<Expression>) -> TacticsResult<()> {
        for (param, def) in parameters.iter().zip(self.parameter_definition.iter()) {
            // Set parameters as variables in the context...
            match (param, def) {
                (Expression::Tactics(t), ParameterDefinition::Tactics(id)) => { context.add_tactics_variable(id.to_string(), t.evaluate(context)?); },
                (Expression::Statement(l), ParameterDefinition::Theorem(id)) => { context.add_label_variable(id.to_string(), l.evaluate(context)?); },
                _ => Err(TacticsError::Error)?
            }
        }
        Ok(())
    }
}
