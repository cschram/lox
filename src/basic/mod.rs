mod builtins;
mod environment;
mod error;
mod expr;
mod parser;
mod resolver;
mod scanner;
mod state;
mod stmt;
mod value;

pub use self::error::*;
use self::{environment::*, parser::*, resolver::*, state::LoxState};
use log::error;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

pub struct Lox;

impl Lox {
    pub fn new() -> Self {
        Self{}
    }
    
    pub fn exec(&mut self, source: &str) -> LoxResult {
        let ParseResult {
            statements,
            errors: parse_errors,
        } = parse(source);
        if !parse_errors.is_empty() {
            for err in parse_errors.iter() {
                error!("Parse Error: {}", err.to_string());
            }
            return Err(LoxError::Runtime("Syntax errors encountered".into()));
        }
        let mut locals: Locals = HashMap::new();
        for (key, value) in Resolver::bind(&statements)?.drain() {
            locals.insert(key, value);
        }
        let mut state = LoxState::new(locals);
        for stmt in statements.iter() {
            stmt.eval(&mut state, GLOBAL_SCOPE)?;
        }
        Ok(())
    }

    pub fn exec_file(&mut self, path: &str) -> LoxResult {
        let file = File::open(path)?;
        let source: String = BufReader::new(file)
            .lines()
            .flat_map(|l| {
                let mut line = l.unwrap().chars().collect::<Vec<char>>();
                line.push('\n');
                line
            })
            .collect();
        self.exec(&source)
    }
}

#[cfg(test)]
mod test {
    use super::super::test_scripts::*;
    use super::*;
    use mock_logger::MockLogger;

    #[test]
    fn print() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(PRINT_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "3.14");
            assert_eq!(entries[1].body, "nil");
        });
        Ok(())
    }

    #[test]
    fn block_scope() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(BLOCK_SCOPE_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "foo");
            assert_eq!(entries[1].body, "bar");
        });
        Ok(())
    }

    #[test]
    fn control_flow() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(CONTROL_FLOW_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "true");
            assert_eq!(entries[1].body, "true");
        });
        Ok(())
    }

    #[test]
    fn while_loop() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(WHILE_LOOP_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 4);
            assert_eq!(entries[0].body, "4");
            assert_eq!(entries[1].body, "3");
            assert_eq!(entries[2].body, "2");
            assert_eq!(entries[3].body, "1");
        });
        Ok(())
    }

    #[test]
    fn for_loop() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(FOR_LOOP_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 5);
            assert_eq!(entries[0].body, "0");
            assert_eq!(entries[1].body, "1");
            assert_eq!(entries[2].body, "2");
            assert_eq!(entries[3].body, "3");
            assert_eq!(entries[4].body, "42");
        });
        Ok(())
    }

    #[test]
    fn builtins() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(BUILTINS_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 1);
            assert_ne!(entries[0].body, "nil");
        });
        Ok(())
    }

    #[test]
    fn function() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(FUNCTION_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].body, "Hello, world!");
        });
        Ok(())
    }

    #[test]
    fn function_closure() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(FUNCTION_CLOSURE_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "1");
            assert_eq!(entries[1].body, "2")
        });
        Ok(())
    }

    #[test]
    fn shadowing() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(SHADOWING_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "global");
            assert_eq!(entries[1].body, "global")
        });
        Ok(())
    }

    #[test]
    fn class() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(CLASS_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].body, "Hello, world!");
        });
        Ok(())
    }
}
