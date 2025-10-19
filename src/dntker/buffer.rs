#[derive(Debug, Clone, Default)]
pub(crate) struct InputBuffer {
    data: Vec<u8>,
    cursor: usize,
}

impl InputBuffer {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_inject(inject: &str) -> Self {
        let mut buffer = Self {
            data: inject.as_bytes().to_vec(),
            cursor: 0,
        };
        buffer.cursor = buffer.data.len();
        buffer
    }

    pub(crate) fn insert(&mut self, byte: u8) {
        self.data.insert(self.cursor, byte);
        self.cursor += 1;
    }

    pub(crate) fn push(&mut self, byte: u8) {
        self.data.push(byte);
        self.cursor = self.data.len();
    }

    pub(crate) fn pop_last(&mut self) -> Option<u8> {
        let popped = self.data.pop()?;
        if self.cursor > self.data.len() {
            self.cursor = self.data.len();
        }
        Some(popped)
    }

    pub(crate) fn delete(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;
        self.data.remove(self.cursor);
        true
    }

    pub(crate) fn move_left(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;
        true
    }

    pub(crate) fn move_right(&mut self) -> bool {
        if self.cursor >= self.data.len() {
            return false;
        }
        self.cursor += 1;
        true
    }

    pub(crate) fn replace(&mut self, statement: &str) {
        self.data.clear();
        self.data.extend_from_slice(statement.as_bytes());
        self.cursor = self.data.len();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub(crate) fn last(&self) -> Option<u8> {
        self.data.last().copied()
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub(crate) fn statement(&self) -> String {
        std::str::from_utf8(&self.data).unwrap().to_string()
    }

    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }
}

#[cfg(test)]
mod tests {
    use super::InputBuffer;

    #[test]
    fn insert_and_delete() {
        let mut buffer = InputBuffer::new();
        assert!(!buffer.delete());

        buffer.insert(b'1');
        buffer.insert(b'+');
        buffer.insert(b'2');
        assert_eq!(buffer.statement(), "1+2");
        assert_eq!(buffer.cursor(), 3);

        buffer.move_left();
        assert_eq!(buffer.cursor(), 2);

        assert!(buffer.delete());
        assert_eq!(buffer.statement(), "12");
        assert_eq!(buffer.cursor(), 1);
    }
}
