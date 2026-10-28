# Mirror
Mirror is a simple command line tool that is used to copy and compare two directories against each other.

## Features
- **File Copying** - Copy files with progress bars to track copying progress
- **Comparison** - Compare two directories and discover their differences in file structure using the `--compare` flag
- **File Filtering** - Add hidden and operating system files and folders to the operations using the `--hidden` and `--system` flags

## Installation
Just install the files in the releases section for your respective operating system, and then you're good to go!

## Usage
To copy files from one directory to another, just use the program with the source and destination directories as the arguments:
```sh
mirror source destination
```

If you just want to copy files from another directory into the current one, omit the destination parameter:
```sh
mirror source
```

To copy over special files, like hidden and operating system files, use the `--hidden` and `--system` flags respectively:
```sh
mirror source destination --hidden --system
```

## Changelog
### 1.0.0
- Implemented core features
- Added the `--hidden` and `--system` flags
- Added the `--compare` flag and comparison features
- Added the `--verbose` flag for more information when using the tool
