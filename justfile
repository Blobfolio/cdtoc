##
# Development Recipes
#
# This justfile is intended to be run from inside a Docker sandbox:
# https://github.com/Blobfolio/righteous-sandbox
#
# docker run \
#	--rm \
#	-v "{{ invocation_directory() }}":/share \
#	-it \
#	--name "righteous_sandbox" \
#	"righteous/sandbox:debian"
#
# Alternatively, you can just run cargo commands the usual way and ignore these
# recipes.
##

pkg_id      := "cdtoc"
pkg_name    := "CDTOC"

cargo_dir   := "/tmp/" + pkg_id + "-cargo"
doc_dir     := justfile_directory() + "/doc"



# Bench it!
bench BENCH="":
	#!/usr/bin/env bash

	clear
	if [ -z "{{ BENCH }}" ]; then
		cargo bench \
			--benches \
			--target x86_64-unknown-linux-gnu \
			--target-dir "{{ cargo_dir }}"
	else
		cargo bench \
			--bench "{{ BENCH }}" \
			--target x86_64-unknown-linux-gnu \
			--target-dir "{{ cargo_dir }}"
	fi
	exit 0


# Check Release!
@check:
	# First let's build the Rust bit.
	cargo check \
		--release \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Clean Cargo crap.
@clean:
	# Most things go here.
	[ ! -d "{{ cargo_dir }}" ] || rm -rf "{{ cargo_dir }}"

	# But some Cargo apps place shit in subdirectories even if
	# they place *other* shit in the designated target dir. Haha.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"

	cargo update -w


# Clippy.
@clippy:
	clear
	cargo clippy \
		--release \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Generate CREDITS.
@credits:
	cargo bashman --no-bash --no-man
	just _fix-chown "{{ justfile_directory() }}/CREDITS.md"


# Build Docs.
@doc:
	# Make sure nightly is installed; this version generates better docs.
	# env RUSTUP_PERMIT_COPY_RENAME=true rustup install nightly

	# Make the docs.
	cargo +nightly rustdoc \
		--release \
		--all-features \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}" \
		-- \
		--cfg docsrs

	# Move the docs and clean up ownership.
	[ ! -d "{{ doc_dir }}" ] || rm -rf "{{ doc_dir }}"
	mv "{{ cargo_dir }}/x86_64-unknown-linux-gnu/doc" "{{ justfile_directory() }}"
	just _fix-chown "{{ doc_dir }}"


# Unit tests!
@test:
	clear
	fyi task "Testing w/ Default Features"
	cargo test \
		--release \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"

	echo ""
	fyi task "Testing w/ No Features"
	cargo test \
		--release \
		--no-default-features \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"

	just _test faster-hex
	just _test accuraterip,faster-hex
	just _test cddb,faster-hex
	just _test accuraterip
	just _test cddb
	just _test ctdb
	just _test musicbrainz


@_test FEATURES:
	echo ""
	fyi task "Testing w/ {{ FEATURES }}"
	cargo test \
		--release \
		--no-default-features \
		--features {{ FEATURES }} \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Get/Set version.
version: credits
	#!/usr/bin/env bash

	# Current version.
	_ver1="$( toml get "{{ justfile_directory() }}/Cargo.toml" package.version | \
		sed 's/"//g' )"

	# Find out if we want to bump it.
	_ver2="$( whiptail --inputbox "Set {{ pkg_name }} version:" --title "Release Version" 0 0 "$_ver1" 3>&1 1>&2 2>&3 )"

	exitstatus=$?
	if [ $exitstatus != 0 ] || [ "$_ver1" = "$_ver2" ]; then
		exit 0
	fi

	fyi success "Setting version to $_ver2."

	# Set the release version!
	just _version "{{ justfile_directory() }}" "$_ver2"


# Set version for real.
@_version DIR VER:
	[ -f "{{ DIR }}/Cargo.toml" ] || exit 1

	# Set the release version!
	toml set "{{ DIR }}/Cargo.toml" package.version "{{ VER }}" > /tmp/Cargo.toml
	just _fix-chown "/tmp/Cargo.toml"
	mv "/tmp/Cargo.toml" "{{ DIR }}/Cargo.toml"


# Fix file/directory permissions.
@_fix-chmod PATH:
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type f -exec chmod 0644 {} +
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type d -exec chmod 0755 {} +


# Fix file/directory ownership.
@_fix-chown PATH:
	[ ! -e "{{ PATH }}" ] || chown -R --reference="{{ justfile() }}" "{{ PATH }}"
