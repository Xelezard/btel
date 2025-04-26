pub struct TextBlock {
    pub input: Vec<Vec<char>>,
    pub vert_cursor: usize,
    pub edit_cursor: usize,
    pub saved: bool,
}
impl TextBlock {
    pub fn new() -> TextBlock {
        TextBlock { input: vec![Vec::new()], vert_cursor: 0, edit_cursor: 0, saved: true }
    }
    pub fn write(&mut self, x: char) {
            self.input[self.vert_cursor].insert(self.edit_cursor, x);
            self.edit_cursor += 1;
            self.saved = false
    }
    pub fn backspace(&mut self) {
        if self.edit_cursor != 0 {
            if self.edit_cursor == self.input[self.vert_cursor].len() {
                let _ = self.input[self.vert_cursor].pop();
                self.edit_cursor -= 1
            } else {
                let _ = self.input[self.vert_cursor].remove(self.edit_cursor - 1);
                self.edit_cursor -= 1;
                self.saved = false
            }
        } else if self.vert_cursor != 0 {
            let mut rest = self.input.remove(self.vert_cursor);
            self.input[self.vert_cursor - 1].append(&mut rest);
            self.vert_cursor -= 1;
            self.edit_cursor = self.input[self.vert_cursor].len() - rest.len()
        }
    }
    pub fn delete(&mut self) {
        if self.edit_cursor != self.input[self.vert_cursor].len() {
            if self.edit_cursor + 1 == self.input[self.vert_cursor].len() {
                let _ = self.input[self.vert_cursor].pop();
            } else {
                let _ = self.input[self.vert_cursor].remove(self.edit_cursor + 1);
                /*edit_cursor -= 1;*/
            }
        } else if self.vert_cursor + 1 != self.input.len() {
            let mut rest = self.input.remove(self.vert_cursor + 1);
            self.input[self.vert_cursor].append(&mut rest)
        }
        self.saved = false
    }
    pub fn enter(&mut self) {
        let line = self.input[self.vert_cursor].split_off(self.edit_cursor);
        self.input.insert(self.vert_cursor+1,line);
        self.vert_cursor += 1;
        self.edit_cursor = 0;
    }
    pub fn left(&mut self) {
        if self.edit_cursor != 0 {
            self.edit_cursor -= 1
        } else if self.vert_cursor != 0 {
            self.vert_cursor -= 1;
            self.edit_cursor = self.input[self.vert_cursor].len()
        }
    }
    pub fn right(&mut self) {
        if self.edit_cursor + 1 <= self.input[self.vert_cursor].len() {
            self.edit_cursor += 1
        } else if self.vert_cursor + 1 != self.input.len() {
            self.vert_cursor += 1;
            self.edit_cursor = 0
        }
    }
    pub fn up(&mut self) {
        if self.vert_cursor != 0 {
            self.vert_cursor -= 1;
            if self.input[self.vert_cursor].len() < self.edit_cursor + 2 {
                self.edit_cursor = self.input[self.vert_cursor].len()
            }
        }
    }
    pub fn down(&mut self) {
        if self.vert_cursor + 2 <= self.input.len() {
            self.vert_cursor += 1;
            if self.input[self.vert_cursor].len() < self.edit_cursor + 2 {
                self.edit_cursor = self.input[self.vert_cursor].len()
            }
        }
    }
    pub fn tab(&mut self) {
        {
            for _ in 0..4 {
                self.input[self.vert_cursor].insert(self.edit_cursor, ' ');
            }
            self.edit_cursor += 4;
            self.saved = false
        }
    }
}
