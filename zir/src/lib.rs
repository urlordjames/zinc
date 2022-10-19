#![feature(associated_type_bounds)]

pub mod parse;

mod node;

#[cfg(feature = "codegen")]
mod buildnode;

#[cfg(feature = "codegen")]
mod stdlib;

#[cfg(feature = "codegen")]
mod buildfunc;

#[cfg(feature = "codegen")]
use cranelift_module::{Module, FuncId};

#[cfg(feature = "interpreter")]
pub mod interpreter;

// TODO: declare functions first outside of this function
// TODO: too many parameters, maybe make some of them structs?
#[cfg(feature = "codegen")]
fn add_function<M: Module>(module: &mut M, func_map: &mut stdlib::FuncMap, function_name: String, function_info: node::FunctionInfo, data_index: &mut u64, optimize: bool) -> FuncId {
	use cranelift_codegen::ir::AbiParam;

	let mut sig = module.make_signature();
	function_info.args.iter().for_each(|arg| {
		let param_type = buildnode::deabstract(&arg.data_type, &module).expect("argument cannot be void");
		sig.params.push(AbiParam::new(param_type));
	});
	if let Some(return_type) = buildnode::deabstract(&function_info.return_type, &module) {
		sig.returns.push(AbiParam::new(return_type));
	}

	let declared_function = module.declare_function(
		&function_name,
		cranelift_module::Linkage::Export,
		&sig
	).unwrap();

	func_map.insert(function_name, declared_function);

	let func = buildfunc::build_func(sig, module, func_map, function_info.body, &function_info.args, data_index);
	let mut context = cranelift_codegen::Context::for_function(func);

	module.define_function(declared_function, &mut context).unwrap();

	if optimize {
		cranelift_preopt::optimize(&mut context, module.isa()).unwrap();
	}

	return declared_function;
}

#[cfg(feature = "native")]
pub fn build_object<P: AsRef<std::path::Path>>(mut file_description: node::FileDescription, output_path: P, optimize: bool) {
	use cranelift_codegen::isa;
	use cranelift_codegen::settings::Configurable;
	use std::io::Write;

	let mut shared_builder = cranelift_codegen::settings::builder();

	// benchmarks show statistically insignificant difference in speed, probably cranelift's fault
	shared_builder.set("opt_level", match optimize {
		true => "speed",
		false => "none"
	}).unwrap();

	let shared_flags = cranelift_codegen::settings::Flags::new(shared_builder);
	let isa_builder = isa::lookup(target_lexicon::Triple::host()).unwrap();
	let isa = isa_builder.finish(shared_flags).unwrap();
	let object_builder = cranelift_object::ObjectBuilder::new::<&str>(
		isa,
		"zinc_object",
		cranelift_module::default_libcall_names()
	).unwrap();

	let mut object_module = cranelift_object::ObjectModule::new(object_builder);

	let mut func_map = stdlib::FuncMap::new();
	stdlib::populate_stdlib(&mut func_map, &mut object_module);

	let mut data_index: u64 = 0;

	for (function_name, function) in file_description.functions.drain() {
		add_function(&mut object_module, &mut func_map, function_name, function, &mut data_index, optimize);
	}

	add_function(&mut object_module, &mut func_map, String::from("zinc_main"), node::FunctionInfo {
		body: file_description.statements,
		args: vec![],
		return_type: node::AbstractType::Void
	}, &mut data_index, optimize);

	let object_product = object_module.finish();

	let mut file = std::fs::File::create(output_path).unwrap();
	file.write_all(&object_product.emit().unwrap()).unwrap();
}

#[cfg(feature = "jit")]
pub unsafe fn run_jit<S: Into<String>>(file_description: node::FileDescription, optimize: bool, symbols: Vec<(S, *const u8)>) {
	let (jit_module, id) = jit_compile(file_description, optimize, symbols);
	let pointer = jit_module.get_finalized_function(id);
	let code_fn = core::mem::transmute::<_, fn()>(pointer);
	code_fn();
	jit_module.free_memory();
}

#[cfg(feature = "jit")]
fn jit_compile<S: Into<String>>(mut file_description: node::FileDescription, optimize: bool, symbols: Vec<(S, *const u8)>) -> (cranelift_jit::JITModule, FuncId) {
	let mut builder = cranelift_jit::JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

	for (symbol_name, symbol_val) in symbols {
		builder.symbol(symbol_name, symbol_val);
	}

	let mut jit_module = cranelift_jit::JITModule::new(builder);

	let mut func_map = stdlib::FuncMap::new();
	stdlib::populate_stdlib(&mut func_map, &mut jit_module);

	let mut data_index: u64 = 0;

	for (function_name, function) in file_description.functions.drain() {
		add_function(&mut jit_module, &mut func_map, function_name, function, &mut data_index, optimize);
	}

	let id = add_function(&mut jit_module, &mut func_map, String::from("zinc_main"), node::FunctionInfo {
		body: file_description.statements,
		args: vec![],
		return_type: node::AbstractType::Void
	}, &mut data_index, optimize);

	jit_module.finalize_definitions();
	return (jit_module, id);
}

#[cfg(feature = "jit")]
#[test]
fn test_jit() {
	let file_description = parse::parse(r#"
		let z: i32 = 2 + 3;
		assert_int_eq(z, 5);
		if (z == 5) {
			let n: i32 = 5;
			while (n > 0) {
				let n: i32 = n - 1;
			}
			assert_int_eq(n, 0);
		} else {
			panic();
		}
	"#).expect("failed to parse");

	#[no_mangle]
	extern "C" fn assert_int_eq(x: i32, y: i32) {
		assert_eq!(x, y);
	}

	#[no_mangle]
	extern "C" fn panic() {
		panic!("bad things have occured");
	}

	let symbols = vec![
		("assert_int_eq", assert_int_eq as *const u8),
		("panic", panic as *const u8)
	];
	unsafe { run_jit(file_description, true, symbols); };
}