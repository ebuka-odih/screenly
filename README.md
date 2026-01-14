# 📽️ Screen Snap

**Screen Snap** is a high-performance, premium screen recording application built for macOS (and coming to other platforms). It combines a sleek, modern glassmorphic interface with powerful native features to make content creation effortless.

![Features](https://img.shields.io/badge/features-Native_Performance-blueviolet)
![Tech](https://img.shields.io/badge/tech-Svelte--Tauri--Rust-orange)

---

## ✨ Key Features

-   **Native macOS Performance**: Leveraging Tauri and Rust for low-latency, high-quality recording.
-   **Draggable Face Cam**: Overlay your camera anywhere on the screen with smooth mirroring and glass borders.
-   **Audio Mixing**: Record from your microphone, system audio, or both simultaneously.
-   **Intelligent Mouse Zoom**: Automatically highlights and "zooms" into your mouse clicks during playback, perfect for tutorials and demonstrations.
-   **Glassmorphism UI**: A stunning, responsive interface designed with modern aesthetics (blur effects, gradients, and micro-animations).
-   **Instant Export**: Review your recording immediately with built-in playback and easy export to standard formats.

---

## 🚀 Getting Started

### Prerequisites

-   **Node.js** (v18 or higher)
-   **Rust** (Installed via [rustup](https://rustup.rs/))
-   **macOS** (Currently optimized for macOS features like Continuity Camera)

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/yourusername/screen-snap.git
    cd screen-snap
    ```

2.  **Install dependencies:**
    ```bash
    npm install
    ```

3.  **Run in Development Mode:**
    ```bash
    npm run tauri dev
    ```

---

## 🛠️ Tech Stack

-   **Frontend**: [SvelteKit](https://kit.svelte.dev/) & Vite
-   **Backend**: [Tauri v2.0](https://tauri.app/) & Rust
-   **OS Integration**: `rdev` for global event tracking & AVFoundation (planned).
-   **Styling**: Pure Vanilla CSS with a focus on rich aesthetics and performance.

---

## 🤝 Contributing & Forking

This project is **open-source** and built for the community! We love contributions, whether they are bug fixes, new features, or design improvements.

### How to Contribute:

1.  **Fork** the repository.
2.  **Create a branch** for your feature: `git checkout -b feature/amazing-feature`.
3.  **Commit your changes**: `git commit -m 'Add some amazing feature'`.
4.  **Push to the branch**: `git push origin feature/amazing-feature`.
5.  **Open a Pull Request**.

We welcome everyone from beginners to experienced Rust/Svelte developers to jump in and help make Screen Snap the best screen recording tool available!

---

*Made with ❤️ by the Screen Snap team.*
