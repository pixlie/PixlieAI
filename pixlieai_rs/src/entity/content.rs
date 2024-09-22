pub struct Title {
    pub value: String,
}

/// Represents any content from any source that is long form text.
///
/// This can be email body, message body, file content, etc.
// pub struct Content {
//     pub chunks: Vec<ContentChunk>,
// }

/// Represents chunks of content.
///
/// We split any content into chunks if the text is too large to fit
/// into the context window of the LLM. We use the position to identify
/// the chunk in the document.
///
/// The position is the index of the chunk in the source content,
/// starting from 0. Since a document is a String (vector of u8),
/// the position is the index in this vector.
pub struct ContentChunk {
    pub id: u32,
    pub node_id: u32,

    pub position: u16,
    pub text_chunk: String,
}
