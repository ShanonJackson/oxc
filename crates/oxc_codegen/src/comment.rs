use std::{borrow::Cow, cell::Cell, iter::FusedIterator};

use rustc_hash::FxHashSet;

use oxc_ast::{Comment, CommentKind, ast::Program};
use oxc_syntax::identifier::is_line_terminator;

use crate::{
    Codegen, LegalComment,
    options::CommentOptions,
    str::{LS_LAST_2_BYTES, LS_OR_PS_FIRST_BYTE, PS_LAST_2_BYTES},
};

#[derive(Default)]
pub struct CommentsMap {
    keys: Vec<u32>,
    values: Vec<Option<Vec<Comment>>>,
    remaining: usize,
    cached_lookup: Cell<Option<(u32, usize)>>,
}

impl CommentsMap {
    fn clear(&mut self) {
        self.keys.clear();
        self.values.clear();
        self.remaining = 0;
        self.cached_lookup.set(None);
    }

    fn rebuild(&mut self, mut items: Vec<(u32, Comment)>) {
        if items.is_empty() {
            self.clear();
            return;
        }

        items.sort_unstable_by_key(|(start, _)| *start);

        self.keys.clear();
        self.values.clear();
        self.keys.reserve(items.len());
        self.values.reserve(items.len());

        let mut index = 0;
        while index < items.len() {
            let start = items[index].0;
            self.keys.push(start);

            let mut group = Vec::new();
            while index < items.len() && items[index].0 == start {
                group.push(items[index].1);
                index += 1;
            }
            self.values.push(Some(group));
        }

        self.remaining = self.values.len();
        self.cached_lookup.set(None);
    }

    #[inline]
    fn find_index(&self, start: u32) -> Option<usize> {
        match self.keys.binary_search(&start) {
            Ok(index) => Some(index),
            Err(_) => None,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.remaining == 0
    }

    #[inline]
    pub fn has(&self, start: u32) -> bool {
        if self.remaining == 0 {
            self.cached_lookup.set(None);
            return false;
        }

        if let Some((cached_start, index)) = self.cached_lookup.get() {
            if cached_start == start {
                return self.values.get(index).is_some_and(|value| value.is_some());
            }
        }

        let Some(index) = self.find_index(start) else {
            self.cached_lookup.set(None);
            return false;
        };

        self.cached_lookup.set(Some((start, index)));
        self.values.get(index).is_some_and(|value| value.is_some())
    }

    #[inline]
    pub fn take(&mut self, start: u32) -> Option<Vec<Comment>> {
        if self.remaining == 0 {
            return None;
        }

        let index = if let Some((cached_start, index)) = self.cached_lookup.get() {
            if cached_start == start {
                index
            } else {
                let index = self.find_index(start)?;
                self.cached_lookup.set(Some((start, index)));
                index
            }
        } else {
            let index = self.find_index(start)?;
            self.cached_lookup.set(Some((start, index)));
            index
        };

        let Some(value) = self.values.get_mut(index) else {
            return None;
        };
        let comments = value.take()?;
        self.remaining -= 1;

        if self.remaining == 0
            || self.cached_lookup.get().is_some_and(|(_, cached_index)| cached_index == index)
        {
            self.cached_lookup.set(None);
        }

        Some(comments)
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

        let bytes = self.text.as_bytes();
        let ascii_index = find_ascii_break(bytes);
        let ls_index = find_unicode_break(bytes);

        match (ascii_index, ls_index) {
            (Some(i), Some(j)) if j < i => split_on_ls(self, j),
            (Some(i), Some(_)) | (Some(i), None) => split_on_ascii(self, i),
            (None, Some(j)) => split_on_ls(self, j),
            (None, None) => {
                let line = self.text;
                self.text = "";
                Some(line)
            }
        }
    }
}

impl FusedIterator for LineTerminatorSplitter<'_> {}

#[inline]
fn split_on_ascii<'a>(splitter: &mut LineTerminatorSplitter<'a>, index: usize) -> Option<&'a str> {
    let bytes = splitter.text.as_bytes();
    match bytes[index] {
        b'\n' => unsafe {
            let line = splitter.text.get_unchecked(..index);
            splitter.text = splitter.text.get_unchecked(index + 1..);
            Some(line)
        },
        b'\r' => {
            let line = unsafe { splitter.text.get_unchecked(..index) };
            let skip = if bytes.get(index + 1) == Some(&b'\n') { 2 } else { 1 };
            splitter.text = unsafe { splitter.text.get_unchecked(index + skip..) };
            Some(line)
        }
        _ => unreachable!(),
    }
}

#[inline]
fn split_on_ls<'a>(splitter: &mut LineTerminatorSplitter<'a>, index: usize) -> Option<&'a str> {
    unsafe {
        let line = splitter.text.get_unchecked(..index);
        splitter.text = splitter.text.get_unchecked(index + 3..);
        Some(line)
    }
}

