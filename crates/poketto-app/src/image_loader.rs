use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

pub const THUMB_WIDTH: u32 = 300;
const JPEG_QUALITY: u8 = 85;
const MAX_CONCURRENT_DOWNLOADS: usize = 6;
const MAX_MEMORY_ENTRIES: usize = 300;
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct LoadedCover {
    pub game_id: String,
    pub generation: u64,
    pub image: Option<DecodedImage>,
}

pub fn cache_key(url: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in url.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

pub fn thumbnail_path(cache_dir: &Path, url: &str) -> PathBuf {
    cache_dir.join(format!("{}.jpg", cache_key(url)))
}

pub fn decode_thumbnail(bytes: &[u8], max_width: u32) -> Option<DecodedImage> {
    let image = image::load_from_memory(bytes).ok()?;
    let rgba = if image.width() <= max_width {
        image.to_rgba8()
    } else {
        image.thumbnail(max_width, u32::MAX).to_rgba8()
    };
    let (width, height) = (rgba.width(), rgba.height());
    Some(DecodedImage {
        pixels: rgba.into_raw(),
        width,
        height,
    })
}

fn encode_jpeg(image: &DecodedImage) -> Option<Vec<u8>> {
    let rgb: Vec<u8> = image
        .pixels
        .chunks_exact(4)
        .flat_map(|pixel| [pixel[0], pixel[1], pixel[2]])
        .collect();
    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, JPEG_QUALITY);
    encoder
        .encode(&rgb, image.width, image.height, image::ExtendedColorType::Rgb8)
        .ok()?;
    drop(encoder);
    Some(buf)
}

struct MemCache {
    map: HashMap<String, DecodedImage>,
    order: VecDeque<String>,
    cap: usize,
}

impl MemCache {
    fn new(cap: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            cap,
        }
    }

    fn get(&mut self, key: &str) -> Option<DecodedImage> {
        self.map.get(key).cloned()
    }

    fn put(&mut self, key: String, image: DecodedImage) {
        if !self.map.contains_key(&key) {
            while self.map.len() >= self.cap {
                if let Some(old) = self.order.pop_front() {
                    self.map.remove(&old);
                } else {
                    break;
                }
            }
            self.order.push_back(key.clone());
        }
        self.map.insert(key, image);
    }

    fn remove(&mut self, key: &str) {
        if self.map.remove(key).is_some() {
            self.order.retain(|key_| key_ != key);
        }
    }
}

struct Shared {
    rt: tokio::runtime::Handle,
    client: reqwest::Client,
    cache_dir: PathBuf,
    mem: Mutex<MemCache>,
    inflight: Mutex<HashSet<String>>,
    generation: AtomicU64,
    done_tx: std::sync::mpsc::Sender<LoadedCover>,
    downloads: tokio::sync::Semaphore,
}

pub struct ImageLoader {
    shared: Arc<Shared>,
    done_rx: std::sync::Mutex<std::sync::mpsc::Receiver<LoadedCover>>,
}

impl ImageLoader {
    pub fn new(
        rt: &tokio::runtime::Handle,
        cache_dir: PathBuf,
    ) -> std::io::Result<Self> {
        std::fs::create_dir_all(&cache_dir)?;
        let client = reqwest::Client::builder()
            .user_agent("Poketto/0.1.0")
            .timeout(DOWNLOAD_TIMEOUT)
            .build()
            .map_err(std::io::Error::other)?;
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        Ok(Self {
            shared: Arc::new(Shared {
                rt: rt.clone(),
                client,
                cache_dir,
                mem: Mutex::new(MemCache::new(MAX_MEMORY_ENTRIES)),
                inflight: Mutex::new(HashSet::new()),
                generation: AtomicU64::new(0),
                done_tx,
                downloads: tokio::sync::Semaphore::new(MAX_CONCURRENT_DOWNLOADS),
            }),
            done_rx: Mutex::new(done_rx),
        })
    }

