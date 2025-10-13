#![expect(clippy::redundant_pub_crate)]

use std::{borrow::Cow, cell::Cell, iter::FusedIterator};

use rustc_hash::FxHashSet;

use oxc_ast::{Comment, CommentKind, ast::Program};
use oxc_syntax::identifier::is_line_terminator;

use crate::{
    Codegen, LegalComment,
    options::CommentOptions,
    str::{LS_LAST_2_BYTES, LS_OR_PS_FIRST_BYTE, PS_LAST_2_BYTES},
};

pub struct CommentsMap {
    buckets: Vec<CommentBucket>,
    storage: Vec<Comment>,
    active: usize,
    cursor: Cell<usize>,
}

impl Default for CommentsMap {
    fn default() -> Self {
        Self { buckets: Vec::new(), storage: Vec::new(), active: 0, cursor: Cell::new(0) }
    }
}

struct CommentBucket {
    key: u32,
    start: usize,
    len: usize,
    consumed: bool,
}

impl CommentBucket {
    #[inline]
    fn new(key: u32, start: usize) -> Self {
        Self { key, start, len: 1, consumed: false }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CommentBlock {
    ptr: *const Comment,
    len: usize,
}

impl CommentBlock {
    #[inline]
    fn new(ptr: *const Comment, len: usize) -> Self {
        Self { ptr, len }
    }

    #[inline]
    pub(crate) fn len(self) -> usize {
        self.len
    }

    #[inline]
    pub(crate) fn is_empty(self) -> bool {
        self.len == 0
    }

    #[inline]
    pub(crate) fn iter(self) -> CommentBlockIter {
        CommentBlockIter { ptr: self.ptr, end: unsafe { self.ptr.add(self.len) } }
    }
}

pub(crate) struct CommentBlockIter {
    ptr: *const Comment,
    end: *const Comment,
}

impl Iterator for CommentBlockIter {
    type Item = Comment;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        } else {
            // SAFETY: `ptr` always advances towards `end`, and both originate from the same slice.
            let comment = unsafe { *self.ptr };
            // SAFETY: `ptr < end`, so incrementing by 1 stays within the original slice bounds.
            self.ptr = unsafe { self.ptr.add(1) };
            Some(comment)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = unsafe { self.end.offset_from(self.ptr) };
        debug_assert!(remaining >= 0);
        let remaining = usize::try_from(remaining)
            .expect("comment iterator pointer order must be non-negative");
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for CommentBlockIter {}

impl FusedIterator for CommentBlockIter {}

impl CommentsMap {
    #[inline]
    pub(crate) fn clear(&mut self) {
        self.buckets.clear();
        self.storage.clear();
        self.active = 0;
        self.cursor.set(0);
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.active == 0
    }

    #[inline]
    pub(crate) fn contains_key(&self, key: u32) -> bool {
        self.find_index(key)
            .map(|idx| {
                let bucket = &self.buckets[idx];
                !bucket.consumed && bucket.len > 0
            })
            .unwrap_or(false)
    }

    #[inline]
    pub(crate) fn reserve(&mut self, additional: usize) {
        self.storage.reserve(additional);
        self.buckets.reserve(additional);
    }

    #[inline]
    pub(crate) fn insert_comment(&mut self, key: u32, comment: Comment) {
        if let Some(bucket) = self.buckets.last_mut() {
            if bucket.key == key {
                debug_assert_eq!(bucket.start + bucket.len, self.storage.len());
                self.storage.push(comment);
                bucket.len += 1;
                return;
            }
            if key > bucket.key {
                let start = self.storage.len();
                self.storage.push(comment);
                self.buckets.push(CommentBucket::new(key, start));
                self.active += 1;
                return;
            }
        } else {
            let start = self.storage.len();
            self.storage.push(comment);
            self.buckets.push(CommentBucket::new(key, start));
            self.active = 1;
            return;
        }

        self.insert_comment_slow_path(key, comment);
    }

    #[inline]
    pub(crate) fn remove(&mut self, key: u32) -> Option<CommentBlock> {
        let idx = self.locate_bucket(key)?;
        let (start, len) = {
            let bucket = &mut self.buckets[idx];
            if bucket.consumed || bucket.len == 0 {
                return None;
            }
            bucket.consumed = true;
            self.active = self.active.saturating_sub(1);
            (bucket.start, bucket.len)
        };
        self.cursor.set(idx.saturating_add(1));
        Some(self.block_from_parts(start, len))
    }

    #[inline]
    pub(crate) fn peek(&self, key: u32) -> Option<CommentBlock> {
        let idx = self.locate_bucket(key)?;
        Some(self.block_from_parts(self.buckets[idx].start, self.buckets[idx].len))
    }

    #[inline]
    fn find_index(&self, key: u32) -> Result<usize, usize> {
        self.buckets.binary_search_by_key(&key, |bucket| bucket.key)
    }

    #[inline(always)]
    fn locate_bucket(&self, key: u32) -> Option<usize> {
        if self.buckets.is_empty() {
            return None;
        }
        let len = self.buckets.len();
        let mut cursor = self.cursor.get().min(len.saturating_sub(1));
        let mut bucket = &self.buckets[cursor];
        if bucket.key == key && !bucket.consumed && bucket.len > 0 {
            return Some(cursor);
        }

        if bucket.key < key {
            while cursor + 1 < len {
                cursor += 1;
                bucket = &self.buckets[cursor];
                if bucket.key > key {
                    break;
                }
                if bucket.key == key && !bucket.consumed && bucket.len > 0 {
                    self.cursor.set(cursor);
                    return Some(cursor);
                }
            }
        } else {
            while cursor > 0 {
                cursor -= 1;
                bucket = &self.buckets[cursor];
                if bucket.key < key {
                    break;
                }
                if bucket.key == key && !bucket.consumed && bucket.len > 0 {
                    self.cursor.set(cursor);
                    return Some(cursor);
                }
            }
        }

        let idx = self.find_index(key).ok()?;
        let bucket = &self.buckets[idx];
        if bucket.consumed || bucket.len == 0 {
            None
        } else {
            self.cursor.set(idx);
            Some(idx)
        }
    }

    #[cold]
    fn insert_comment_slow_path(&mut self, key: u32, comment: Comment) {
        match self.find_index(key) {
            Ok(idx) => {
                debug_assert!(!self.buckets[idx].consumed);
                let bucket = &mut self.buckets[idx];
                let insert_at = bucket.start + bucket.len;
                self.storage.insert(insert_at, comment);
                bucket.len += 1;
                for later in &mut self.buckets[idx + 1..] {
                    later.start += 1;
                }
            }
            Err(idx) => {
                let insert_at = if idx == self.buckets.len() {
                    self.storage.len()
                } else {
                    self.buckets[idx].start
                };
                self.storage.insert(insert_at, comment);
                if idx < self.buckets.len() {
                    for later in &mut self.buckets[idx..] {
                        later.start += 1;
                    }
                }
                self.buckets.insert(idx, CommentBucket::new(key, insert_at));
                self.active += 1;
                self.cursor.set(idx);
            }
        }
    }

    #[inline]
    fn block_from_parts(&self, start: usize, len: usize) -> CommentBlock {
        let ptr = unsafe { self.storage.as_ptr().add(start) };
        CommentBlock::new(ptr, len)
    }
}

/// Custom iterator that splits text on line terminators while handling CRLF as a single unit.
/// This avoids creating empty strings between CR and LF characters.
///
/// Also splits on irregular line breaks (LS and PS).
///
/// # Example
/// Standard split would turn `"line1\r\nline2"` into `["line1", "", "line2"]` because
/// it treats `\r` and `\n` as separate terminators. This iterator correctly produces
/// `["line1", "line2"]` by treating `\r\n` as a single terminator.
struct LineTerminatorSplitter<'a> {
    text: &'a str,
}

impl<'a> LineTerminatorSplitter<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> Iterator for LineTerminatorSplitter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.text.is_empty() {
            return None;
        }

