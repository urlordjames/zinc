use crate::node::{AbstractType, Node, Statement, FunctionInfo, Definition, FileDescription};
use std::collections::HashMap;

use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "zinc.pest"]
struct ZincParser;

fn nodeify(pair: pest::iterators::Pair<Rule>) -> Node {
	match pair.as_rule() {
		Rule::expr => {
			nodeify(pair.into_inner().next().unwrap())
		},
		Rule::var_declaration => {
			let mut inner = pair.into_inner();
			let mut declaration = inner.next().unwrap().into_inner();

			Node::Set {
				name: String::from(declaration.next().unwrap().as_str()),
				var_type: to_abstract_type(declaration.next().unwrap().as_str()),
				value: Box::new(nodeify(inner.next().unwrap()))
			}
		},
		Rule::identifier => {
			let name = String::from(pair.as_str());

			Node::Get {
				name
			}
		},
		Rule::operand => {
			nodeify(pair.into_inner().next().unwrap())
		},
		Rule::number => {
			Node::Int(
				pair.as_str().parse().expect("int out of bounds")
			)
		},
		Rule::boolean => {
			Node::Bool(
				match pair.as_str() {
					"true" => true,
					"false" => false,
					_ => unreachable!("boolean should be true or false")
				}
			)
		},
		Rule::string_literal => {
			let slice = pair.as_str();
			Node::StringLiteral(String::from(&slice[1..slice.len() - 1]))
		},
		Rule::binary_expr => {
			let mut values = pair.into_inner();
			let mut first_val = nodeify(values.next().unwrap());

			while let Some(operator) = values.next() {
				let lhs = Box::new(first_val);
				let rhs = Box::new(nodeify(values.next().unwrap()));

				first_val = match operator.as_str() {
					"+" => Node::Add { lhs, rhs },
					"-" => Node::Subtract { lhs, rhs },
					"*" => Node::Multiply { lhs, rhs },
					"/" => Node::Divide { lhs, rhs },
					"==" => Node::Equal { lhs, rhs },
					"!=" => Node::NotEqual { lhs, rhs },
					"=?" => Node::BoolEqual { lhs, rhs },
					"!?" => Node::BoolNotEqual { lhs, rhs },
					"<=" => Node::LessThanOrEqual { lhs, rhs },
					">=" => Node::GreaterThanOrEqual { lhs, rhs },
					"<" => Node::LessThan { lhs, rhs },
					">" => Node::GreaterThan { lhs, rhs },
					_ => unreachable!("nonexistent operator")
				};
			}

			return first_val;
		},
		Rule::function_expr => {
			let mut inner = pair.into_inner();
			let name = inner.next().unwrap();
			let args = inner.next();

			Node::Function {
				name: String::from(name.as_str()),
				args: match args {
					Some(args) => {
						args.into_inner().map(|arg| nodeify(arg)).collect()
					},
					None => vec![]
				}
			}
		},
		_ => unreachable!()
	}
}

fn to_abstract_type(type_str: &str) -> AbstractType {
	match type_str {
		"i32" => AbstractType::Integer,
		"bool" => AbstractType::Boolean,
		"str" => AbstractType::String,
		"void" => AbstractType::Void,
		_ => panic!("nonexistent type")
	}
}

fn to_statement(pair: pest::iterators::Pair<Rule>) -> Statement {
	match pair.as_rule() {
		Rule::expr | Rule::var_declaration => Statement::Node(nodeify(pair)),
		Rule::return_statement => Statement::Return(nodeify(pair.into_inner().next().unwrap())),
		Rule::if_statement => {
			let mut inner = pair.into_inner();
			let condition = inner.next().unwrap();

			let branch = inner.next().unwrap().into_inner();
			let branch_statements: Vec<Statement> = branch.map(|pair| {
				to_statement(pair.into_inner().next().unwrap())
			}).collect();

			let mut else_branch_statements = vec![];

			if let Some(else_branch) = inner.next() {
				let else_branch = else_branch.into_inner();

				else_branch.for_each(|pair| {
					let statement = to_statement(pair.into_inner().next().unwrap());
					else_branch_statements.push(statement);
				});
			}

			Statement::If {
				condition: nodeify(condition),
				branch: branch_statements,
				else_branch: else_branch_statements
			}
		},
		Rule::while_loop => {
			let mut inner = pair.into_inner();
			let condition = nodeify(inner.next().unwrap());
			let loop_statements = inner.next().unwrap().into_inner().map(|pair| {
				to_statement(pair.into_inner().next().unwrap())
			}).collect();

			Statement::While {
				condition,
				loop_statements
			}
		},
		Rule::infinite_loop => {
			let loop_statements = pair.into_inner().next().unwrap();
			Statement::InfiniteLoop(loop_statements.into_inner().map(|pair| {
				to_statement(pair.into_inner().next().unwrap())
			}).collect())
		},
		_ => unreachable!()
	}
}

pub fn parse(code: &str) -> Result<FileDescription, pest::error::Error<Rule>> {
	let file = ZincParser::parse(Rule::file, &code)?.next().unwrap();

	let mut statements: Vec<Statement> = vec![];
	let mut functions: HashMap<String, FunctionInfo> = HashMap::new();

	file.into_inner().filter(|pair| {
		pair.as_rule() != Rule::EOI
	}).for_each(|pair| {
		match pair.as_rule() {
			Rule::line => {
				let inner_line = pair.into_inner().next().unwrap();

				statements.push(to_statement(inner_line));
			},
			Rule::func_declaration => {
				let mut function = pair.into_inner();
				let mut signature = function.next().unwrap().into_inner();
				let return_type = function.next().unwrap();
				let lines: Vec<Statement> = function.next().unwrap().into_inner().map(|line| {
					to_statement(line.into_inner().next().unwrap())
				}).collect();

				let function_name = signature.next().unwrap();

				let args: Vec<Definition> = match signature.next() {
					Some(arg_list) => {
						arg_list.into_inner()
						.map(|arg| arg.into_inner())
						.map(|mut arg| {
							Definition {
								name: String::from(arg.next().unwrap().as_str()),
								data_type: to_abstract_type(arg.next().unwrap().as_str())
							}
						}).collect()
					},
					None => vec![]
				};

				functions.insert(String::from(function_name.as_str()), FunctionInfo {
					body: lines,
					args,
					return_type: to_abstract_type(return_type.as_str())
				});
			}
			_ => unreachable!()
		}
	});

	Ok(FileDescription {
		statements,
		functions
	})
}
