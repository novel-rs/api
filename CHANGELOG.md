# Changelog

All notable changes to this project will be documented in this file.

## [0.9.0] - 2023-07-31

### <!-- 0 -->⛰️ Features

- Encrypt cookie file and config file
- Add database time-consuming log
- Use sqlcipher
- Improve SMS verification code enter

### <!-- 2 -->🚜 Refactor

- Refactor code slightly
- Use dialoguer
- Use ring instead of sha2
- No longer use boringssl
- No longer use parking_lot

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Update version
- Update version
- No longer ignore RUSTSEC-2022-0090
- Update deps
- Update deps
- Update deps
- Update sfacg version
- Update deps

## [0.8.0] - 2023-07-11

### <!-- 2 -->🚜 Refactor

- Specify the name of the organization
- Disable redirect
- Use open instead of opener

### <!-- 5 -->🎨 Styling

- Run prettier

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Update deps
- Update deps
- Update deps
- Bump uuid
- Bump user agent
- Bump sfacg version
- Update scraper requirement from 0.16.0 to 0.17.1 ([#17](https://github.com/novel-rs/api/issues/17))
- Pre-commit autoupdate ([#16](https://github.com/novel-rs/api/issues/16))
- Update deps
- Pre-commit autoupdate ([#15](https://github.com/novel-rs/api/issues/15))

## [0.7.2] - 2023-06-12

### <!-- 1 -->🐛 Bug Fixes

- Bookshelf_infos handles albums and comics

### <!-- 3 -->📚 Documentation

- Add msrv badge

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Update deps
- Correct incorrect manifest field
- Update deps
- Record minimum supported Rust version in metadata

## [0.7.1] - 2023-06-02

### <!-- 1 -->🐛 Bug Fixes

- Add is_accessible and is_valid missing `! `

## [0.7.0] - 2023-06-02

### <!-- 0 -->⛰️ Features

- Set the password's memory to zero

## [0.6.0] - 2023-06-02

### <!-- 0 -->⛰️ Features

- Set the password's memory to zero

### <!-- 1 -->🐛 Bug Fixes

- Attempt to fix panics on Linux
- Attempt to fix panics on Linux
- Uid is generated as empty

### <!-- 2 -->🚜 Refactor

- Remove is_some_and
- Use is_ci instead of build.rs

### <!-- 6 -->🧪 Testing

- Attempt to remove is_ci check
- Add keyring async test

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Add ciweimao example
- Update ciweimao version
- Bump sfacg version
- Update deps
- Temporarily disable trust-dns
- Update deps
- Update .justfile
- Update sfacg and iOS version
- Update cliff.toml
- Update cliff.toml
- Update changelog
- Update cliff.toml
- Update deps
- Use cargo-nextest

## [0.5.0] - 2023-05-17

### <!-- 2 -->🚜 Refactor

- Some minor modifications to the client
- Modify some log output
- Modify some log output

### <!-- 3 -->📚 Documentation

- Update README.md

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Fix wrong tag
- Remove semver-checks directory
- Update changelog
- Update deps
- Update deps
- Remove outdated action
- Update deps
- Change cliff.toml
- Update deps
- Add git-cliff to generate changelog
- Remove redundant install action
- Update app version
- Update deps
- Pre-commit autoupdate ([#12](https://github.com/novel-rs/api/issues/12))

## [0.4.0] - 2023-04-10

### <!-- 0 -->⛰️ Features

- Add a blocked tag
- Remove non-system tags
- Impl ToString for Category and Tag
- Add novels api
- Add category and tag api
- Add category and tag api
- Add shutdown for client
- Disable compress
- Add can_download for ChapterInfo

### <!-- 1 -->🐛 Bug Fixes

- Solve the problem of http image download

### <!-- 2 -->🚜 Refactor

- Many small improvements
- Use tokio::fs::try_exists
- Many minor modifications
- Change shutdown parament
- Some minor modifications
- Some minor modifications
- Remove some test code
- Remove the lifetimes of Options
- Change Options field

### <!-- 3 -->📚 Documentation

- Update README.md

### <!-- 6 -->🧪 Testing

- Add novels test

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Update deps
- Update deps
- Update machine-uid requirement from 0.2.0 to 0.3.0 ([#10](https://github.com/novel-rs/api/issues/10))
- Pre-commit autoupdate ([#11](https://github.com/novel-rs/api/issues/11))
- Update deps
- Update opener requirement from 0.5.2 to 0.6.0 ([#9](https://github.com/novel-rs/api/issues/9))
- Update directories requirement from 4.0.1 to 5.0.0 ([#8](https://github.com/novel-rs/api/issues/8))
- Pre-commit autoupdate ([#7](https://github.com/novel-rs/api/issues/7))
- Update deps
- Update deps
- Pre-commit autoupdate ([#4](https://github.com/novel-rs/api/issues/4))
- Update
- Update deps
- Update example
- Bump uuid
- Disable default-features for all crate
- Update deps
- Add cargo-semver-checks install action

## [0.3.0] - 2023-01-30

### <!-- 0 -->⛰️ Features

- Handle the case that novel does not exist
- Add is_some_and()
- Add home_dir_path()
- Initial

### <!-- 1 -->🐛 Bug Fixes

- Check is logged in incorrectly
- Error in image path parsing
- Wrong path on windows

### <!-- 2 -->🚜 Refactor

- Many minor modifications
- Drop confy
- Many minor modifications
- Many minor modifications
- Many minor modifications
- Handle response result parsing errors
- Some minor modifications
- Apply clippy
- Rename a error name
- Rename some fields and add doc

### <!-- 6 -->🧪 Testing

- Fix failing test on Windows
- Remove test that don't work on CI
- Ignore Keyring test in CI

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Add check semver version-tag-prefix
- Add aarch64-apple-darwin target
- Remove unused feature
- Update geetest.js
- Update deps
- Add changelog
- Remove outdated action schedule
- Bump opener
- Add cargo-semver-checks-action
- Add license allow
- Change prompt
- Remove redundant period
- Install NASM when building on Windows
