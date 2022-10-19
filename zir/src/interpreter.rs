use crate::node::{FileDescription, Statement, Node, AbstractType, FunctionInfo};
use std::collections::HashMap;

#[derive(Debug)]
pub enum RuntimeResult {
	TypeError(&'static str),
	IncorrectArgumentNumber,
	UndefinedVariable(String),
	UndefinedFunction(String),
	AdditionOverflow,
	SubtractionOverflow,
	MultiplicationOverflow,
	DivisionError,
	Panic
}

pub fn interpret(file_description: FileDescription) -> Result<String, RuntimeResult> {
	let interpreter_state = InterpreterState {
		file_description
	};

	Ok(interpreter_state.run_main()?)
}

#[derive(Debug, Clone)]
pub enum Value {
	Integer(i32),
	Boolean(bool),
	String(String),
	None
}

impl Value {
	pub fn to_abstract(&self) -> AbstractType {
		match self {
			Value::Integer(_) => AbstractType::Integer,
			Value::Boolean(_) => AbstractType::Boolean,
			Value::String(_) => AbstractType::String,
			Value::None => AbstractType::Void
		}
	}
}

struct InterpreterState {
	file_description: FileDescription
}

impl InterpreterState {
	fn run_main(self) -> Result<String, RuntimeResult> {
		let mut output_string = String::new();

		let mut main_function = FunctionState {
			info: &FunctionInfo {
				body: self.file_description.statements,
				args: vec![],
				return_type: AbstractType::Void
			},
			variables: HashMap::new(),
			functions: &self.file_description.functions,
			output_string: &mut output_string
		};

		main_function.run(vec![])?;
		Ok(output_string)
	}
}

#[test]
fn test_interpreter() {
	let file_description = crate::parse::parse(r#"
		assert_int_eq(2 + 3, 5);
		assert_int_eq(3 - 2, 1);
		assert_int_eq(5 * 5, 25);
		assert_int_eq(9 / 3, 3);

		assert_bool_eq(1 + 1 == 2, true);
		assert_bool_eq(3 * 5 != 15, false);

		if (false) {
			panic();
		} else {
			assert_bool_eq(true !? false, true);
		}

		if (true) {
			assert_bool_eq(true =? true, true);
		} else {
			panic();
		}

		assert_bool_eq(3 <= 3, true);
		assert_bool_eq(3 >= 3, true);
		assert_bool_eq(3 < 3, false);
		assert_bool_eq(3 > 3, false);
		assert_bool_eq(2 > 1, true);
		assert_bool_eq(1 < 2, true);

		let x: i32 = 3 * 3 + 1;
		assert_int_eq(x, 10);

		let y: bool = x == 10;
		assert_bool_eq(y, true);

		let n: i32 = 50;
		while (n > 0) {
			let n: i32 = n - 1;
		}
		assert_int_eq(n, 0);

		let s: str = "bruh";
		assert_int_eq(str_len(s), 4);

		fn square(n: i32) -> i32 {
			return n * n;
		}
		assert_int_eq(square(5), 25);

		fn cube(n: i32) -> i32 {
			return square(n) * n;
		}
		assert_int_eq(cube(5), 125);

		assert_str_eq(str_concat("br", "uh"), "bruh");

		print_int(60 + 9);
	"#).expect("failed to parse");

	assert_eq!(interpret(file_description).expect("no runtime failures"), "69\n");
}

struct FunctionState<'a> {
	info: &'a FunctionInfo,
	variables: HashMap<String, Value>,
	functions: &'a HashMap<String, FunctionInfo>,
	output_string: &'a mut String
}

impl<'a> FunctionState<'a> {
	fn run(&mut self, arguments: Vec<Value>) -> Result<Value, RuntimeResult> {
		if arguments.len() != self.info.args.len() {
			return Err(RuntimeResult::IncorrectArgumentNumber)
		}

		for (arg, def) in arguments.iter().zip(self.info.args.iter()) {
			if arg.to_abstract() != def.data_type {
				return Err(RuntimeResult::TypeError("user-defined function called with incorrect arguments"))
			}
			self.variables.insert(def.name.clone(), arg.clone());
		}

		match self.eval_statements(&self.info.body)? {
			Some(val) => Ok(val),
			None => Ok(Value::None)
		}
	}

