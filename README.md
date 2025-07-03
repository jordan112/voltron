# Voltron Terminal

A fast, modern 4-panel terminal emulator built with Tauri and Rust, featuring GPU-accelerated rendering. Perfect for developers who need multiple terminal sessions visible at once.

![image](https://github.com/user-attachments/assets/af4c78b3-84a8-47d9-a0bb-bb6b355e4fcc)

## Features

- 🚀 **Fast Performance**: Native Rust backend with portable-pty for efficient terminal emulation
- 🎨 **GPU Acceleration**: WebGL-accelerated rendering via xterm.js
- 📑 **4-Panel Layout**: Four independent terminal sessions in a grid layout
- 🎯 **Smart Focus**: Click any panel to focus and start typing
- 🖥️ **Cross-Platform**: Built with Tauri for macOS (Windows/Linux support coming)
- ⚡ **Lightweight**: ~600KB base app size thanks to native webview

## Prerequisites

- **macOS 10.15+** (Catalina or later)
- **Rust** (latest stable version)
- **Node.js 18+** and **pnpm**

## Installation

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/voltron.git
   cd voltron
   ```

3. **Install dependencies**:
   ```bash
   pnpm install
   ```

## Running the App

### Quick Start

Use the provided startup script:

```bash
./start.sh
```

### Manual Start

If you prefer to run manually:

```bash
# Ensure Rust is in your PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Run in development mode
pnpm run tauri dev
```

The app will:
1. Start the Vite development server
2. Compile the Rust backend
3. Launch the Voltron terminal window

## Development

### Project Structure

```
voltron/
├── src/                    # Frontend (Svelte/TypeScript)
│   ├── lib/               # Components and stores
│   │   ├── components/    # UI components
│   │   └── stores/        # State management
│   └── routes/            # SvelteKit routes
├── src-tauri/             # Backend (Rust)
│   ├── src/
│   │   ├── main.rs       # Entry point
│   │   └── terminal.rs   # PTY management
│   ├── Cargo.toml        # Rust dependencies
│   └── tauri.conf.json   # Tauri configuration
└── package.json           # Frontend dependencies
```

### Key Technologies

- **Backend**: Rust + Tauri + portable-pty
- **Frontend**: SvelteKit + TypeScript + xterm.js
- **Build**: Vite + pnpm

### Building for Production

```bash
pnpm run tauri build
```

This will create a production build in `src-tauri/target/release/bundle/`.

## Configuration

### Terminal Settings

The terminal uses your system's default shell (typically `bash` on macOS) and inherits your environment variables.

### Customization

You can customize the terminal appearance by modifying the theme in `src/lib/components/Terminal.svelte`:

```javascript
theme: {
  background: '#1e1e1e',
  foreground: '#d4d4d4',
  // ... other color settings
}
```

## Troubleshooting

### "Rust is not installed or not in PATH"
Make sure Rust is properly installed and added to your PATH:
```bash
source "$HOME/.cargo/env"
rustc --version
```

### Port 5173 is already in use
The startup script automatically kills processes on port 5173. If you still have issues:
```bash
lsof -ti:5173 | xargs kill -9
```

### Icons missing error
The app currently uses placeholder icons. To add proper icons:
1. Create your app icon (1024x1024 recommended)
2. Use a tool like [Tauri Icon Generator](https://tauri.app/v1/guides/features/icons/) to generate all required sizes
3. Place them in `src-tauri/icons/`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Your chosen license]

## Acknowledgments

- Built with [Tauri](https://tauri.app/)
- Terminal rendering by [xterm.js](https://xtermjs.org/)
- PTY management via [portable-pty](https://github.com/wez/wezterm/tree/main/pty)
