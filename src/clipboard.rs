//! Clipboard utilities with HTML-to-Markdown conversion support

#[cfg(not(target_os = "android"))]
use clipboard_rs::{Clipboard as ClipboardTrait, ClipboardContext, ContentFormat};
#[cfg(not(target_os = "android"))]
use std::sync::{Mutex, OnceLock};
use htmd::{Element, HtmlToMarkdown, element_handler::Handlers, options::{BulletListMarker, Options}};

pub type ClipboardResult<T> = Result<T, ClipboardError>;

#[derive(Debug)]
#[allow(dead_code)] // ContextCreation/ReadError are unused on platforms without a system clipboard
pub enum ClipboardError {
    ContextCreation(String),
    ReadError(String),
    ConversionError(String),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContextCreation(e) => write!(f, "Failed to create clipboard context: {}", e),
            Self::ReadError(e) => write!(f, "Failed to read clipboard: {}", e),
            Self::ConversionError(e) => write!(f, "Failed to convert HTML: {}", e),
        }
    }
}

pub enum ClipboardContent {
    Markdown(String),
    PlainText(String),
    Empty,
}

/// The one, lazily-created, process-wide clipboard context.
///
/// `clipboard-rs`'s X11 backend spawns a background thread per context that
/// holds the CLIPBOARD selection so its contents survive. Creating a fresh
/// context for every copy/paste therefore leaked a thread each time and, worse,
/// each new context stole selection ownership from the previous one — which
/// made the now-orphaned thread print "Somebody else owns the clipboard now"
/// to stdout and corrupt the TUI. Reusing one long-lived context keeps a single
/// owner, so re-copying never triggers a self-inflicted `SelectionClear`.
///
/// Returns `None` if a backend could not be created (e.g. no display server).
/// Kept non-generic so there is exactly one `CTX` regardless of caller.
#[cfg(not(target_os = "android"))]
fn clipboard_context() -> Option<&'static Mutex<ClipboardContext>> {
    static CTX: OnceLock<Option<Mutex<ClipboardContext>>> = OnceLock::new();
    CTX.get_or_init(|| ClipboardContext::new().ok().map(Mutex::new))
        .as_ref()
}

/// Run `f` against the shared clipboard context, or return `None` if no
/// clipboard backend is available or the lock is poisoned.
#[cfg(not(target_os = "android"))]
fn with_clipboard<T>(f: impl FnOnce(&ClipboardContext) -> T) -> Option<T> {
    let guard = clipboard_context()?.lock().ok()?;
    Some(f(&guard))
}

/// Write plain text to the system clipboard.
///
/// No-op on platforms without a system clipboard backend (e.g. Android/Termux),
/// where the editor relies on its internal clipboard instead. Failures are
/// swallowed silently — never logged to stdout/stderr, which would corrupt the
/// TUI.
#[cfg(not(target_os = "android"))]
pub fn set_system_text(text: &str) {
    let _ = with_clipboard(|ctx| ctx.set_text(text.to_string()));
}

#[cfg(target_os = "android")]
pub fn set_system_text(_text: &str) {}

/// Read plain text from the system clipboard, or `None` if unavailable.
#[cfg(not(target_os = "android"))]
pub fn get_system_text() -> Option<String> {
    with_clipboard(|ctx| ctx.get_text().ok()).flatten()
}

#[cfg(target_os = "android")]
pub fn get_system_text() -> Option<String> {
    None
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
pub fn has_html() -> bool {
    with_clipboard(|ctx| ctx.has(ContentFormat::Html)).unwrap_or(false)
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
pub fn has_html() -> bool {
    false
}

#[cfg(not(target_os = "android"))]
pub fn get_html() -> ClipboardResult<Option<String>> {
    with_clipboard(|ctx| {
        if !ctx.has(ContentFormat::Html) {
            return Ok(None);
        }
        ctx.get_html()
            .map(Some)
            .map_err(|e| ClipboardError::ReadError(e.to_string()))
    })
    .unwrap_or_else(|| Err(ClipboardError::ContextCreation("clipboard unavailable".to_string())))
}

#[cfg(target_os = "android")]
pub fn get_html() -> ClipboardResult<Option<String>> {
    Ok(None)
}

#[cfg(not(target_os = "android"))]
pub fn get_text() -> ClipboardResult<Option<String>> {
    with_clipboard(|ctx| {
        ctx.get_text()
            .map(Some)
            .map_err(|e| ClipboardError::ReadError(e.to_string()))
    })
    .unwrap_or_else(|| Err(ClipboardError::ContextCreation("clipboard unavailable".to_string())))
}

#[cfg(target_os = "android")]
pub fn get_text() -> ClipboardResult<Option<String>> {
    Ok(None)
}

fn create_converter() -> HtmlToMarkdown {
    let options = Options {
        bullet_list_marker: BulletListMarker::Dash,
        ..Options::default()
    };

    HtmlToMarkdown::builder()
        .options(options)
        .add_handler(vec!["a"], |handlers: &dyn Handlers, element: Element| {
            let mut href: Option<String> = None;
            for attr in element.attrs.iter() {
                if &*attr.name.local == "href" {
                    href = Some(attr.value.to_string());
                    break;
                }
            }

            let href = match href {
                Some(h) if !h.is_empty() => h,
                _ => return Some(handlers.walk_children(element.node)),
            };

            if href.starts_with('#') {
                return Some(handlers.walk_children(element.node));
            }

            let content = handlers.walk_children(element.node).content;
            let text = content.trim();

            if text.is_empty() {
                return None;
            }

            // Escape parentheses in URL
            let href = href.replace('(', "\\(").replace(')', "\\)");

            Some(format!("[{}]({})", text, href).into())
        })
        .build()
}

/// Convert HTML to Markdown using htmd with custom link handling
pub fn html_to_markdown(html: &str) -> ClipboardResult<String> {
    let converter = create_converter();
    converter
        .convert(html)
        .map_err(|e| ClipboardError::ConversionError(e.to_string()))
}

/// Get clipboard content, converting HTML to Markdown if available
///
/// Priority:
/// 1. If HTML is available, convert to Markdown
/// 2. Fall back to plain text
/// 3. Return Empty if nothing available
pub fn get_content_as_markdown() -> ClipboardResult<ClipboardContent> {
    if let Ok(Some(html)) = get_html() {
        if !html.trim().is_empty() {
            match html_to_markdown(&html) {
                Ok(md) => {
                    let trimmed = md.trim().to_string();
                    if !trimmed.is_empty() {
                        return Ok(ClipboardContent::Markdown(trimmed));
                    }
                }
                Err(_) => {
                }
            }
        }
    }

    match get_text() {
        Ok(Some(text)) if !text.is_empty() => Ok(ClipboardContent::PlainText(text)),
        Ok(_) => Ok(ClipboardContent::Empty),
        Err(e) => Err(e),
    }
}

#[allow(dead_code)]
pub fn get_content_plain() -> ClipboardResult<ClipboardContent> {
    match get_text() {
        Ok(Some(text)) if !text.is_empty() => Ok(ClipboardContent::PlainText(text)),
        Ok(_) => Ok(ClipboardContent::Empty),
        Err(e) => Err(e),
    }
}
