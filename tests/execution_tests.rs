use std::path::Path;

#[cfg(feature = "native")]
fn test_native(file_path: &Path, optimize: bool) {
	use std::io::Write;

	let temp_dir = tempfile::tempdir().expect("cannot create temporary directory");
	let executable_path = temp_dir.path().join("test");

	zink::build_executable(file_path, &executable_path, optimize);

	let output = std::process::Command::new(executable_path).output().unwrap();
	std::io::stdout().write_all(&output.stdout).unwrap();
	assert!(output.status.success());
}

#[cfg(not(feature = "native"))]
fn test_native(_file_path: &Path, _optimize: bool) {
	println!("native not enabled, skipping test")
}

#[cfg(feature = "jit")]
fn test_jit(file_path: &Path) {
	let code = std::fs::read_to_string(file_path).unwrap();

	println!("testing unoptimized JIT");
	zink::run_jit(&code, false);
	println!("testing optimized JIT");
	zink::run_jit(&code, true);
}

#[cfg(not(feature = "jit"))]
fn test_jit(_file_path: &Path) {
	println!("jit not enabled, skipping test")
}

#[cfg(feature = "interpreter")]
fn test_interpreter(file_path: &Path) {
	let code = std::fs::read_to_string(file_path).unwrap();
	zink::run_interpreter(&code);
}

#[cfg(not(feature = "interpreter"))]
fn test_interpreter(_file_path: &Path) {
	println!("interpreter not enabled, skipping test");
}

fn test_file(file_path: &Path) {
	println!("testing unoptimized native");
	test_native(file_path, false);
	println!("testing optimized native");
	test_native(file_path, true);

	test_jit(file_path);

	test_interpreter(file_path);
}

#[test]
fn booleans() {
	test_file(Path::new("./tests/booleans.zn"));
}

#[test]
fn conditionals() {
	test_file(Path::new("./tests/conditionals.zn"));
}

#[test]
fn functions() {
	test_file(Path::new("./tests/functions.zn"));
}

#[test]
fn infinite_loop() {
	test_file(Path::new("./tests/infinite_loop.zn"));
}

#[test]
fn integers() {
	test_file(Path::new("./tests/integers.zn"));
}

#[test]
fn nested_while() {
	test_file(Path::new("./tests/nested_while.zn"));
}

#[test]
fn recursion() {
	test_file(Path::new("./tests/recursion.zn"));
}

#[test]
fn strings() {
	test_file(Path::new("./tests/strings.zn"));
}

#[test]
fn variables() {
	test_file(Path::new("./tests/variables.zn"));
}

#[test]
fn while_loop() {
	test_file(Path::new("./tests/while_loop.zn"));
}
