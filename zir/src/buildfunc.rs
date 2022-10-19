use cranelift_codegen::ir::{Signature, Function, UserFuncName, InstBuilder};
use cranelift_frontend::{FunctionBuilderContext, FunctionBuilder};
use cranelift_module::Module;

use crate::node::{Statement, Definition};
use crate::buildnode::BuildState;
use crate::stdlib::FuncMap;

pub fn build_func<M: Module>(sig: Signature, module: &mut M, func_map: &mut FuncMap, statements: Vec<Statement>, args: &Vec<Definition>, data_index: &mut u64) -> Function {
	let mut fn_builder_ctx = FunctionBuilderContext::new();

	let mut func = Function::with_name_signature(UserFuncName::user(0, func_map.len() as u32), sig);
	let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

	let main_block = builder.create_block();
	builder.append_block_params_for_function_params(main_block);
	builder.switch_to_block(main_block);

	let mut build_state = BuildState::new(&mut builder, &func_map, module, args, data_index);

	build_state.build_statements(statements);
	if !builder.is_filled() {
		builder.ins().return_(&[]);
	}

	builder.seal_all_blocks();
	builder.finalize();

	let flags = cranelift_codegen::settings::Flags::new(cranelift_codegen::settings::builder());
	cranelift_codegen::verify_function(&func, &flags).unwrap();

	return func;
}
