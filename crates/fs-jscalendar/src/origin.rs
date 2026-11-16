//! Where an imported object came from (fs-objects `object_origin`), shared by calendar and tasks.
//!
//! Pattern: Value object — plain data, read only. "Unchanged since import" is computed
//! (`revision == rev_at_import`), the file name never counts (ADR 0085 section 1). The shape is
//! field for field the one both topic traits use, so their stand-ins convert without loss.

use fs_types::{FileRef, LocalDateTime, MediaType, Reference, Revision, Utc};

/// The channel an object came in through.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OriginChannel {
    /// A file.
    File,
    /// A `CalDAV` account.
    CalDav,
    /// A Google account (closed format: import only).
    Google,
    /// A Microsoft account (closed format: import only).
    Outlook,
    /// An iTIP message.
    Itip,
    /// The hub.
    Hub,
    /// A feed by URL (`webcal:`/`https:`): `Origin::source` is the feed IRI, not a file hash, so
    /// repeated fetches share one source. JMAP is deliberately not in v1.
    Subscription,
}

/// The origin of an imported object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Origin {
    /// How it came in.
    pub channel: OriginChannel,
    /// Accounts: IRI of the collection; files: `urn:sha256:<hex>` of the file; feeds: the feed IRI.
    pub source: Reference,
    /// The media type of what was read.
    pub media_type: MediaType,
    /// The foreign `UID`.
    pub foreign_uid: String,
    /// The foreign `RECURRENCE-ID`, when it was one occurrence.
    pub foreign_recurrence_id: Option<LocalDateTime>,
    /// The foreign `ETag`.
    pub foreign_etag: Option<String>,
    /// The foreign `PRODID`.
    pub prod_id: Option<String>,
    /// When it was imported.
    pub imported: Utc,
    /// Name and version of the importer.
    pub importer: String,
    /// The revision of the row at import.
    pub rev_at_import: Revision,
    /// The original (place and checksum) when kept: the basis of "byte for byte" round trips
    /// (ADR 0085 section 1, ADR 0086 point 4).
    pub original: Option<FileRef>,
}
