#[derive(Debug, Default)]
pub struct SseBlockDecoder { buffer: String }

impl SseBlockDecoder {
    pub fn push(&mut self, chunk: &str) -> Vec<String> {
        self.buffer.push_str(chunk);
        let mut blocks = Vec::new();
        while let Some(index) = self.buffer.find("\n\n") {
            let block = self.buffer[..index].trim().to_string();
            self.buffer.drain(..index + 2);
            if !block.is_empty() { blocks.push(block); }
        }
        blocks
    }
    pub fn pending(&self) -> &str { &self.buffer }
}
