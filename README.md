# Toxcord

A modern, privacy-focused Tox client built with Tauri and Rust.

## Features

- **P2P Messaging** - Direct peer-to-peer encrypted communication via the Tox protocol
- **Voice/Video Calls** - ToxAV support for audio and video calls
- **Group Chats** - Tox group conference support
- **Proxy Support** - Route traffic through SOCKS5 or HTTP proxies
- **I2P Integration** - Optional embedded I2P router for enhanced privacy (hides your IP from peers)
- **Cross-Platform** - Linux support (Windows/macOS planned)

## Downloads

Download pre-built binaries from the [Releases](https://github.com/D4M13N-D3V/Toxcord/releases) page.

### Linux

| File | Description |
|------|-------------|
| `toxcord-*-linux-x86_64-portable.tar.gz` | Portable binary with bundled libraries (recommended) |
| `toxcord-*-linux-x86_64.AppImage` | AppImage package |
| `toxcord-*-linux-x86_64.deb` | Debian/Ubuntu package |
| `toxcord-i2p-*-portable.tar.gz` | Portable binary with embedded I2P support |

#### Running the Portable Version

```bash
tar -xzf toxcord-*-portable.tar.gz
cd toxcord-*-portable
./run.sh
```

## Building from Source

### Prerequisites

**Rust toolchain:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**System dependencies (Ubuntu/Debian):**
```bash
sudo apt-get install build-essential cmake pkg-config \
  libsodium-dev libopus-dev libvpx-dev \
  libgtk-3-dev libwebkit2gtk-4.1-dev \
  libappindicator3-dev librsvg2-dev libasound2-dev
```

**System dependencies (Arch Linux):**
```bash
sudo pacman -S base-devel cmake pkg-config \
  libsodium opus libvpx \
  gtk3 webkit2gtk libappindicator-gtk3 librsvg alsa-lib
```

**Node.js and pnpm:**
```bash
# Install Node.js 20+
# Then install pnpm:
npm install -g pnpm
```

### Build

```bash
# Clone with submodules
git clone --recursive https://github.com/D4M13N-D3V/Toxcord.git
cd Toxcord

# Install frontend dependencies
pnpm install

# Install Tauri CLI
cargo install tauri-cli

# Build the application
cd apps/desktop
cargo tauri build
```

The built binary will be in `target/release/toxcord`.

### Build with I2P Support

```bash
cd apps/desktop
cargo tauri build --features i2p
```

## Video Calling

Toxcord supports 1-on-1 video calls via ToxAV. Video is captured from your webcam, converted to YUV420, and transmitted peer-to-peer.

### Linux Camera Requirements

On Linux, webcam support requires the **uvcvideo** kernel module. Most distributions load this automatically when a USB webcam is plugged in.

**If your camera isn't detected:**

1. **Check if the webcam is connected:**
   ```bash
   lsusb | grep -i cam
   ```

2. **Check for video devices:**
   ```bash
   ls /dev/video*
   ```

3. **If webcam is connected but no /dev/video* exists:**

   The app will show an "Enable Camera" button in the device settings. Click it to load the driver (requires password).

   Or manually:
   ```bash
   sudo modprobe uvcvideo
   ```

4. **If modprobe says "Module not found":**

   Your kernel modules may be out of sync. This happens after kernel updates. **Reboot your system** to use the new kernel with the correct modules.

   Check current vs installed kernel:
   ```bash
   uname -r              # Running kernel
   ls /lib/modules/      # Installed modules
   ```

5. **Verify UVC support is enabled in your kernel:**
   ```bash
   zcat /proc/config.gz | grep CONFIG_USB_VIDEO_CLASS
   # Should show: CONFIG_USB_VIDEO_CLASS=m or CONFIG_USB_VIDEO_CLASS=y
   ```

### Permissions

Ensure your user is in the `video` group:
```bash
sudo usermod -aG video $USER
# Log out and back in for changes to take effect
```

## Proxy Configuration

Toxcord supports routing Tox traffic through SOCKS5 or HTTP proxies. This can be used with:
- I2P (via emissary or i2pd)
- Tor
- VPN SOCKS proxies
- Any standard proxy server

### Environment Variables

```bash
# Proxy type: "socks5", "http", or "none"
export TOXCORD_PROXY_TYPE=socks5

# Proxy server address
export TOXCORD_PROXY_HOST=127.0.0.1

# Proxy server port
export TOXCORD_PROXY_PORT=4447
```

### Using with I2P

If built with the `i2p` feature, Toxcord embeds an I2P router that starts automatically. The embedded router provides a SOCKS5 proxy that Tox traffic is routed through.

**Note:** I2P adds significant latency (~2-5 seconds per hop). For best results, use with I2P-native Tox bootstrap nodes if available.

## Project Structure

```
Toxcord/
├── apps/
│   └── desktop/          # Tauri desktop application
│       └── src-tauri/    # Rust backend
├── crates/
│   ├── toxcord-tox/      # High-level Tox wrapper
│   ├── toxcord-tox-sys/  # c-toxcore FFI bindings
│   └── toxcord-protocol/ # Protocol definitions
└── packages/             # Shared frontend packages
```

## Architecture

Toxcord uses:
- **[Tauri](https://tauri.app/)** - Lightweight desktop app framework
- **[c-toxcore](https://github.com/TokTok/c-toxcore)** - The Tox protocol implementation (vendored)
- **[emissary](https://github.com/eepnet/emissary)** - Embedded I2P router (optional)

## Privacy

Toxcord inherits the privacy properties of the Tox protocol:
- End-to-end encryption for all messages
- No central servers - fully peer-to-peer
- Public key-based identity (Tox ID)

With I2P or proxy support enabled, your IP address is hidden from Tox peers and bootstrap nodes.

## License

MIT

## Contributing

Contributions are welcome! Please open an issue or pull request.