    pub fn next_generation(&self) -> u64 {
        self.shared.generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn request(&self, game_id: &str, url: &str) {
        let key = cache_key(url);
        let generation = self.shared.generation.load(Ordering::SeqCst);
        if let Some(image) = self.shared.mem.lock().expect("mem cache").get(&key) {
            let _ = self.shared.done_tx.send(LoadedCover {
                game_id: game_id.to_string(),
                generation,
                image: Some(image),
            });
            return;
        }
        if !self
            .shared
            .inflight
            .lock()
            .expect("inflight set")
            .insert(game_id.to_string())
        {
            return;
        }
        let shared = self.shared.clone();
        let game_id = game_id.to_string();
        let url = url.to_string();
        self.shared.rt.spawn(async move {
            let image = load_one(&shared, &url).await;
            if let Some(image) = &image {
                shared
                    .mem
                    .lock()
                    .expect("mem cache")
                    .put(key, image.clone());
            }
            shared
                .inflight
                .lock()
                .expect("inflight set")
                .remove(&game_id);
            let _ = shared.done_tx.send(LoadedCover {
                game_id,
                generation,
                image,
            });
        });
    }

    pub fn evict_url(&self, url: &str) {
        self.shared.mem.lock().expect("mem cache").remove(&cache_key(url));
        let _ = std::fs::remove_file(thumbnail_path(&self.shared.cache_dir, url));
    }

    pub fn poll(&self) -> Vec<LoadedCover> {
        let current = self.shared.generation.load(Ordering::SeqCst);
        let mut loaded = Vec::new();
        let rx = self.done_rx.lock().expect("completions");
        while let Ok(cover) = rx.try_recv() {
            if cover.generation == current {
                loaded.push(cover);
            }
        }
        loaded
    }
}

async fn load_one(shared: &Shared, url: &str) -> Option<DecodedImage> {
    let path = thumbnail_path(&shared.cache_dir, url);
    if let Ok(bytes) = tokio::fs::read(&path).await {
        if let Some(image) = decode_thumbnail(&bytes, THUMB_WIDTH) {
            return Some(image);
        }
    }
    let _permit = shared.downloads.acquire().await.ok()?;
    let bytes = shared
        .client
        .get(url)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .bytes()
        .await
        .ok()?;
    let image = decode_thumbnail(&bytes, THUMB_WIDTH)?;
    if let Some(encoded) = encode_jpeg(&image) {
        let _ = tokio::fs::write(&path, &encoded).await;
    }
    Some(image)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_is_stable_hex() {
        let first = cache_key("https://example.com/a.jpg");
        assert_eq!(first, cache_key("https://example.com/a.jpg"));
        assert_eq!(first.len(), 16);
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(first, cache_key("https://example.com/b.jpg"));
    }

    #[test]
    fn thumbnail_path_is_safe_jpeg() {
        let path = thumbnail_path(Path::new("/cache"), "https://example.com/a/b.png?x=1");
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("jpg"));
        let stem = path.file_stem().and_then(|s| s.to_str()).expect("stem");
        assert!(stem.chars().all(|c| c.is_ascii_hexdigit()));
    }