	fn eval_statements(&mut self, statements: &Vec<Statement>) -> Result<Option<Value>, RuntimeResult> {
		for statement in statements {
			match self.eval_statement(statement)? {
				Some(returned_val) => {
					return Ok(Some(returned_val));
				}
				_ => ()
			}
		}

		return Ok(None);
	}

	fn eval_statement(&mut self, statement: &Statement) -> Result<Option<Value>, RuntimeResult> {
		match statement {
			Statement::Node(node) => {
				self.eval_node(node)?;
				Ok(None)
			},
			Statement::Return(node) => {
				Ok(Some(self.eval_node(node)?))
			},
			Statement::If { condition, branch, else_branch } => {
				match self.eval_node(condition)? {
					Value::Boolean(val) => match val {
						true => Ok(self.eval_statements(branch)?),
						false => Ok(self.eval_statements(else_branch)?)
					},
					_ => Err(RuntimeResult::TypeError("if condition must be boolean"))
				}
			},
			Statement::While { condition, loop_statements } => {
				while match self.eval_node(condition)? {
					Value::Boolean(val) => val,
					_ => return Err(RuntimeResult::TypeError("while condition must be boolean"))
				} {
					match self.eval_statements(loop_statements)? {
						Some(val) => {
							return Ok(Some(val));
						},
						None => ()
					};
				}

				return Ok(None);
			},
			Statement::InfiniteLoop(statements) => {
				loop {
					match self.eval_statements(statements)? {
						Some(val) => {
							return Ok(Some(val));
						},
						None => ()
					};
				}
			}
		}
	}

	fn eval_node(&mut self, node: &Node) -> Result<Value, RuntimeResult> {
		match node {
			Node::Int(val) => Ok(Value::Integer(*val)),
			Node::Add { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Integer(l), Value::Integer(r)) => {
						match l.checked_add(r) {
							Some(res) => Ok(Value::Integer(res)),
							None => Err(RuntimeResult::AdditionOverflow)
						}
					},
					_ => Err(RuntimeResult::TypeError("cannot add non-integers"))
				}
			},
			Node::Subtract { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Integer(l), Value::Integer(r)) => {
						match l.checked_sub(r) {
							Some(res) => Ok(Value::Integer(res)),
							None => Err(RuntimeResult::SubtractionOverflow)
						}
					},
					_ => Err(RuntimeResult::TypeError("cannot subtract non-integers"))
				}
			},
			Node::Multiply { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Integer(l), Value::Integer(r)) => {
						match l.checked_mul(r) {
							Some(res) => Ok(Value::Integer(res)),
							None => Err(RuntimeResult::MultiplicationOverflow)
						}
					},
					_ => Err(RuntimeResult::TypeError("cannot multiply non-integers"))
				}
			},
			Node::Divide { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Integer(l), Value::Integer(r)) => {
						match l.checked_div(r) {
							Some(res) => Ok(Value::Integer(res)),
							None => Err(RuntimeResult::DivisionError)
						}
					},
					_ => Err(RuntimeResult::TypeError("cannot divide non-integers"))
				}
			},
			Node::Equal { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l == r)),
					_ => Err(RuntimeResult::TypeError("cannot check equality of non-integers"))
				}
			},
			Node::NotEqual { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l != r)),
					_ => Err(RuntimeResult::TypeError("cannot check non-equality of non-integers"))
				}
			},
			Node::LessThanOrEqual { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
					_ => Err(RuntimeResult::TypeError("cannot check less than or equal of non-integers"))
				}
			},
			Node::GreaterThanOrEqual { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
					_ => Err(RuntimeResult::TypeError("cannot check greater than or equal of non-integers"))
				}
			},
			Node::LessThan { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
					_ => Err(RuntimeResult::TypeError("cannot check less than of non-integers"))
				}
			},
			Node::GreaterThan { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
					_ => Err(RuntimeResult::TypeError("cannot check greater than of non-integers"))
				}
			},
			Node::Bool(val) => Ok(Value::Boolean(*val)),
			Node::BoolEqual { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l == r)),
					_ => Err(RuntimeResult::TypeError("cannot check equality of non-booleans"))
				}
			},
			Node::BoolNotEqual { lhs, rhs } => {
				let lhv = self.eval_node(lhs)?;
				let rhv = self.eval_node(rhs)?;

				match (lhv, rhv) {
					(Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l != r)),
					_ => Err(RuntimeResult::TypeError("cannot check non-equality of non-booleans"))
				}
			},
			Node::StringLiteral(val) => Ok(Value::String(val.to_string())),
			Node::Set { name, var_type, value } => {
				let value = self.eval_node(value)?;
				if value.to_abstract() != *var_type {
					return Err(RuntimeResult::TypeError("value must be same type as variable is declared"))
				}

				self.variables.insert(name.to_string(), value.clone()); // clone = bad
				Ok(value) // for compatability with native/jit
			},
			Node::Get { name } => {
				match self.variables.get(name) {
					Some(val) => Ok(val.clone()), // clone = bad
					None => Err(RuntimeResult::UndefinedVariable(name.to_string()))
				}
			},
			Node::Function { name, args } => {
				let mut evaluated_args: Vec<Value> = vec![];
				for arg in args {
					evaluated_args.push(self.eval_node(arg)?)
				}

				match try_std_function(name, &evaluated_args, &mut self.output_string)? {
					Some(value) => Ok(value),
					None => {
						let mut function_state = FunctionState {
							info: match self.functions.get(name) {
								Some(info) => info,
								None => return Err(RuntimeResult::UndefinedFunction(name.to_string()))
							},
							variables: HashMap::new(),
							functions: self.functions,
							output_string: self.output_string
						};

						Ok(function_state.run(evaluated_args)?)
					}
				}
			},
		}
	}
}

