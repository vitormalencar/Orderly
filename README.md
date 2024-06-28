# Orderly 🧹

![Orderly Logo](logo.png)

Orderly is a command-line tool written in Rust for automating the organization of files on your macOS system. Inspired by Hazel, Orderly allows users to define rules and actions to keep their files neat and tidy 🧹

## Features

- Rule-based file organization
- Support for various file actions (move, copy, rename, delete)
- Condition-based rules (file type, date, size, etc.)
- Easy to use CLI interface

## Installation

To install Orderly, ensure you have Rust installed on your system. Then run the following command:

```bash
cargo install orderly
```

## Usage

Orderly uses a configuration file to define rules. Here's a basic example of how to set it up:

1. Initialize Orderly in a directory:

```bash
orderly init
```

2. Create a new rule file in the `rules` directory:

```bash
touch rules/example.yaml
```

3. Open the `example.yaml` file and add the following content:

```yaml
name: Example Rule
description: An example rule for organizing files
actions:
  - name: Move to Trash
    path: ~/.Trash
    condition:
      type: file
      size: 100MB
    action:
      type: move
      path: ~/.Trash
```

4. Run Orderly to apply the rule:

```bash
orderly run
```

## Configuration

The `orderly.yaml` file supports various conditions and actions:

### Conditions

- **extension**: Match files by their extension
- **size**: Match files by their size
- **date**: Match files by their creation or modification date
- **name**: Match files by their name pattern

### Actions

- **move**: Move files to a specified directory
- **copy**: Copy files to a specified directory
- **rename**: Rename files using a pattern
- **delete**: Delete files

## Contributing

We welcome contributions to Orderly! To contribute, follow these steps:

1. Fork the repository
2. Create a new branch (`git checkout -b feature/your-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin feature/your-feature`)
5. Create a new Pull Request

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgements

Orderly is inspired by [Hazel](https://www.noodlesoft.com/), a similar tool for macOS.
