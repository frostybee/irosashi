//! Direct Oniguruma calls. A safe wrapper's search entry points allocate a match
//! parameter per call and validate the encoding per call; the tokenizer issues tens of
//! searches per scan step, so those costs dominate.

use std::ffi::CStr;
use std::sync::Mutex;

use onig_sys::{OnigErrorInfo, OnigRegex, OnigRegion};

static COMPILE: Mutex<()> = Mutex::new(());

pub(crate) enum Search {
    Found,
    NotFound,
    Failed,
}

/// A compiled pattern. Searching is thread safe; the results live in the caller's
/// `Region`.
pub(crate) struct Regex {
    raw: OnigRegex,
}

unsafe impl Send for Regex {}
unsafe impl Sync for Regex {}

impl Regex {
    /// Compiles with capture groups on, UTF-8, and Oniguruma's default syntax.
    pub fn new(source: &str) -> Result<Self, String> {
        let bytes = source.as_bytes();
        let mut raw: OnigRegex = std::ptr::null_mut();
        let mut info = OnigErrorInfo {
            enc: std::ptr::null_mut(),
            par: std::ptr::null_mut(),
            par_end: std::ptr::null_mut(),
        };
        let _guard = COMPILE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let code = unsafe {
            onig_sys::onig_new(
                &mut raw,
                bytes.as_ptr(),
                bytes.as_ptr().add(bytes.len()),
                onig_sys::ONIG_OPTION_CAPTURE_GROUP,
                &raw mut onig_sys::OnigEncodingUTF8,
                onig_sys::OnigDefaultSyntax,
                &mut info,
            )
        };
        if code == onig_sys::ONIG_NORMAL as i32 {
            Ok(Self { raw })
        } else {
            Err(error_message(code, &info))
        }
    }

    /// Searches `text` for a match starting in `start..=text.len()`, with the whole
    /// text visible to lookbehind and anchors.
    pub fn search(&self, text: &str, start: usize, options: u32, region: &mut Region) -> Search {
        debug_assert!(start <= text.len());
        let bytes = text.as_bytes();
        let code = unsafe {
            onig_sys::onig_search(
                self.raw,
                bytes.as_ptr(),
                bytes.as_ptr().add(bytes.len()),
                bytes.as_ptr().add(start),
                bytes.as_ptr().add(bytes.len()),
                region.raw,
                options,
            )
        };
        if code >= 0 {
            Search::Found
        } else if code == onig_sys::ONIG_MISMATCH {
            Search::NotFound
        } else {
            Search::Failed
        }
    }
}

impl Regex {
    /// Whether the pattern matches anywhere in `text`.
    pub fn is_match_anywhere(&self, text: &str) -> bool {
        matches!(
            self.search(text, 0, onig_sys::ONIG_OPTION_NONE, &mut Region::new()),
            Search::Found
        )
    }
}

impl Drop for Regex {
    fn drop(&mut self) {
        unsafe { onig_sys::onig_free(self.raw) };
    }
}

/// Capture positions of the last successful search.
pub(crate) struct Region {
    raw: *mut OnigRegion,
}

unsafe impl Send for Region {}

impl Region {
    pub fn new() -> Self {
        let raw = unsafe { onig_sys::onig_region_new() };
        assert!(!raw.is_null(), "onig_region_new returned null");
        Self { raw }
    }

    pub fn len(&self) -> usize {
        unsafe { (*self.raw).num_regs.max(0) as usize }
    }

    /// Byte range of group `index`, `None` when it did not participate.
    pub fn pos(&self, index: usize) -> Option<(usize, usize)> {
        if index >= self.len() {
            return None;
        }
        let (beg, end) = unsafe { (*(*self.raw).beg.add(index), *(*self.raw).end.add(index)) };
        if beg == onig_sys::ONIG_REGION_NOTPOS || beg < 0 || end < beg {
            None
        } else {
            Some((beg as usize, end as usize))
        }
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        unsafe { onig_sys::onig_region_free(self.raw, 1) };
    }
}

fn error_message(code: i32, info: &OnigErrorInfo) -> String {
    let mut buf = [0u8; onig_sys::ONIG_MAX_ERROR_MESSAGE_LEN as usize + 1];
    let len = unsafe {
        onig_sys::onig_error_code_to_str(buf.as_mut_ptr(), code, info as *const OnigErrorInfo)
    };
    if len <= 0 {
        return format!("oniguruma error {code}");
    }
    CStr::from_bytes_until_nul(&buf)
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|_| format!("oniguruma error {code}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_searches_and_reports_errors() {
        let re = Regex::new(r"(\w+)(?:\s+(\d+))?").unwrap();
        let mut region = Region::new();
        assert!(matches!(
            re.search("hi 42", 0, 0, &mut region),
            Search::Found
        ));
        assert_eq!(region.len(), 3);
        assert_eq!(region.pos(0), Some((0, 5)));
        assert_eq!(region.pos(2), Some((3, 5)));
        assert!(matches!(re.search("hi", 1, 0, &mut region), Search::Found));
        assert_eq!(region.pos(2), None);
        assert!(matches!(
            re.search("  ", 0, 0, &mut region),
            Search::NotFound
        ));
        let err = match Regex::new("(") {
            Ok(_) => panic!("unbalanced paren compiled"),
            Err(err) => err,
        };
        assert!(err.contains("end pattern"), "{err}");
    }

    #[test]
    fn not_begin_options_are_honoured() {
        let re = Regex::new(r"\G\w").unwrap();
        let mut region = Region::new();
        assert!(matches!(re.search("abc", 1, 0, &mut region), Search::Found));
        assert!(matches!(
            re.search(
                "abc",
                1,
                onig_sys::ONIG_OPTION_NOT_BEGIN_POSITION,
                &mut region
            ),
            Search::NotFound
        ));
    }
}
