# embedded-rgba

A lightweight, no_std RGBA framebuffer and canvas abstraction for the [`embedded-graphics`](https://github.com/embedded-graphics/embedded-graphics) ecosystem. It adds **alpha blending**, **buffering strategies**, and a practical way to manage drawing pipelines on microcontrollers and resource‑constrained devices.

---

## ✨ Features

- ✅ **Alpha blending** – draw `Rgba` pixels onto RGB framebuffers with per‑pixel transparency.
- ✅ **Flexible buffering** – choose between **double buffering**, **single buffering**, or **scanline buffering (coming soon)** depending on memory and performance tradeoffs.
- ✅ **Drop‑in integration** with `embedded-graphics`’s `DrawTarget` and `PixelColor`.
- ✅ **No heap allocation** – designed for MCUs without a heap.
- ✅ **Optimized for speed** – fast fill paths for solid colors and contiguous data.

---

## 🚀 Usage

### Create a double‑buffered canvas

```rust
use embedded_rgba::Canvas;
use embedded_graphics::mock_display::MockDisplay;
use embedded_graphics::pixelcolor::Rgb565;

let display = MockDisplay::new();

// 240x320 RGB565 framebuffer with double buffering
let mut canvas = Canvas::<_, Rgb565, _, 240, 320>::double_buffered(display);
```

### Draw with alpha

```rust
let mut alpha = canvas.alpha();
alpha.fill_solid(&Rectangle::new(Point::zero(), Size::new(50, 50)), Rgba::new(Rgb565::RED, 128));
canvas.flush().unwrap();
```

### Single buffer

```rust
let mut canvas = Canvas::<_, Rgb565, _, 240, 320>::single_buffered(display);
```

---

## 📊 When to use which buffer?

- **Double buffer** → flicker‑free updates, at the cost of RAM (2 full framebuffers).
- **Single buffer** → less RAM, may tear if drawn while refreshing.
- **Line buffer** → minimal RAM (1 row), best for preplanned scanline rendering.

---

## 🔮 Roadmap

- [ ] SIMD‑style blending optimizations
- [ ] Region‑based dirty rectangle updates
- [ ] More color formats

---

## 🤝 Contributing

Contributions, bug reports, and feature requests are welcome! Open an issue or PR on GitHub.

---