fn find_unicode_break(bytes: &[u8]) -> Option<usize> {
    let mut search = bytes;
    let mut offset = 0usize;

    while let Some(index) = find_byte(search, LS_OR_PS_FIRST_BYTE) {
        let candidate = offset + index;
        if candidate + 2 >= bytes.len() {
            break;
        }
        let next2 = [bytes[candidate + 1], bytes[candidate + 2]];
        if matches!(next2, LS_LAST_2_BYTES | PS_LAST_2_BYTES) {
            return Some(candidate);
        }
        let next_start = index + 1;
        search = &search[next_start..];
        offset += next_start;
    }

    None
}

fn find_ascii_break(bytes: &[u8]) -> Option<usize> {
    const BYTE_REPEAT: u64 = 0x0101_0101_0101_0101;
    const HIGH_BITS: u64 = 0x8080_8080_8080_8080;
    const NL_REPEATED: u64 = u64::from_ne_bytes([b'\n'; 8]);
    const CR_REPEATED: u64 = u64::from_ne_bytes([b'\r'; 8]);

    let ptr = bytes.as_ptr();
    let len = bytes.len();
    let mut index = 0usize;

    while index + 8 <= len {
        let chunk = unsafe { ptr.add(index).cast::<u64>().read_unaligned() };
        let nl_diff = chunk ^ NL_REPEATED;
        let nl_matches = nl_diff.wrapping_sub(BYTE_REPEAT) & !nl_diff & HIGH_BITS;
        let cr_diff = chunk ^ CR_REPEATED;
        let cr_matches = cr_diff.wrapping_sub(BYTE_REPEAT) & !cr_diff & HIGH_BITS;
        let combined = nl_matches | cr_matches;
        if combined != 0 {
            let offset = (combined.trailing_zeros() as usize) >> 3;
            return Some(index + offset);
        }
        index += 8;
    }

    while index < len {
        let byte = unsafe { *ptr.add(index) };
        if byte == b'\n' || byte == b'\r' {
            return Some(index);
        }
        index += 1;
    }

    None
}

fn find_byte(bytes: &[u8], target: u8) -> Option<usize> {
    const BYTE_REPEAT: u64 = 0x0101_0101_0101_0101;
    const HIGH_BITS: u64 = 0x8080_8080_8080_8080;
    let repeated = u64::from_ne_bytes([target; 8]);

    let ptr = bytes.as_ptr();
    let len = bytes.len();
    let mut index = 0usize;

    while index + 8 <= len {
        let chunk = unsafe { ptr.add(index).cast::<u64>().read_unaligned() };
        let diff = chunk ^ repeated;
        let matches = diff.wrapping_sub(BYTE_REPEAT) & !diff & HIGH_BITS;
        if matches != 0 {
            let offset = (matches.trailing_zeros() as usize) >> 3;
            return Some(index + offset);
        }
        index += 8;
    }

    while index < len {
        if unsafe { *ptr.add(index) } == target {
            return Some(index);
        }
        index += 1;
    }

    None
}

impl Codegen<'_> {
    pub(crate) fn build_comments(&mut self, comments: &[Comment]) {
        if self.options.comments == CommentOptions::disabled() {
            self.comments.clear();
            return;
        }

        let mut filtered = Vec::new();
        filtered.reserve(comments.len());

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
                filtered.push((comment.attached_to, *comment));
            }
        }

        self.comments.rebuild(filtered);
    }

    #[inline]
    pub(crate) fn has_comment(&self, start: u32) -> bool {
        self.comments.has(start)
    }

    pub(crate) fn print_leading_comments(&mut self, start: u32) {
        if let Some(comments) = self.comments.take(start) {
            self.print_comments(&comments);
        }
    }

    pub(crate) fn get_comments(&mut self, start: u32) -> Option<Vec<Comment>> {
        if self.comments.is_empty() {
            return None;
        }
        self.comments.take(start)
    }

    #[inline]
    pub(crate) fn print_comments_at(&mut self, start: u32) {
        if let Some(comments) = self.get_comments(start) {
            self.print_comments(&comments);
        }
    }

    pub(crate) fn print_expr_comments(&mut self, start: u32) -> bool {
        if self.comments.is_empty() {
            return false;
        }
        let Some(comments) = self.comments.take(start) else { return false };

        for comment in &comments {
            self.print_hard_newline();
            self.print_indent();
            self.print_comment(comment);
        }

        if comments.is_empty() {
            false
        } else {
            self.print_hard_newline();
            true
        }
    }

    pub(crate) fn print_comments(&mut self, comments: &[Comment]) {
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
            self.print_comment(comment);
            if i == comments.len() - 1 {
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
