use std::str::CharIndices;

#[derive(Debug, Clone)]
pub struct LineChars<'a> {
    iter: CharIndices<'a>,

    pub index: usize,
    pub line: usize,
    pub line_index: usize,
}

impl<'a> LineChars<'a> {
    pub fn new(char_indices: CharIndices<'a>) -> Self {
        Self {
            iter: char_indices,
            index: 0,
            line: 1,
            line_index: 1,
        }
    }

    pub fn as_str(&self) -> &'a str {
        self.iter.as_str()
    }
}

impl<'a> Iterator for LineChars<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let result = self.iter.next();
        if let Some((index, ch)) = result {
            self.index = index;
            if ch == '\n' {
                self.line += 1;
                self.line_index = 1;
            }
        }
        result
    }
}