        for (index, &byte) in self.text.as_bytes().iter().enumerate() {
            match byte {
                b'\n' => {
                    // SAFETY: Byte at `index` is `\n`, so `index` and `index + 1` are both UTF-8 char boundaries.
                    // Therefore, slices up to `index` and from `index + 1` are both valid `&str`s.
                    unsafe {
                        let line = self.text.get_unchecked(..index);
                        self.text = self.text.get_unchecked(index + 1..);
                        return Some(line);
                    }
                }
                b'\r' => {
                    // SAFETY: Byte at `index` is `\r`, so `index` is on a UTF-8 char boundary
                    let line = unsafe { self.text.get_unchecked(..index) };
                    // If the next byte is `\n`, consume it as well
                    let skip_bytes =
                        if self.text.as_bytes().get(index + 1) == Some(&b'\n') { 2 } else { 1 };
                    // SAFETY: `index + skip_bytes` is after `\r` or `\n`, so on a UTF-8 char boundary.
                    // Therefore slice from `index + skip_bytes` is a valid `&str`.
                    self.text = unsafe { self.text.get_unchecked(index + skip_bytes..) };
                    return Some(line);
                }
                LS_OR_PS_FIRST_BYTE => {
                    let next2: [u8; 2] = {
                        // SAFETY: 0xE2 is always the start of a 3-byte Unicode character,
                        // so there must be 2 more bytes available to consume
                        let next2 =
                            unsafe { self.text.as_bytes().get_unchecked(index + 1..index + 3) };
                        next2.try_into().unwrap()
                    };
                    // If this is LS or PS, treat it as a line terminator
                    if matches!(next2, LS_LAST_2_BYTES | PS_LAST_2_BYTES) {
                        // SAFETY: `index` is the start of a 3-byte Unicode character,
                        // so `index` and `index + 3` are both UTF-8 char boundaries.
                        // Therefore, slices up to `index` and from `index + 3` are both valid `&str`s.
                        unsafe {
                            let line = self.text.get_unchecked(..index);
                            self.text = self.text.get_unchecked(index + 3..);
                            return Some(line);
                        }
                    }
                }
                _ => {}
            }
        }

