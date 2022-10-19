use cranelift_codegen::ir::{InstBuilder, Value, immediates::Imm64, entities::FuncRef, condcodes::IntCC, types::*};
use cranelift_frontend::{FunctionBuilder, Variable};
use cranelift_module::{Module, DataContext, Linkage};
use crate::stdlib::FuncMap;
use crate::node::{Node, Statement, Definition, AbstractType};

pub fn deabstract<M: Module>(abstract_type: &AbstractType, object_module: &M) -> Option<Type> {
	match abstract_type {
		AbstractType::Integer => Some(I32),
		AbstractType::Boolean => Some(B1),
		AbstractType::String => Some(object_module.target_config().pointer_type()),
		AbstractType::Void => None,
	}
}

pub struct BuildState<'a, 'b, M: Module> {
	pub builder: &'a mut FunctionBuilder<'b>,
	func_map: &'a FuncMap,
	module: &'a mut M,
	variable_list: std::collections::HashMap::<String, Variable>,
	variable_count: u32,
	data_index: &'a mut u64
}

impl<'a, 'b, M: Module> BuildState<'a, 'b, M> {
	pub fn new(builder: &'a mut FunctionBuilder<'b>, func_map: &'a FuncMap, module: &'a mut M, args: &Vec<Definition>, data_index: &'a mut u64) -> Self {
		let current_block = builder.current_block().unwrap();
		let block_args = builder.block_params(current_block).to_vec();

		let mut inst = Self {
			builder,
			func_map,
			module,
			variable_list: std::collections::HashMap::new(),
			variable_count: 0,
			data_index
		};

		for (arg, block_arg) in args.iter().zip(block_args) {
			let data_type = deabstract(&arg.data_type, &inst.module).expect("argument cannot be void");
			let var = inst.get_new_variable(arg.name.clone(), data_type);
			inst.builder.def_var(var, block_arg);
		}

		return inst;
	}

	pub fn get_new_variable(&mut self, name: String, var_type: Type) -> Variable {
		*self.variable_list.entry(name).or_insert_with(|| {
			let var = Variable::with_u32(self.variable_count);
			self.variable_count += 1;
			self.builder.declare_var(var, var_type);
			return var;
		})
	}

	fn get_declared_variable(&mut self, name: String) -> Option<Variable> {
		self.variable_list.get(&name).map(|var| *var)
	}

	fn build_node(&mut self, node: &Node) -> Value {
		match node {
			Node::Int(val) => {
				self.builder.ins().iconst(I32, Imm64::new(*val as i64))
			},
			Node::Add { lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				self.builder.ins().iadd(lv, rv)
			},
			Node::Subtract{ lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				self.builder.ins().isub(lv, rv)
			},
			Node::Multiply { lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				self.builder.ins().imul(lv, rv)
			},
			Node::Divide { lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				self.builder.ins().sdiv(lv, rv)
			},
			Node::Equal { lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				self.builder.ins().icmp(IntCC::Equal, lv, rv)
			},
			Node::NotEqual { lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				self.builder.ins().icmp(IntCC::NotEqual, lv, rv)
			},
			Node::LessThanOrEqual { lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, lv, rv)
			},
			Node::GreaterThanOrEqual { lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lv, rv)
			},
			Node::LessThan { lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				self.builder.ins().icmp(IntCC::SignedLessThan, lv, rv)
			},
			Node::GreaterThan { lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				self.builder.ins().icmp(IntCC::SignedGreaterThan, lv, rv)
			},

			Node::Bool(val) => {
				self.builder.ins().bconst(B1, *val)
			},
	
			Node::BoolEqual{ lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				let li = self.builder.ins().bint(I8, lv);
				let ri = self.builder.ins().bint(I8, rv);
				self.builder.ins().icmp(IntCC::Equal, li, ri)
			},
			Node::BoolNotEqual { lhs, rhs } => {
				let lv = self.build_node(lhs);
				let rv = self.build_node(rhs);
				let li = self.builder.ins().bint(I8, lv);
				let ri = self.builder.ins().bint(I8, rv);
				self.builder.ins().icmp(IntCC::NotEqual, li, ri)
			},