    fn test_png(width: u32, height: u32) -> Vec<u8> {
        let image = image::RgbImage::from_fn(width, height, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        });
        let mut buf = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buf);
        image::DynamicImage::ImageRgb8(image)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .expect("encode");
        drop(cursor);
        buf
    }

    #[test]
    fn large_image_downscales_to_thumb_width() {
        let decoded = decode_thumbnail(&test_png(1200, 1600), THUMB_WIDTH).expect("decode");
        assert_eq!(decoded.width, 300);
        assert_eq!(decoded.height, 400);
        assert_eq!(decoded.pixels.len(), 300 * 400 * 4);
    }

    #[test]
    fn small_image_never_upscales() {
        let decoded = decode_thumbnail(&test_png(100, 100), THUMB_WIDTH).expect("decode");
        assert_eq!(decoded.width, 100);
        assert_eq!(decoded.height, 100);
    }

    #[test]
    fn corrupt_bytes_decode_to_none() {
        assert_eq!(decode_thumbnail(b"not an image", THUMB_WIDTH).is_none(), true);
        assert_eq!(decode_thumbnail(&[], THUMB_WIDTH).is_none(), true);
    }

    #[test]
    fn jpeg_round_trip_preserves_dimensions() {
        let decoded = decode_thumbnail(&test_png(600, 800), THUMB_WIDTH).expect("decode");
        let encoded = encode_jpeg(&decoded).expect("encode");
        let again = decode_thumbnail(&encoded, THUMB_WIDTH).expect("re-decode");
        assert_eq!((again.width, again.height), (decoded.width, decoded.height));
    }

    #[test]
    fn memory_cache_evicts_oldest() {
        let mut mem = MemCache::new(2);
        let image = decode_thumbnail(&test_png(10, 10), THUMB_WIDTH).expect("decode");
        mem.put("a".to_string(), image.clone());
        mem.put("b".to_string(), image.clone());
        mem.put("c".to_string(), image);
        assert_eq!(mem.get("a").is_none(), true);
        assert_eq!(mem.get("b").is_some(), true);
        assert_eq!(mem.get("c").is_some(), true);
    }

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("poketto-loader-test")
            .join(format!("{}-{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dir");
        dir
    }

    fn test_loader(dir: &Path) -> (tokio::runtime::Runtime, ImageLoader) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let loader = ImageLoader::new(rt.handle(), dir.to_path_buf()).expect("loader");
        (rt, loader)
    }

    #[test]
    fn seeded_disk_cache_serves_without_network() {
        let dir = test_dir("disk");
        let url = "https://example.com/cover.jpg";
        let decoded = decode_thumbnail(&test_png(600, 900), THUMB_WIDTH).expect("decode");
        std::fs::write(
            thumbnail_path(&dir, url),
            encode_jpeg(&decoded).expect("encode"),
        )
        .expect("seed");
        let (rt, loader) = test_loader(&dir);
        rt.block_on(async {
            loader.request("g1", url);
            let loaded = wait_for_cover(&loader, "g1").await.expect("loaded");
            assert_eq!(loaded.image.expect("image").width, 300);
        });
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unreachable_url_completes_without_image() {
        let dir = test_dir("unreachable");
        let (rt, loader) = test_loader(&dir);
        rt.block_on(async {
            loader.request("g1", "http://127.0.0.1:9/cover.jpg");
            let loaded = wait_for_cover(&loader, "g1").await.expect("loaded");
            assert_eq!(loaded.image.is_none(), true);
        });
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stale_generation_is_filtered() {
        let dir = test_dir("generation");
        let (rt, loader) = test_loader(&dir);
        rt.block_on(async {
            loader.request("g1", "http://127.0.0.1:9/cover.jpg");
            tokio::time::sleep(Duration::from_millis(500)).await;
            loader.next_generation();
            assert_eq!(loader.poll().len(), 0);
        });
        let _ = std::fs::remove_dir_all(&dir);
    }

    async fn wait_for_cover(loader: &ImageLoader, game_id: &str) -> Option<LoadedCover> {
        for _ in 0..100 {
            for cover in loader.poll() {
                if cover.game_id == game_id {
                    return Some(cover);
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        None
    }

    #[test]
    fn evict_url_clears_disk_and_memory() {
        let dir = test_dir("evict");
        let url = "https://example.com/evict.jpg";
        let decoded = decode_thumbnail(&test_png(100, 100), THUMB_WIDTH).expect("decode");
        std::fs::write(
            thumbnail_path(&dir, url),
            encode_jpeg(&decoded).expect("encode"),
        )
        .expect("seed");
        let (_rt, loader) = test_loader(&dir);
        loader
            .shared
            .mem
            .lock()
            .expect("mem cache")
            .put(cache_key(url), decoded);
        loader.evict_url(url);
        assert!(!thumbnail_path(&dir, url).exists());
        assert!(loader
            .shared
            .mem
            .lock()
            .expect("mem cache")
            .get(&cache_key(url))
            .is_none());
        loader.evict_url(url);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
