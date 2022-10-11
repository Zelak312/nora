use std::rc::Rc;

use crate::{
    ast::{
        NodeBinaryOperator, NodeBlock, NodeCondition, NodeContent, NodeIdentifer, NodeNumber,
        NodeString, NodeTernary,
    },
    base_parser::BaseParser,
    errors::{BasicError, Error},
    node::ExecutableNode,
    token::{Token, Type},
};

pub struct Parser {
    base_parser: BaseParser,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            base_parser: BaseParser::new(tokens),
        }
    }

    pub fn content_all(&mut self, start: &str) -> String {
        let mut content = String::from(start);
        while let Some(token) = self.base_parser.chain_reader.get_current() {
            if token.r#type == Type::BlockStart {
                break;
            }

            content += &token.raw;
            self.base_parser.chain_reader.advance();
        }

        content
    }

    pub fn parse(&mut self) -> Result<Rc<dyn ExecutableNode>, Box<dyn Error>> {
        let token = self.base_parser.any()?;
        match token.r#type {
            Type::BlockStart => {
                let content = self.parse_inner_block()?;
                let mut block_node = NodeBlock {
                    content,
                    next: None,
                };
                if let Ok(node) = self.parse() {
                    block_node.next = Some(node);
                }

                Ok(Rc::new(block_node))
            }
            _ => {
                let content = self.content_all(&token.raw);
                let mut content_node = NodeContent {
                    content,
                    next: None,
                };
                if let Ok(node) = self.parse() {
                    content_node.next = Some(node);
                }

                Ok(Rc::new(content_node))
            }
        }
    }

    pub fn parse_identifier(&mut self) -> Result<Rc<dyn ExecutableNode>, Box<dyn Error>> {
        let token = self.base_parser.expect(Type::Identifier)?;
        Ok(Rc::new(NodeIdentifer { content: token.raw }))
    }

    pub fn parse_string(&mut self) -> Result<Rc<dyn ExecutableNode>, Box<dyn Error>> {
        let token = self.base_parser.expect(Type::String)?;
        Ok(Rc::new(NodeString { content: token.raw }))
    }

    pub fn parse_number(&mut self) -> Result<Rc<dyn ExecutableNode>, Box<dyn Error>> {
        // check if it's an unary
        let unary = self
            .base_parser
            .expect_m(vec![Type::Addition, Type::Subtraction]);
        let token = self.base_parser.expect(Type::Number)?;
        let content = (unary.map_or("".to_owned(), |t| t.raw).to_owned() + &token.raw)
            .parse::<f64>()
            .map_err(|_| BasicError::new("ss".to_owned()))?;
        Ok(Rc::new(NodeNumber { content }))
    }

    pub fn parse_basic_type(&mut self) -> Result<Rc<dyn ExecutableNode>, Box<dyn Error>> {
        let identifer = self.parse_identifier();
        if identifer.is_ok() {
            return identifer;
        }

        let string = self.parse_string();
        if string.is_ok() {
            return string;
        }

        self.parse_number()
    }

    pub fn parse_binary_operation(&mut self) -> Result<Rc<dyn ExecutableNode>, Box<dyn Error>> {
        let mut left = self.parse_binary_pow_div()?;
        while let Ok(operator) = self
            .base_parser
            .expect_m(vec![Type::Addition, Type::Subtraction])
        {
            let right = self.parse_binary_pow_div()?;
            left = Rc::new(NodeBinaryOperator {
                operator: operator.r#type,
                left,
                right,
            });
        }

        Ok(left)
    }

    pub fn parse_binary_pow_div(&mut self) -> Result<Rc<dyn ExecutableNode>, Box<dyn Error>> {
        let mut left = self.parse_binary_parenthese()?;
        while let Ok(operator) = self
            .base_parser
            .expect_m(vec![Type::Multiplication, Type::Division])
        {
            let right = self.parse_binary_parenthese()?;
            left = Rc::new(NodeBinaryOperator {
                operator: operator.r#type,
                left,
                right,
            });
        }

        Ok(left)
    }

    pub fn parse_binary_parenthese(&mut self) -> Result<Rc<dyn ExecutableNode>, Box<dyn Error>> {
        if let Ok(_) = self.base_parser.expect(Type::ParentL) {
            let math = self.parse_ternary()?;
            self.base_parser.expect(Type::ParentR)?;
            return Ok(math);
        }

        self.parse_basic_type()
    }

    pub fn parse_inner_block(&mut self) -> Result<Rc<dyn ExecutableNode>, Box<dyn Error>> {
        let node = self.parse_ternary()?;
        self.base_parser.expect(Type::BlockEnd)?;
        Ok(node)
    }

    pub fn parse_ternary(&mut self) -> Result<Rc<dyn ExecutableNode>, Box<dyn Error>> {
        let condition = self.parse_condition()?;
        if self.base_parser.expect(Type::QuestionMark).is_ok() {
            let left = self.parse_ternary()?;
            self.base_parser.expect(Type::Semicolon)?;
            let right = self.parse_ternary()?;

            return Ok(Rc::new(NodeTernary {
                condition,
                left,
                right,
            }));
        }

        Ok(condition)
    }

    pub fn parse_condition(&mut self) -> Result<Rc<dyn ExecutableNode>, Box<dyn Error>> {
        let mut left = self.parse_binary_operation()?;
        while let Ok(operator) = self.base_parser.expect_m(vec![
            Type::LessThanSign,
            Type::LessThanEqualSign,
            Type::GreaterThanSign,
            Type::GreaterThanEqualSign,
            Type::DoubleEqualSign,
        ]) {
            let right = self.parse_binary_operation()?;
            left = Rc::new(NodeCondition {
                operator: operator.r#type,
                left,
                right,
            });
        }

        Ok(left)
    }
}