fn try_std_function(name: &str, args: &Vec<Value>, output_string: &mut String) -> Result<Option<Value>, RuntimeResult> {
	let result = match name {
		"print_int" => {
			match &args[0] {
				Value::Integer(val) => {
					output_string.push_str(&format!("{}\n", val));
					Ok(Some(Value::None))
				},
				_ => Err(RuntimeResult::TypeError("print_int only prints integers"))
			}
		},
		"print_bool" => {
			match &args[0] {
				Value::Boolean(val) => {
					output_string.push_str(&format!("{}\n", val));
					Ok(Some(Value::None))
				},
				_ => Err(RuntimeResult::TypeError("print_bool only prints booleans"))
			}
		},
		"print_str" => {
			match &args[0] {
				Value::String(val) => {
					output_string.push_str(&val);
					output_string.push('\n');
					Ok(Some(Value::None))
				},
				_ => Err(RuntimeResult::TypeError("print_str only prints strings"))
			}
		},
		"str_eq" => {
			match (&args[0], &args[1]) {
				(Value::String(l), Value::String(r)) => {
					Ok(Some(Value::Boolean(l == r)))
				},
				_ => Err(RuntimeResult::TypeError("str_eq only compares strings"))
			}
		},
		"str_len" => {
			match &args[0] {
				Value::String(val) => {
					Ok(Some(Value::Integer(val.len() as i32)))
				},
				_ => Err(RuntimeResult::TypeError("str_len only takes a string"))
			}
		},
		"str_concat" => {
			match (&args[0], &args[1]) {
				(Value::String(l), Value::String(r)) => {
					Ok(Some(Value::String(l.to_string() + r)))
				},
				_ => Err(RuntimeResult::TypeError("str_concat can only concatenate strings"))
			}
		},
		"assert_int_eq" => {
			match (&args[0], &args[1]) {
				(Value::Integer(l), Value::Integer(r)) => {
					if l != r {
						return Err(RuntimeResult::Panic);
					}

					Ok(Some(Value::None))
				},
				_ => Err(RuntimeResult::TypeError("assert_int_eq only compares integers"))
			}
		},
		"assert_bool_eq" => {
			match (&args[0], &args[1]) {
				(Value::Boolean(l), Value::Boolean(r)) => {
					if l != r {
						return Err(RuntimeResult::Panic);
					}

					Ok(Some(Value::None))
				},
				_ => Err(RuntimeResult::TypeError("assert_bool_eq only compares booleans"))
			}
		},
		"assert_str_eq" => {
			match (&args[0], &args[1]) {
				(Value::String(l), Value::String(r)) => {
					if l != r {
						return Err(RuntimeResult::Panic);
					}

					Ok(Some(Value::None))
				},
				_ => Err(RuntimeResult::TypeError("assert_str_eq only compares strings"))
			}
		},
		"panic" => Err(RuntimeResult::Panic),
		_ => Ok(None)
	};

	match result {
		// for compatability with native/jit
		Ok(result) => Ok(result.map(|val| match val {
			Value::None => Value::Integer(0),
			_ => val
		})),
		Err(err) => Err(err)
	}
}
