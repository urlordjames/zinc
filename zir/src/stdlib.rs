use cranelift_codegen::ir::{Signature, AbiParam, types::*};
use cranelift_module::{Module, FuncId, Linkage};

pub type FuncMap = std::collections::HashMap::<String, FuncId>;

fn add_function<M: Module>(fn_map: &mut FuncMap, object_module: &mut M, name: &str, sig: Signature) {
	let declared_func = object_module.declare_function(
		name,
		Linkage::Import,
		&sig
	).unwrap();

	fn_map.insert(String::from(name), declared_func);
}

pub fn populate_stdlib<M: Module>(fn_map: &mut FuncMap, object_module: &mut M) {
	let pointer_type = object_module.target_config().pointer_type();

	{
		let mut print_int_sig = object_module.make_signature();
		print_int_sig.params.push(AbiParam::new(I32));
		add_function(fn_map, object_module, "print_int", print_int_sig);
	}

	{
		let mut print_bool_sig = object_module.make_signature();
		print_bool_sig.params.push(AbiParam::new(B1));
		add_function(fn_map, object_module, "print_bool", print_bool_sig);
	}

	{
		let mut print_str_sig = object_module.make_signature();
		print_str_sig.params.push(AbiParam::new(pointer_type));
		add_function(fn_map, object_module, "print_str", print_str_sig);
	}

	{
		let mut str_eq_sig = object_module.make_signature();
		str_eq_sig.params.push(AbiParam::new(pointer_type));
		str_eq_sig.params.push(AbiParam::new(pointer_type));
		str_eq_sig.returns.push(AbiParam::new(B1));
		add_function(fn_map, object_module, "str_eq", str_eq_sig);
	}

	{
		let mut str_len_sig = object_module.make_signature();
		str_len_sig.params.push(AbiParam::new(pointer_type));
		str_len_sig.returns.push(AbiParam::new(I32));
		add_function(fn_map, object_module, "str_len", str_len_sig);
	}

	{
		let mut str_concat_sig = object_module.make_signature();
		str_concat_sig.params.push(AbiParam::new(pointer_type));
		str_concat_sig.params.push(AbiParam::new(pointer_type));
		str_concat_sig.returns.push(AbiParam::new(pointer_type));
		add_function(fn_map, object_module, "str_concat", str_concat_sig);
	}

	{
		let mut assert_int_eq_sig = object_module.make_signature();
		assert_int_eq_sig.params.push(AbiParam::new(I32));
		assert_int_eq_sig.params.push(AbiParam::new(I32));
		add_function(fn_map, object_module, "assert_int_eq", assert_int_eq_sig);
	}

	{
		let mut assert_bool_eq_sig = object_module.make_signature();
		assert_bool_eq_sig.params.push(AbiParam::new(B1));
		assert_bool_eq_sig.params.push(AbiParam::new(B1));
		add_function(fn_map, object_module, "assert_bool_eq", assert_bool_eq_sig);
	}

	{
		let mut assert_str_eq_sig = object_module.make_signature();
		assert_str_eq_sig.params.push(AbiParam::new(pointer_type));
		assert_str_eq_sig.params.push(AbiParam::new(pointer_type));
		add_function(fn_map, object_module, "assert_str_eq", assert_str_eq_sig);
	}

	{
		let panic_sig = object_module.make_signature();
		add_function(fn_map, object_module, "panic", panic_sig);
	}
}
