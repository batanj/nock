# Nock

**Nock** is a lightweight, high-performance, compositor-agnostic Wayland desktop shell toolkit built in Rust. It serves as a high-performance alternative to tools like Quickshell and AGS, combining absolute design flexibility at the authoring layer (Flutter-like widgets in Lua) with microscopic idle CPU and RAM usage.

---

## Philosophy & Key Features

* **Compositor-Agnostic Core:** Relies on capability-detection protocols (`wlr-layer-shell-unstable-v1`, `ext-session-lock-v1`, etc.) rather than hardcoding specific compositor names.
* **Opt-in Frame Budgets:** Static UI components sleep entirely in `epoll` (~0% CPU usage), and animation costs are strictly opt-in per surface.
* **Flutter-Like Widget Model:** Parent-to-child immutable config widgets separating user configuration, Rust host primitives, and Lua composition functions.
* **Zero CSS:** Clean, closed style structures (`Color`, `Edges`, `SizeHint`, `Radius`, etc.) with zero selectors, media queries, or z-index soup.

---

## Tech Stack

* **Language:** Rust (Edition 2021, `rustc` >= 1.80)
* **Canvas:** `skia-safe` (with features: `gl`, `egl`, `wayland`, `textlayout`, `binary-cache`)
* **Backend:** OpenGL via EGL (`wayland-egl` + `khronos-egl`)
* **Event Loop:** `calloop` + `calloop-wayland-source`
* **Wayland Protocol:** `smithay-client-toolkit` (0.21+)
* **Scripting:** `mlua` with vendored Lua 5.4
* **DBus Integration:** `zbus` (for SNI, MPRIS, UPower, Notifications)

---

## System Dependencies

To build and run Nock, ensure you have the following system packages installed:

* `libwayland-dev`
* `libegl1-mesa-dev`
* `libxkbcommon-dev`
* `pkg-config`

---

## Getting Started

Clone the repository and build the project using Cargo:

```bash
git clone [https://github.com/batanj/nock.git](https://github.com/batanj/nock.git)
cd nock
cargo build --release
