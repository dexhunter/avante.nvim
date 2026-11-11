//! Peak Rust heap of `do_html2md` on one script-heavy HTML page.
//! Run from the repo root:
//!   cargo run --quiet --release --manifest-path osspr-intake/harness/html2md_peak/Cargo.toml
//! Prints the peak live bytes allocated during that call.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

include!("../../../../crates/avante-html2md/src/lib.rs");

struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

fn note_grow(additional: usize) {
    let previous = LIVE.fetch_add(additional, Ordering::Relaxed);
    let current = previous + additional;
    let mut peak = PEAK.load(Ordering::Relaxed);
    while current > peak {
        match PEAK.compare_exchange_weak(peak, current, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(observed) => peak = observed,
        }
    }
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            note_grow(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            if new_size > layout.size() {
                note_grow(new_size - layout.size());
            } else {
                LIVE.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        new_ptr
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn reset() {
    LIVE.store(0, Ordering::Relaxed);
    PEAK.store(0, Ordering::Relaxed);
}

/// A documentation page shaped like the HTML `do_fetch_md` hands to `do_html2md`:
/// a short article plus a large inline script, stylesheet, header, and footer.
/// `skip_tags` already drops those elements from the markdown.
fn page() -> String {
    const SCRIPT_BYTES: usize = 500_000;
    let mut html = String::from(
        "<html><head><style>body{color:#222;font:16px sans-serif}</style><script>",
    );
    html.push_str(&"var x=1;".repeat(SCRIPT_BYTES / 8));
    html.push_str(
        "</script></head><body><header>Site nav</header><article><h1>Title</h1><p>Hello <b>world</b>.</p></article><footer>Site footer</footer></body></html>",
    );
    html
}

fn peak_bytes(html: &str) -> usize {
    reset();
    let markdown = do_html2md(html).expect("html2md");
    if markdown != "# Title\n\nHello **world**." {
        eprintln!("unexpected markdown: {markdown:?}");
        std::process::exit(1);
    }
    PEAK.load(Ordering::Relaxed)
}

fn main() {
    let html = page();
    let _ = peak_bytes(&html);
    println!("{}", peak_bytes(&html));
}
