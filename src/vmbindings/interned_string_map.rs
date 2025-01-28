//! Provides interned string map
use std::rc::Rc;
const MAX_LENGTH: usize = u16::MAX as usize;

#[derive(Debug, Clone, Default)]
pub struct InternedStringMap {
    data: Vec<Rc<String>>,
}

impl InternedStringMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_insert<T: AsRef<str>>(&mut self, s: &T) -> Option<u16> {
        let s: &str = s.as_ref();
        // only intern string in this length range to save up memory
        if !(2..20).contains(&s.len()) {
            return None;
        }
        let it = self
            .data
            .iter()
            .enumerate()
            .find(|(_, key)| key.as_str() == s);

        if let Some((idx, _)) = it {
            Some(idx as u16)
        } else if self.data.len() > MAX_LENGTH {
            None
        } else {
            self.data.push(Rc::new(String::from(s)));
            Some((self.data.len() - 1) as u16)
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, idx: u16) -> Option<&Rc<String>> {
        self.data.get(idx as usize)
    }

    pub unsafe fn get_unchecked(&self, idx: u16) -> &Rc<String> {
        self.data.get_unchecked(idx as usize)
    }
}
