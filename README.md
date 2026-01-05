# Git Checkout TUI

A modern, terminal-based user interface for switching git branches, written in Rust.

## Features

- **Interactive TUI**: Clean and responsive terminal interface using `ratatui`.
- **Vim-style Navigation**: Support for `j`/`k` navigation as well as arrow keys.
- **Safe Switching**: Prevents switching if you have unstaged changes that would be overwritten.
- **Visual Feedback**: Highlights the current active branch and provides success/error messages.

## Installation

### Prerequisites

You need to have Rust installed. If you don't have it, install it via rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building from Source

Clone the repository and install using cargo:

```bash
git clone https://github.com/S1933/git-checkout.git
cd git-checkout
cargo install --path .
```

### Using Docker

You can also run the tool using Docker:

```bash
docker build -t git-checkout .
docker run -it --rm -v "$(pwd)":/app git-checkout git-checkout
```

## Usage

Simply run the tool in any git repository:

```bash
git-checkout
```

### Controls

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` | Checkout selected branch |
| `q` / `Esc` | Quit |

## License

MIT
