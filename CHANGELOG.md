# Changelog

All notable changes to this project will be documented in this file.

## [0.11.2] - 2024-06-17

### 🚜 Refactor

- _(sfacg)_ Remove useless code

### 🎨 Styling

- Reformatting the code

## [0.11.1] - 2024-05-25

### 🐛 Bug Fixes

- Add a request to retry

## [0.11.0] - 2024-05-25

### 🐛 Bug Fixes

- _(sfacg)_ Handling content encryption

### 🚜 Refactor

- Apply clippy

## [0.10.7] - 2024-04-29

### ⛰️ Features

- Set connect timeout for db connect

### 🐛 Bug Fixes

- _(ciweimao)_ Http request occasionally fails

### 🚜 Refactor

- Simplify webp saving

### ⚙️ Miscellaneous Tasks

- _(ci)_ Coverage is no longer run during pull_request
- Add examples

## [0.10.6] - 2024-04-22

### 🐛 Bug Fixes

- _(ciweimao)_ Remove an invalid non-system tag

## [0.10.5] - 2024-04-20

### ⛰️ Features

- _(ciweimao)_ Add some non-system tags

### 🐛 Bug Fixes

- _(ciweimao)_ Keyword search results are incorrect

## [0.10.4] - 2024-04-20

### ⛰️ Features

- _(ciweimao)_ Add a non-system tag

### 🐛 Bug Fixes

- _(ci)_ Temporarily turn off semver-checks because webp yanked

### ⚙️ Miscellaneous Tasks

- _(ci)_ Remove unnecessary operations

## [0.10.3] - 2024-04-17

### ⚙️ Miscellaneous Tasks

- _(ciweimao)_ Downgrade app version

## [0.10.2] - 2024-04-13

### 🐛 Bug Fixes

- _(ciyuanji)_ Some chapters failed to download

### 📚 Documentation

- Update README.md

### ⚙️ Miscellaneous Tasks

- Update app version

## [0.10.1] - 2024-04-12

### 🐛 Bug Fixes

- _(sfacg)_ Add assert to limit search size
- Close connection before moving database

### 📚 Documentation

- Update README.md

## [0.10.0] - 2024-04-11

### ⛰️ Features

- _(ciyuanji)_ Add some tags
- Ignore errors when file is parsed incorrectly
- _(net)_ Set timeout
- Rewritten most of the code

### 🐛 Bug Fixes

- _(ciweimao)_ Wrong search
- _(ci)_ Use macos-14 instead of macos-latest
- Sometimes encounters errors when sending requests
- _(ciweimao)_ Chapters without price are invalid
- Wrong sql type
- Fix some warnings
- Some minor fixes

### 🚜 Refactor

- Rename client api name
- Change log level
- Remove db timing
- _(utils)_ Change save_as_webp's parameter
- Remove Identifier type
- _(net)_ Update reqwest and remove reqwest_cookie_store
- Rename a field

### ⚙️ Miscellaneous Tasks

- Upgrade various project files
- Upgrade various project files

## [0.9.0] - 2023-07-31

### ⛰️ Features

- Encrypt cookie file and config file
- Add database time-consuming log
- Use sqlcipher
- Improve SMS verification code enter

### 🚜 Refactor

- Refactor code slightly
- Use dialoguer
- Use ring instead of sha2
- No longer use boringssl
- No longer use parking_lot

### ⚙️ Miscellaneous Tasks

- _(sfacg)_ Update version
- _(ciweimao)_ Update version
- No longer ignore RUSTSEC-2022-0090
- _(sfacg)_ Update sfacg version

## [0.8.0] - 2023-07-11

### 🚜 Refactor

- _(utils)_ Specify the name of the organization
- _(net)_ Disable redirect
- Use open instead of opener

### 🎨 Styling

- Run prettier

### ⚙️ Miscellaneous Tasks

- _(net)_ Bump user agent
- _(sfacg)_ Bump sfacg version

## [0.7.2] - 2023-06-12

### 🐛 Bug Fixes

- _(sfacg)_ Bookshelf_infos handles albums and comics

### 📚 Documentation

- Add msrv badge

### ⚙️ Miscellaneous Tasks

- Correct incorrect manifest field
- Record minimum supported Rust version in metadata

## [0.7.1] - 2023-06-02

### 🐛 Bug Fixes

- Add is_accessible and is_valid missing `! `

## [0.7.0] - 2023-06-02

### ⛰️ Features

- _(keyring)_ Set the password's memory to zero

## [0.6.0] - 2023-06-02

### ⛰️ Features

- Set the password's memory to zero

### 🐛 Bug Fixes

- _(keyring)_ Attempt to fix panics on Linux
- _(keyring)_ Attempt to fix panics on Linux
- Uid is generated as empty

### 🚜 Refactor

- Remove is_some_and
- Use is_ci instead of build.rs

### 🧪 Testing

- _(keyring)_ Attempt to remove is_ci check
- Add keyring async test

### ⚙️ Miscellaneous Tasks

- Add ciweimao example
- _(ciweimao)_ Update ciweimao version
- _(sfacg)_ Bump sfacg version
- Temporarily disable trust-dns
- Update .justfile
- _(sfacg)_ Update sfacg and iOS version
- Update cliff.toml
- Update cliff.toml
- Update changelog
- Update cliff.toml
- Use cargo-nextest

## [0.5.0] - 2023-05-17

### 🚜 Refactor

- Some minor modifications to the client
- Modify some log output
- Modify some log output

### 📚 Documentation

- Update README.md

### ⚙️ Miscellaneous Tasks

- _(ci)_ Fix wrong tag
- _(ci)_ Remove semver-checks directory
- Update changelog
- _(ci)_ Remove outdated action
- Change cliff.toml
- Add git-cliff to generate changelog
- Remove redundant install action
- _(sfacg)_ Update app version

## [0.4.0] - 2023-04-10

### ⛰️ Features

- _(sfacg)_ Add a blocked tag
- _(ciweimao)_ Remove non-system tags
- Impl ToString for Category and Tag
- Add novels api
- _(ciweimao)_ Add category and tag api
- _(sfacg)_ Add category and tag api
- Add shutdown for client
- _(ciweimao)_ Disable compress
- Add can_download for ChapterInfo

### 🐛 Bug Fixes

- Solve the problem of http image download

### 🚜 Refactor

- Many small improvements
- Use tokio::fs::try_exists
- Many minor modifications
- Change shutdown parament
- Some minor modifications
- _(ciweimao)_ Some minor modifications
- Remove some test code
- Remove the lifetimes of Options
- Change Options field

### 📚 Documentation

- Update README.md

### 🧪 Testing

- Add novels test

### ⚙️ Miscellaneous Tasks

- Update example
- Disable default-features for all crate
- Add cargo-semver-checks install action

## [0.3.0] - 2023-01-30

### ⛰️ Features

- Handle the case that novel does not exist
- Add is_some_and()
- Add home_dir_path()
- Initial

### 🐛 Bug Fixes

- _(ciweimao)_ Check is logged in incorrectly
- _(ciweimao)_ Error in image path parsing
- _(ciweimao)_ Wrong path on windows

### 🚜 Refactor

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

### 🧪 Testing

- Fix failing test on Windows
- Remove test that don't work on CI
- Ignore Keyring test in CI

### ⚙️ Miscellaneous Tasks

- Add check semver version-tag-prefix
- Add aarch64-apple-darwin target
- Update geetest.js
- Add changelog
- Remove outdated action schedule
- Add cargo-semver-checks-action
- Add license allow
- Change prompt
- Remove redundant period
- Install NASM when building on Windows