			Node::StringLiteral(val) => {
				let mut data_context = DataContext::new();

				// TODO: sigh comma groan clone bad
				let mut string_bytes = val.clone().into_bytes();
				string_bytes.push('\0' as u8);

				let n = *self.data_index;

				data_context.define(string_bytes.into_boxed_slice());
				let data_id = self.module.declare_data(
					&format!("string{}", n),
					Linkage::Hidden,
					false,
					false).unwrap();
	
				*self.data_index = n + 1;

				self.module.define_data(data_id, &data_context).unwrap();

				let pointer_type = self.module.target_config().pointer_type();
				let global_value = self.module.declare_data_in_func(
					data_id,
					self.builder.func);
				self.builder.ins().symbol_value(pointer_type, global_value)
			},

			Node::Set { name, var_type, value } => {
				let data_type = deabstract(var_type, &self.module);
				let var = self.get_new_variable(String::from(name), data_type.expect("variable type cannot be void"));
	
				let value = self.build_node(value);
				self.builder.def_var(var, value);
				self.builder.use_var(var)
			},
			Node::Get { name } => {
				let var = self.get_declared_variable(String::from(name)).expect("undeclared variable");
				self.builder.use_var(var)
			},

			Node::Function { name, args } => {
				let args: Vec<Value> = args.iter().map(|node| {
					self.build_node(node)
				}).collect();

				let imported_func: FuncRef = self.module.declare_func_in_func(
					self.func_map[name],
					self.builder.func
				);

				let function_result = self.builder.ins().call(imported_func, args.as_slice());
				match self.builder.inst_results(function_result).get(0) {
					Some(val) => *val,
					None => self.builder.ins().iconst(I32, 0) // TODO: make a better system
				}
			}
		}
	}

	pub fn build_statements(&mut self, statements: Vec<Statement>) {
		for statement in statements {
			match statement {
				Statement::Node(node) => {
					self.build_node(&node);
				},
				Statement::Return(node) => {
					let val = self.build_node(&node);
					self.builder.ins().return_(&[val]);
				},
				Statement::If { condition, branch, else_branch } => {
					let val = self.build_node(&condition);

					let cond_block = self.builder.create_block();
					self.builder.ins().brnz(val, cond_block, &[]);

					let else_block = self.builder.create_block();
					self.builder.ins().jump(else_block, &[]);

					self.builder.switch_to_block(else_block);
					self.build_statements(else_branch);

					let mut after_block = None;
					if !self.builder.is_filled() {
						let new_after_block = self.builder.create_block();
						self.builder.ins().jump(new_after_block, &[]);
						after_block = Some(new_after_block);
					}

					self.builder.switch_to_block(cond_block);
					self.build_statements(branch);
					if !self.builder.is_filled() {
						match after_block {
							Some(after_block) => {
								self.builder.ins().jump(after_block, &[]);
							},
							None => {
								let new_after_block = self.builder.create_block();
								self.builder.ins().jump(new_after_block, &[]);
								after_block = Some(new_after_block);
							}
						}
					}

					match after_block {
						Some(after_block) => {
							self.builder.switch_to_block(after_block);
						},
						None => {
							assert!(self.builder.is_filled());
						}
					}
				},
				Statement::While { condition, loop_statements } => {
					let test_block = self.builder.create_block();
					self.builder.ins().jump(test_block, &[]);

					let loop_block = self.builder.create_block();
					let after_block = self.builder.create_block();

					self.builder.switch_to_block(test_block);
					let val = self.build_node(&condition);
					self.builder.ins().brnz(val, loop_block, &[]);
					self.builder.ins().jump(after_block, &[]);

					self.builder.switch_to_block(loop_block);
					self.build_statements(loop_statements);
					self.builder.ins().jump(test_block, &[]);

					self.builder.switch_to_block(after_block);
				},
				Statement::InfiniteLoop(loop_statements) => {
					let loop_block = self.builder.create_block();
					self.builder.ins().jump(loop_block, &[]);
					self.builder.switch_to_block(loop_block);

					self.build_statements(loop_statements);
					self.builder.ins().jump(loop_block, &[]);
				}
			};
		}
	}
}