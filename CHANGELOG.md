# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### <!-- 7 -->⚙️ Miscellaneous Tasks

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
- Update changelog

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

### <!-- 3 -->📚 Documentation

- Update changelog

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
