use crate::error::{BotlError, Result};
use crate::parser::objects::{ObjRef, ObjectParser, PdfDict, PdfObject, PdfStream};
use crate::parser::xref::{parse_xref_from_data, XrefEntry, XrefTable};
use hashbrown::HashMap;
use std::path::Path;
use std::sync::Arc;

/// A parsed PDF document.
pub struct Document {
    /// The raw file data (mmap'd or heap-allocated).
    data: Vec<u8>,
    /// The cross-reference table.
    xref: XrefTable,
    /// Cache of parsed indirect objects, wrapped in Arc to avoid cloning large trees.
    object_cache: HashMap<u32, Arc<PdfObject>>,
    /// Flattened page references (lazily populated).
    page_cache: Option<Vec<ObjRef>>,
    /// Cache of decoded object stream data, keyed by object stream number.
    /// Avoids re-decompressing the same stream when resolving sibling objects.
    obj_stream_cache: HashMap<u32, Arc<Vec<u8>>>,
}

impl Document {
    /// Open a PDF document from a file path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path)?;
        Self::from_bytes(data)
    }

    /// Parse a PDF document from bytes.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        if data.len() < 5 {
            return Err(BotlError::ParseError("File too small to be a PDF".into()));
        }
        if !data.starts_with(b"%PDF-") {
            return Err(BotlError::ParseError(
                "File does not start with %PDF- header".into(),
            ));
        }

        let xref = parse_xref_from_data(&data)?;
        let doc = Self {
            data,
            xref,
            object_cache: HashMap::new(),
            page_cache: None,
            obj_stream_cache: HashMap::new(),
        };

        // Pre-load the root catalog
        if doc.xref.root().is_none() {
            return Err(BotlError::ParseError("No Root in trailer".into()));
        }

        Ok(doc)
    }

    /// Get the PDF version string (e.g., "1.7").
    pub fn version(&self) -> Option<&str> {
        // Parse %PDF-X.Y header
        if self.data.len() >= 8 {
            let header = &self.data[..8];
            if header.starts_with(b"%PDF-") {
                return std::str::from_utf8(&header[5..8]).ok();
            }
        }
        None
    }

    /// Resolve an indirect object reference to its PdfObject.
    /// Returns an Arc<PdfObject> to avoid cloning large object trees on cache hits.
    pub fn resolve(&mut self, reference: ObjRef) -> Result<Arc<PdfObject>> {
        // Check cache first
        if let Some(obj) = self.object_cache.get(&reference.obj_num) {
            return Ok(Arc::clone(obj));
        }

        let entry = *self
            .xref
            .get(reference.obj_num)
            .ok_or(BotlError::InvalidReference(
                reference.obj_num,
                reference.gen_num,
            ))?;

        let object = match entry {
            XrefEntry::InUse { offset, .. } => {
                let offset = offset as usize;
                if offset >= self.data.len() {
                    return Err(BotlError::ParseError(format!(
                        "Object {} offset {} past end of file",
                        reference.obj_num, offset
                    )));
                }
                let mut parser = ObjectParser::new(&self.data[offset..]);
                let indirect = parser.parse_indirect_object()?;
                indirect.object
            }
            XrefEntry::Compressed {
                obj_stream_num,
                index,
            } => {
                // Check object stream cache first
                let decoded = if let Some(cached) = self.obj_stream_cache.get(&obj_stream_num) {
                    Arc::clone(cached)
                } else {
                    // Load and decompress the object stream
                    let stream_obj = self.resolve(ObjRef::new(obj_stream_num, 0))?;
                    let stream = stream_obj.as_stream().ok_or_else(|| {
                        BotlError::ParseError("Expected stream for object stream".into())
                    })?;
                    let decoded = Arc::new(crate::codecs::decode_stream_data(stream)?);
                    self.obj_stream_cache.insert(obj_stream_num, Arc::clone(&decoded));
                    decoded
                };
                // We need the stream dict for N/First values; resolve the stream again
                // (it's cached in object_cache, so this is cheap)
                let stream_obj = self.resolve(ObjRef::new(obj_stream_num, 0))?;
                let stream_dict = stream_obj.as_stream()
                    .map(|s| &s.dict)
                    .ok_or_else(|| BotlError::ParseError("Expected stream for object stream".into()))?;
                self.parse_object_from_decoded_data(&decoded, stream_dict, index)?
            }
            XrefEntry::Free { .. } => {
                return Err(BotlError::ParseError(format!(
                    "Object {} is free (not in use)",
                    reference.obj_num
                )))
            }
        };

        let arc = Arc::new(object);
        self.object_cache.insert(reference.obj_num, Arc::clone(&arc));
        Ok(arc)
    }

    /// Parse objects from an object stream (ObjStm).
    fn parse_object_from_stream(
        &mut self,
        stream: &PdfStream,
        target_index: u32,
    ) -> Result<PdfObject> {
        let decoded = crate::codecs::decode_stream_data(stream)?;
        self.parse_object_from_decoded_data(&decoded, &stream.dict, target_index)
    }

    /// Parse a target object from already-decoded object stream data.
    fn parse_object_from_decoded_data(
        &self,
        decoded: &[u8],
        stream_dict: &PdfDict,
        target_index: u32,
    ) -> Result<PdfObject> {
        let n = stream_dict
            .get_integer("N")
            .ok_or_else(|| BotlError::ParseError("Object stream missing N".into()))?
            as u32;
        let first = stream_dict
            .get_integer("First")
            .ok_or_else(|| BotlError::ParseError("Object stream missing First".into()))?
            as usize;

        let header_data = &decoded[..first];
        let mut parser = ObjectParser::new(header_data);
        let mut pairs = Vec::new();
        for _ in 0..n {
            let obj_num = parser.parse_object()?.as_integer().unwrap_or(0) as u32;
            let offset = parser.parse_object()?.as_integer().unwrap_or(0) as usize;
            pairs.push((obj_num, offset));
        }

        if (target_index as usize) >= pairs.len() {
            return Err(BotlError::ParseError(
                "Object stream index out of range".into(),
            ));
        }

        let (_obj_num, offset) = pairs[target_index as usize];
        let abs_offset = first + offset;
        let mut obj_parser = ObjectParser::new(&decoded[abs_offset..]);
        obj_parser.parse_object()
    }    /// Parse a target object from already-decoded object stream data.
    fn parse_object_from_decoded(
        &self,
        decoded: &[u8],
        stream_dict: &PdfDict,
        target_index: u32,
    ) -> Result<PdfObject> {
        let n = stream_dict
            .get_integer("N")
            .ok_or_else(|| BotlError::ParseError("Object stream missing N".into()))?
            as u32;
        let first = stream_dict
            .get_integer("First")
            .ok_or_else(|| BotlError::ParseError("Object stream missing First".into()))?
            as usize;

        let header_data = &decoded[..first];
        let mut parser = ObjectParser::new(header_data);
        let mut pairs = Vec::new();
        for _ in 0..n {
            let obj_num = parser.parse_object()?.as_integer().unwrap_or(0) as u32;
            let offset = parser.parse_object()?.as_integer().unwrap_or(0) as usize;
            pairs.push((obj_num, offset));
        }

        if (target_index as usize) >= pairs.len() {
            return Err(BotlError::ParseError(
                "Object stream index out of range".into(),
            ));
        }

        let (_obj_num, offset) = pairs[target_index as usize];
        let abs_offset = first + offset;
        let mut obj_parser = ObjectParser::new(&decoded[abs_offset..]);
        obj_parser.parse_object()
    }

    /// Get the root catalog dictionary.
    pub fn catalog(&mut self) -> Result<PdfDict> {
        let root_ref = self
            .xref
            .root()
            .ok_or_else(|| BotlError::ParseError("No Root".into()))?;
        let obj = self.resolve(root_ref)?;
        obj.as_dict()
            .cloned()
            .ok_or_else(|| BotlError::ParseError("Root is not a dictionary".into()))
    }

    /// Get the number of pages.
    pub fn num_pages(&mut self) -> Result<usize> {
        let catalog = self.catalog()?;
        let pages_ref = catalog
            .get_reference("Pages")
            .ok_or_else(|| BotlError::ParseError("Catalog missing Pages reference".into()))?;
        let pages_dict = self.resolve(pages_ref)?;
        let pages_dict = pages_dict
            .as_dict()
            .ok_or_else(|| BotlError::ParseError("Pages is not a dictionary".into()))?;
        Ok(pages_dict.get_integer("Count").unwrap_or(0) as usize)
    }

    /// Get the page tree node.
    #[allow(dead_code)]
    fn get_pages_tree(&mut self) -> Result<PdfDict> {
        let catalog = self.catalog()?;
        let pages_ref = catalog
            .get_reference("Pages")
            .ok_or_else(|| BotlError::ParseError("Catalog missing Pages reference".into()))?;
        let pages_obj = self.resolve(pages_ref)?;
        pages_obj
            .as_dict()
            .cloned()
            .ok_or_else(|| BotlError::ParseError("Pages is not a dictionary".into()))
    }

    /// Get a specific page dictionary (0-indexed).
    /// Uses a flat page cache for O(1) access after the first call.
    pub fn get_page(&mut self, page_index: usize) -> Result<PdfDict> {
        if self.page_cache.is_none() {
            self.flatten_pages()?;
        }
        let cache = self.page_cache.as_ref().unwrap();
        if page_index >= cache.len() {
            return Err(BotlError::PageOutOfRange {
                page: page_index,
                total: cache.len(),
            });
        }
        let page_ref = cache[page_index];
        let obj = self.resolve(page_ref)?;
        obj.as_dict()
            .cloned()
            .ok_or_else(|| BotlError::ParseError("Page is not a dictionary".into()))
    }

    /// Get the flattened list of page object references.
    /// Lazily populated on first access.
    pub fn page_refs(&mut self) -> Result<&[ObjRef]> {
        if self.page_cache.is_none() {
            self.flatten_pages()?;
        }
        Ok(self.page_cache.as_ref().unwrap())
    }

    /// Resolve a destination to a 0-based page index.
    ///
    /// A destination can be:
    /// - An array: `[page_ref /XYZ left top zoom]` or `[page_ref /Fit ...]`
    /// - A name (named destination) -- currently not supported, returns None
    ///
    /// The page reference in the destination is matched against the document's
    /// page tree to find the 0-based index.
    pub fn resolve_destination_page(&mut self, dest: &PdfObject) -> Option<usize> {
        match dest {
            PdfObject::Array(arr) => {
                let page_ref = arr.first()?.as_reference()?;
                self.find_page_index(page_ref)
            }
            PdfObject::Name(_name) => {
                // Named destinations require a lookup in the Destinations name tree
                // or the Dests dictionary in the catalog. Not yet implemented.
                None
            }
            PdfObject::String(_s) => {
                // Named destination as a string -- same as above.
                None
            }
            PdfObject::Reference(r) => {
                // Indirect reference to a destination array or name.
                let resolved = self.resolve(*r).ok()?;
                self.resolve_destination_page(&resolved)
            }
            _ => None,
        }
    }

    /// Given an `ObjRef` that refers to a page dictionary, find its 0-based
    /// index in the page tree. Returns `None` if the reference doesn't match
    /// any page.
    fn find_page_index(&mut self, target: ObjRef) -> Option<usize> {
        let refs = self.page_refs().ok()?;
        refs.iter().position(|&r| r == target)
    }

    /// Flatten the entire page tree into a flat list of page references.
    fn flatten_pages(&mut self) -> Result<()> {
        let catalog = self.catalog()?;
        let pages_ref = catalog
            .get_reference("Pages")
            .ok_or_else(|| BotlError::ParseError("Catalog missing Pages reference".into()))?;
        let mut pages = Vec::new();
        self.collect_pages(pages_ref, &mut pages)?;
        self.page_cache = Some(pages);
        Ok(())
    }

    /// Recursively collect all leaf page references from the page tree.
    fn collect_pages(&mut self, node_ref: ObjRef, pages: &mut Vec<ObjRef>) -> Result<()> {
        let node_obj = self.resolve(node_ref)?;
        let node = node_obj
            .as_dict()
            .ok_or_else(|| BotlError::ParseError("Page tree node is not a dictionary".into()))?;

        let node_type = node.get_name("Type").unwrap_or("");
        if node_type == "Page" {
            pages.push(node_ref);
            return Ok(());
        }

        // It's a Pages node; resolve Kids
        let kids = match node.get(b"Kids") {
            Some(PdfObject::Array(arr)) => arr.clone(),
            Some(PdfObject::Reference(r)) => {
                let resolved = self.resolve(*r)?;
                match resolved.as_ref() {
                    PdfObject::Array(arr) => arr.clone(),
                    other => vec![other.clone()],
                }
            }
            _ => return Err(BotlError::ParseError("Pages node missing Kids".into())),
        };

        for kid in &kids {
            let kid_ref = kid
                .as_reference()
                .ok_or_else(|| BotlError::ParseError("Kid is not a reference".into()))?;
            self.collect_pages(kid_ref, pages)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn find_page(
        &mut self,
        node_ref: ObjRef,
        target: usize,
        counter: &mut usize,
    ) -> Result<PdfDict> {
        let node_obj = self.resolve(node_ref)?;
        let node = node_obj
            .as_dict()
            .ok_or_else(|| BotlError::ParseError("Page tree node is not a dictionary".into()))?;

        let node_type = node.get_name("Type").unwrap_or("");
        if node_type == "Page" {
            if *counter == target {
                return Ok(node.clone());
            }
            *counter += 1;
            return Err(BotlError::PageOutOfRange {
                page: target,
                total: *counter,
            });
        }

        // It's a Pages node; iterate kids
        let kids = match node.get(b"Kids") {
            Some(PdfObject::Array(arr)) => arr.clone(),
            Some(PdfObject::Reference(r)) => {
                let resolved = self.resolve(*r)?;
                match resolved.as_ref() {
                    PdfObject::Array(arr) => arr.clone(),
                    other => vec![other.clone()],
                }
            }
            _ => return Err(BotlError::ParseError("Pages node missing Kids".into())),
        };

        for kid in &kids {
            let kid_ref = kid
                .as_reference()
                .ok_or_else(|| BotlError::ParseError("Kid is not a reference".into()))?;
            let kid_count = {
                let kid_obj = self.resolve(kid_ref)?;
                let kid_dict = kid_obj
                    .as_dict()
                    .ok_or_else(|| BotlError::ParseError("Kid is not a dictionary".into()))?;
                if kid_dict.get_name("Type") == Some("Page") {
                    1
                } else {
                    kid_dict.get_integer("Count").unwrap_or(0) as usize
                }
            };

            if target < *counter + kid_count {
                return self.find_page(kid_ref, target, counter);
            }
            *counter += kid_count;
        }

        let total = *counter;
        Err(BotlError::PageOutOfRange {
            page: target,
            total,
        })
    }

    /// Extract metadata from the document.
    pub fn metadata(&mut self) -> Result<DocumentMetadata> {
        let catalog = self.catalog()?;
        let mut meta = DocumentMetadata::default();

        // Try Info dictionary from trailer
        if let Some(info_ref) = self.xref.info() {
            if let Ok(info_obj) = self.resolve(info_ref) {
                if let Some(info_dict) = info_obj.as_dict() {
                    meta.title = info_dict.get_string("Title").map(String::from);
                    meta.author = info_dict.get_string("Author").map(String::from);
                    meta.subject = info_dict.get_string("Subject").map(String::from);
                    meta.keywords = info_dict.get_string("Keywords").map(String::from);
                    meta.creator = info_dict.get_string("Creator").map(String::from);
                    meta.producer = info_dict.get_string("Producer").map(String::from);
                    meta.creation_date = info_dict.get_string("CreationDate").map(String::from);
                    meta.mod_date = info_dict.get_string("ModDate").map(String::from);
                }
            }
        }

        // Try metadata stream (PDF 2.0+)
        if meta.title.is_none() {
            if let Some(meta_ref) = catalog.get_reference("Metadata") {
                if let Ok(meta_obj) = self.resolve(meta_ref) {
                    if let Some(stream) = meta_obj.as_stream() {
                        // TODO: Parse XMP metadata
                        let _ = stream;
                    }
                }
            }
        }

        meta.page_count = self.num_pages()?;
        meta.version = self.version().map(String::from);

        Ok(meta)
    }

    /// Check if the document is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.xref.encrypt().is_some()
    }

    /// Get the raw cross-reference table (for debugging).
    pub fn xref(&self) -> &XrefTable {
        &self.xref
    }

    /// Access the raw file data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Document metadata.
#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub mod_date: Option<String>,
    pub page_count: usize,
    pub version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_non_pdf() {
        let result = Document::from_bytes(b"Hello, world!".to_vec());
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_too_small() {
        let result = Document::from_bytes(b"PDF".to_vec());
        assert!(result.is_err());
    }
}
