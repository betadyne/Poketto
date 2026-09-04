# Product Requirement Document (PRD)

## Project: Poketto (Rewrite Pure Rust + Slint)
### 1. Objective & Scope
Membangun ulang visual novel / game library client yang cepat, hemat memori (< 60 MB RAM), instan saat dibuka, dan bebas stutter scroll di Linux (Wayland/X11) maupun Windows, dengan mempertahankan fungsionalitas inti:
 1. Integrasi VNDB API (metadata, search, cover image, characters).
 2. Local Library Management (SQLite berbasis WAL).
 3. Linux Runtime Detection (Wine prefix & Steam Proton runner/classifier).
 4. Discord Rich Presence (status permainan real-time).
 5. Cover Art Virtualized Grid dengan lazy caching.
### 2. Tech Stack & Dependencies (Latest Target)
#### A. Core Engine & UI
 * **Rust Edition:** 2021 / Rust 1.82+
 * **UI Toolkit:** slint = "latest" (Fitur: backend-winit, renderer-femtovg atau renderer-skia, live-preview)
 * **Slint Build:** slint-build = "latest"
 * **Async Runtime:** tokio = { version = "latest", features = ["full"] }
#### B. Storage & Data Management
 * **Database:** rusqlite = { version = "latest", features = ["bundled"] }
 * **Serialization:** serde = { version = "latest", features = ["derive"] }, serde_json = "1.0"
 * **Directories:** directories = "latest" (standard XDG paths di Linux, AppData di Windows)
#### C. Network & External Integrations
 * **HTTP Client:** reqwest = { version = "latest", default-features = false, features = ["rustls-tls", "json", "stream"] }
 * **Discord RPC:** discord-rich-presence = "latest" (atau IPC socket native)
 * **Image Processing (Thumbnail caching):** image = { version = "latest", default-features = false, features = ["png", "jpeg", "webp"] }
### 3. Architecture Design: Multi-Crate Cargo Workspace
Untuk mencegah domain logic bercampur dengan UI, repositori dibagi menjadi Workspace:
```text
poketto/
├── Cargo.toml                  # Workspace definition
├── crates/
│   ├── poketto-core/           # Business Logic, DB, Network, Platform
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── db/             # Rusqlite repositories & migrations
│   │       ├── vndb/           # VNDB HTTPS client & models
│   │       ├── wine/           # Proton & Wine scanner/prefix analyzer
│   │       ├── discord/        # Rich presence background worker
│   │       ├── process/        # Game process launcher & tracker
│   │       └── lib.rs
│   │
│   └── poketto-app/            # Entry point & Slint UI
│       ├── Cargo.toml
│       ├── build.rs            # slint_build::compile("ui/app.slint")
│       ├── ui/
│       │   ├── tokens.slint    # Theme colors, spacing, typography
│       │   ├── components/     # GameCard, SearchBar, Sidebar, TagBadge
│       │   ├── views/          # LibraryView, DetailView, SettingsView
│       │   └── app.slint       # Root Window
│       └── src/
│           ├── main.rs         # Setup runtime & Slint loop
│           ├── state.rs        # App state & background thread bridges
│           ├── adapters/       # Mengonversi Rust models -> Slint ModelData
│           └── image_loader.rs # Async disk cache ke slint::Image

```
### 4. Strategi UI (Anti "Tailwind-to-Slint Hell")
Agar AI atau developer tidak halusinasi mencoba menulis sintaks ala CSS web, semua antarmuka mengadopsi aturan arsitektur berikut:
#### A. Design Tokens (ui/tokens.slint)
Semua ukuran, radius, dan warna didefinisikan secara statis terlebih dahulu:
```slint
export global Theme {
    // Colors
    out property <color> bg-dark: #0f1115;
    out property <color> surface: #181a20;
    out property <color> surface-hover: #222630;
    out property <color> primary: #a277ff;
    out property <color> text-primary: #ededed;
    out property <color> text-muted: #8e94a4;
    out property <color> border: #282c37;

    // Metrics
    out property <length> radius-sm: 6px;
    out property <length> radius-md: 10px;
    out property <length> pad-sm: 8px;
    out property <length> pad-md: 14px;
    out property <length> pad-lg: 20px;
}

```
#### B. Component Decomposition Guidelines
 1. **Dilarang langsung mem-porting JSX:** Dekonstruksi setiap elemen menjadi 3 primitif Slint: HorizontalLayout, VerticalLayout, dan Rectangle (sebagai background container).
 2. **Hover & Event Isolation:** Setiap elemen interaktif harus membungkus atau sejajar dengan TouchArea eksplisit:
   ```slint
   component CustomButton inherits Rectangle {
       in property <string> text;
       callback clicked;
   
       background: touch.has-hover ? Theme.surface-hover : Theme.surface;
       border-radius: Theme.radius-sm;
   
       touch := TouchArea {
           clicked => { root.clicked(); }
       }
       // Content Layout
   }
   
   ```
 3. **Penyajian Data List:** Gunakan ListView bawaan Slint yang dipasangkan dengan std::rc::Rc<slint::VecModel<T>> dari sisi Rust untuk menangani virtualisasi ribuan visual novel secara efisien tanpa re-render DOM.
