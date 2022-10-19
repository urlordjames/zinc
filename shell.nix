with import <nixpkgs> {
	overlays = [(import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz))];
};
let nightlyRust = (rustChannelOf {
		date = "2022-06-12";
		channel = "nightly";
		sha256 = "sha256-L1FLQis9jgw1/F6e6l6kXDF7qgHZyfvdUUyTutDcn7E=";
	});
in mkShell {
	nativeBuildInputs = [
		nightlyRust.rust
	];
}
