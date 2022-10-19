use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::path::{Path, PathBuf};

fn build_file(temp_dir: &Path, file_path: &Path) -> (PathBuf, PathBuf) {
	let optimized_path = temp_dir.join("optimized");
	let unoptimized_path = temp_dir.join("unoptimized");

	zink::build_executable(file_path, &optimized_path, true);
	zink::build_executable(file_path, &unoptimized_path, false);

	return (optimized_path, unoptimized_path);
}

fn test_executable(executable_path: &Path) {
	let output = std::process::Command::new(executable_path).output().unwrap();
	assert!(output.status.success());
}

fn benchmark_entry(c: &mut Criterion) {
	let mut group = c.benchmark_group("performance");
	for file_name in ["test.zn"].iter() {
		let file_path = &Path::new("./benches").join(file_name);
		let temp_dir = tempfile::tempdir().expect("cannot create temporary directory");
		let (opt, unopt) = build_file(temp_dir.path(), file_path);

		group.bench_with_input(BenchmarkId::new("native-optimized", file_name),
			&opt, |b, path| b.iter(|| test_executable(path)));
		group.bench_with_input(BenchmarkId::new("native-unoptimized", file_name),
			&unopt, |b, path| b.iter(|| test_executable(path)));
		group.bench_with_input(BenchmarkId::new("JIT-optimized", file_name),
			file_path, |b, path| b.iter(|| zink::run_jit(&std::fs::read_to_string(path).unwrap(), true)));
		group.bench_with_input(BenchmarkId::new("JIT-unoptimized", file_name),
			file_path, |b, path| b.iter(|| zink::run_jit(&std::fs::read_to_string(path).unwrap(), false)));
		group.bench_with_input(BenchmarkId::new("interpreter", file_name),
			file_path, |b, path| b.iter(|| zink::run_interpreter(&std::fs::read_to_string(path).unwrap())));
	}
}

criterion_group!(benches, benchmark_entry);
criterion_main!(benches);