### 5. Core Subsystem Specifications
#### 5.1. Image & Cover Art Pipeline (image_loader.rs)
 * **Problem:** Memuat gambar beresolusi asli dari VNDB langsung ke memory texture akan memicu lonjakan RAM (OOM) dan micro-stutter pada UI thread.
 * **Solusi:**
   1. Download gambar via reqwest di background worker.
   2. Simpan file mentah di ~/.cache/poketto/covers/.
   3. Lakukan thumbnailing via image crate ke ukuran display maksimum (contoh: lebar 300px, aspect ratio 3:4) dan simpan sebagai WebP/PNG cache.
   4. Dekode ke slint::SharedPixelBuffer<slint::Rgba8Pixel> lalu bungkus ke slint::Image::from_rgba8().
   5. Inject ke Slint UI model lewat slint::invoke_from_event_loop.
 * **Performance Rule:** Main thread must remain strictly non-blocking (<16ms per event). Heavy image decoding and disk I/O must always occur off the Slint UI thread. All image assets for lists/cards must be pre-downscaled to thumbnail dimensions before instantiating slint::Image. Character avatars decode at avatar width (150px, ~2x of the 75px display size); covers decode at 300px.
#### 5.2. Wine / Proton Prefix Scanner (poketto-core::wine)
 * Migrasi modular dari logika lama di src-tauri/src/wine/*.
 * Scan path otomatis:
   * Steam root library (~/.local/share/Steam, ~/.steam/root, library folder sekunder dari libraryfolders.vdf).
   * Lutris prefix, Bottles bottles directory, dan custom wineprefixes (~/.wine).
 * Deteksi binary runner (Proton GE, Proton Experimental, system wine).
#### 5.3. Process Management & Discord RPC
 * **Process Watcher:**
   * Saat game diluncurkan (melalui runner Wine atau native binary), buat subprocess async menggunakan tokio::process::Command.
   * Track PID dan catat waktu bermain (*playtime tracking*) ke SQLite saat proses berakhir (Child::wait).
 * **Discord RPC Bridge:**
   * Background loop yang mengirim event update aktivitas hanya saat status permainan berubah (Playing / Idle di library), mencegah spam IPC socket.
### 6. Roadmap Migrasi Bertahap
```text
Phase 1: Foundation (Workspace & Database)
├── Inisialisasi Cargo workspace & dependency setup (Slint + Tokio)
├── Porting SQLite schema & migration dari repo lama ke `poketto-core/src/db`
└── Unit test query local library

Phase 2: Core Services Porting
├── Porting scanner Wine & Proton (`classify.rs`, `detect.rs`, `steam.rs`)
├── Implementasi VNDB API client (HTTP Client + JSON mappers)
└── Process execution & Discord RPC worker

Phase 3: Slint UI Layer
├── Pembuatan design tokens (`tokens.slint`) & template preview
├── Komponen dasar: `Sidebar`, `TagBadge`, `GameCard`
├── Layar utama: Grid View dengan model data `VecModel<GameCardData>`
├── Integrasi async image loader (disk -> memory texture)
└── Layar Game Detail, Settings, & Wine Configuration Overlay

Phase 4: Release & Packaging
├── Windows target (MSVC, single `.exe`)
├── Linux packaging (Native binary, Flatpak / AppImage manifest)
└── Benchmark resource usage (< 50 MB RAM, 60+ FPS scroll Wayland)

```