        // No line break found - return the remaining text. Next call will return `None`.
        let line = self.text;
        self.text = "";
        Some(line)
    }
}

impl FusedIterator for LineTerminatorSplitter<'_> {}

impl Codegen<'_> {
    pub(crate) fn build_comments(&mut self, comments: &[Comment]) {
        if self.options.comments == CommentOptions::disabled() {
            return;
        }
        self.comments.reserve(comments.len());
        for comment in comments {
            // Omit pure comments because they are handled separately.
            if comment.is_pure() || comment.is_no_side_effects() {
                continue;
            }
            let mut add = false;
            if comment.is_leading() {
                if comment.is_legal() && self.options.print_legal_comment() {
                    add = true;
                }
                if comment.is_jsdoc() && self.options.print_jsdoc_comment() {
                    add = true;
                }
                if comment.is_annotation() && self.options.print_annotation_comment() {
                    add = true;
                }
                if comment.is_normal() && self.options.print_normal_comment() {
                    add = true;
                }
            }
            if add {
                self.comments.insert_comment(comment.attached_to, *comment);
            }
        }
    }

    pub(crate) fn has_comment(&self, start: u32) -> bool {
        self.comments.contains_key(start)
    }

    pub(crate) fn print_leading_comments(&mut self, start: u32) {
        if let Some(comments) = self.comments.remove(start) {
            self.print_comments(comments);
        }
    }

    pub(crate) fn take_comments(&mut self, start: u32) -> Option<CommentBlock> {
        if self.comments.is_empty() {
            return None;
        }
        self.comments.remove(start)
    }

    #[inline]
    pub(crate) fn peek_comments(&self, start: u32) -> Option<CommentBlock> {
        if self.comments.is_empty() {
            return None;
        }
        self.comments.peek(start)
    }

    #[inline]
    pub(crate) fn print_comments_at(&mut self, start: u32) {
        if let Some(comments) = self.take_comments(start) {
            self.print_comments(comments);
        }
    }

    pub(crate) fn print_expr_comments(&mut self, start: u32) -> bool {
        if self.comments.is_empty() {
            return false;
        }
        let Some(comments) = self.comments.remove(start) else { return false };
        if comments.is_empty() {
            return false;
        }

        for comment in comments.iter() {
            self.print_hard_newline();
            self.print_indent();
            self.print_comment(&comment);
        }

        self.print_hard_newline();
        true
    }

    pub(crate) fn print_comments(&mut self, comments: CommentBlock) {
        let total = comments.len();
        if total == 0 {
            return;
        }
        for (i, comment) in comments.iter().enumerate() {
            if i == 0 {
                if comment.preceded_by_newline() {
                    // Skip printing newline if this comment is already on a newline.
                    if let Some(b) = self.last_byte() {
                        match b {
                            b'\n' => self.print_indent(),
                            b'\t' => { /* noop */ }
                            _ => {
                                self.print_hard_newline();
                                self.print_indent();
                            }
                        }
                    }
                } else {
                    self.print_indent();
                }
            }
            if i >= 1 {
                if comment.preceded_by_newline() {
                    self.print_hard_newline();
                    self.print_indent();
                } else if comment.is_legal() {
                    self.print_hard_newline();
                }
            }
            self.print_comment(&comment);
            if i == total - 1 {
                if comment.is_line() || comment.followed_by_newline() {
                    self.print_hard_newline();
                } else {
                    self.print_next_indent_as_space = true;
                }
            }
        }
    }

    fn print_comment(&mut self, comment: &Comment) {
        let Some(source_text) = self.source_text else {
            return;
        };
        let comment_source = comment.span.source_text(source_text);
        match comment.kind {
            CommentKind::Line => {
                self.print_str_escaping_script_close_tag(comment_source);
            }
            CommentKind::Block => {
                for line in LineTerminatorSplitter::new(comment_source) {
                    if !line.starts_with("/*") {
                        self.print_indent();
                    }
                    self.print_str_escaping_script_close_tag(line.trim_start());
                    if !line.ends_with("*/") {
                        self.print_hard_newline();
                    }
                }
            }
        }
    }

    /// Handle Eof / Linked / External Comments.
    /// Return a list of comments of linked or external.
    pub(crate) fn handle_eof_linked_or_external_comments(
        &mut self,
        program: &Program<'_>,
    ) -> Vec<Comment> {
        let legal_comments = &self.options.comments.legal;
        if matches!(legal_comments, LegalComment::None | LegalComment::Inline) {
            return vec![];
        }

        // Dedupe legal comments for smaller output size.
        let mut set = FxHashSet::default();
        let mut comments = vec![];

        let source_text = program.source_text;
        for comment in program.comments.iter().filter(|c| c.is_legal()) {
            let mut text = Cow::Borrowed(comment.span.source_text(source_text));
            if comment.is_block() && text.contains(is_line_terminator) {
                let mut buffer = String::with_capacity(text.len());
                // Print block comments with our own indentation.
                for line in LineTerminatorSplitter::new(&text) {
                    if !line.starts_with("/*") {
                        buffer.push('\t');
                    }
                    buffer.push_str(line.trim_start());
                    if !line.ends_with("*/") {
                        buffer.push('\n');
                    }
                }
                text = Cow::Owned(buffer);
            }
            if set.insert(text) {
                comments.push(*comment);
            }
        }

        if comments.is_empty() {
            return vec![];
        }

        match legal_comments {
            LegalComment::Eof => {
                self.print_hard_newline();
                for c in comments {
                    self.print_comment(&c);
                    self.print_hard_newline();
                }
                vec![]
            }
            LegalComment::Linked(path) => {
                let path = path.clone();
                self.print_hard_newline();
                self.print_str("/*! For license information please see ");
                self.print_str(&path);
                self.print_str(" */");
                comments
            }
            LegalComment::External => comments,
            LegalComment::None | LegalComment::Inline => unreachable!(),
        }
    }
}
