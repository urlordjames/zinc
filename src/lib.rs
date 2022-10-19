use std::path::Path;
#[cfg(feature = "native")]
use std::io::Write;

#[cfg(all(target_os = "windows", feature = "native"))]
fn link(temp_path: &Path, output: &Path, optimize: bool) {
	let cl = cc::windows_registry::find_tool("x86_64-msvc", "cl.exe").expect("cannot find cl");

	let output_arg = String::from("/OUT:") + output.to_str().expect("invalid output path");

	let output = cl.to_command().
	current_dir(temp_path).args([
		"zir_obj.o",
		"zinc_std_c.c",
		"zinc_entry_c.c",
		// benchmarks show that /O2 and /Ox both produce slower code than /0d
		match optimize {
			true => "/Od",
			false => "/Od"
		},
		"/link",
		&output_arg]).output().unwrap();

	if !output.status.success() {
		std::io::stdout().write_all(&output.stdout).unwrap();
		panic!("failed to link");
	}
}

#[cfg(all(target_os = "linux", feature = "native"))]
fn link(temp_path: &Path, output: &Path, optimize: bool) {
	let mut gcc = std::process::Command::new("gcc");
	let output = gcc.current_dir(temp_path).args([
		"zir_obj.o",
		"zinc_std_c.c",
		"zinc_entry_c.c",
		match optimize {
			true => "-O2",
			false => "-O0"
		},
		"-o", output.to_str().expect("invalid output path")]).output().unwrap();

	if !output.status.success() {
		std::io::stderr().write_all(&output.stderr).unwrap();
		panic!("failed to link");
	}
}

#[cfg(feature = "native")]
pub fn build_executable(input: &Path, output: &Path, optimize: bool) {
	use path_absolutize::Absolutize;

	let input_contents = std::fs::read_to_string(input).expect("cannot read file");
	let file_description = zir::parse::parse(&input_contents).expect("failed to parse");

	let temp_dir = tempfile::tempdir().expect("cannot create temporary directory");
	let temp_path = temp_dir.path();

	{
		let std_path = temp_path.join("zinc_std_c.c");
		let mut std_file = std::fs::File::create(&std_path).expect("cannot create standard library file");
		std_file.write_all(include_bytes!("../zinc_std_c.c")).expect("cannot write to standard library file");
	}

	{
		let entry_path = temp_path.join("zinc_entry_c.c");
		let mut entry_file = std::fs::File::create(&entry_path).expect("cannot create entry file");
		entry_file.write_all(include_bytes!("../zinc_entry_c.c")).expect("cannot write to entry file");
	}

	zir::build_object(file_description, temp_path.join("zir_obj.o"), optimize);
	link(temp_path, &output.absolutize().unwrap(), optimize);
}

#[cfg(not(feature = "native"))]
pub fn build_executable(_input: &Path, _output: &Path, _optimize: bool) {
	panic!("native feature not enabled");
}

#[cfg(feature = "jit")]
extern {
	fn print_int(x: i32);
	fn print_bool(x: bool);
	fn print_str(str: *const u8);
	fn str_eq(lhs: *const u8, rhs: *const u8) -> bool;
	fn str_len(str: *const u8) -> i32;
	fn str_concat(lhs: *const u8, rhs: *const u8) -> *const u8;
	fn assert_int_eq(lhs: i32, rhs: i32);
	fn assert_bool_eq(lhs: i32, rhs: i32);
	fn assert_str_eq(lhs: *const u8, rhs: *const u8);
	fn panic();
}

#[cfg(feature = "jit")]
pub fn run_jit(code: &str, optimize: bool) {
	let file_description = zir::parse::parse(code).expect("failed to parse");
	let symbols = vec![
		("print_int", print_int as *const u8),
		("print_bool", print_bool as *const u8),
		("print_str", print_str as *const u8),
		("str_eq", str_eq as *const u8),
		("str_len", str_len as *const u8),
		("str_concat", str_concat as *const u8),
		("assert_int_eq", assert_int_eq as *const u8),
		("assert_bool_eq", assert_bool_eq as *const u8),
		("assert_str_eq", assert_str_eq as *const u8),
		("panic", panic as *const u8)
	];
	unsafe { zir::run_jit(file_description, optimize, symbols); };
}

#[cfg(not(feature = "jit"))]
pub fn run_jit(_code: &str, _optimize: bool) {
	panic!("jit feature not enabled");
}

#[cfg(feature = "interpreter")]
pub fn run_interpreter(code: &str) -> String {
	let file_description = zir::parse::parse(code).expect("failed to parse");
	zir::interpreter::interpret(file_description).unwrap()
}

#[cfg(not(feature = "interpreter"))]
pub fn run_interpreter(_code: &str) -> String {
	panic!("interpreter feature not enabled");
}
