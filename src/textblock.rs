use crate::view::ViewRule;
pub struct TextBlock {
    pub input: Vec<Vec<char>>,
    pub vert_cursor: usize,
    pub edit_cursor: usize,
    pub saved: bool,
    pub view: Vec<Vec<char>>,
    pub view_info: Vec<(usize,ViewRule)>
}
pub enum Target {
    Input,
    View
}
impl TextBlock {
    pub fn new() -> TextBlock {
        TextBlock { input: vec![Vec::new()], vert_cursor: 0, edit_cursor: 0, saved: true,view: vec![Vec::new()],view_info: Vec::new()}
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
                self.edit_cursor -= 1;
                self.saved = false
            } else {
                let _ = self.input[self.vert_cursor].remove(self.edit_cursor - 1);
                self.edit_cursor -= 1;
                self.saved = false
            }
        } else if self.vert_cursor != 0 {
            let mut rest = self.input.remove(self.vert_cursor);
            let len = rest.len();
            self.input[self.vert_cursor - 1].append(&mut rest);
            self.vert_cursor -= 1;
            self.edit_cursor = self.input[self.vert_cursor].len() - len;
            self.saved = false
        }
    }
    pub fn delete(&mut self) {
        if self.edit_cursor != self.input[self.vert_cursor].len() {
            if self.edit_cursor + 1 == self.input[self.vert_cursor].len() {
                let _ = self.input[self.vert_cursor].pop();
            } else {
                let _ = self.input[self.vert_cursor].remove(self.edit_cursor);
            }
        } else if self.vert_cursor + 1 != self.input.len() {
            let mut rest = self.input.remove(self.vert_cursor+ 1);
            self.input[self.vert_cursor].append(&mut rest)
        }
        self.saved = false
    }
    pub fn enter(&mut self) {
        let line = self.input[self.vert_cursor].split_off(self.edit_cursor);
        self.input.insert(self.vert_cursor+1,line);
        self.vert_cursor += 1;
        self.edit_cursor = 0;
        self.saved = false
    }
    pub fn left(&mut self,target: Target) {
        let target = match target {
            Target::View => &self.view,
            Target::Input => &self.input
        };
        if self.edit_cursor != 0 {
            self.edit_cursor -= 1
        } else if self.vert_cursor != 0 {
            self.vert_cursor -= 1;
            self.edit_cursor = target[self.vert_cursor].len()
        }
    }
    pub fn right(&mut self,target: Target) {
        let target = match target {
            Target::View => &self.view,
            Target::Input => &self.input
        };
        if self.edit_cursor + 1 <= target[self.vert_cursor].len() {
            self.edit_cursor += 1
        } else if self.vert_cursor + 1 != target.len() {
            self.vert_cursor += 1;
            self.edit_cursor = 0
        }
    }
    pub fn up(&mut self,target: Target) {
        let target = match target {
            Target::View => &self.view,
            Target::Input => &self.input
        };
        if self.vert_cursor != 0 {
            self.vert_cursor -= 1;
            if target[self.vert_cursor].len() < self.edit_cursor + 2 {
                self.edit_cursor = target[self.vert_cursor].len()
            }
        }
    }
    pub fn down(&mut self,target: Target) {
        let target = match target {
            Target::View => &self.view,
            Target::Input => &self.input
        };
        if self.vert_cursor + 2 <= target.len() {
            self.vert_cursor += 1;
            if target[self.vert_cursor].len() < self.edit_cursor + 2 {
                self.edit_cursor = target[self.vert_cursor].len()
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
