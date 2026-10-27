#[allow(unused_imports)]
use super::*;

impl Vocab {
    /// Returns the id for `word`, assigning one if it is new.
    pub fn intern(&mut self, word: &str) -> WordId {
        if let Some(id) = self.ids.get(word) {
            return *id;
        }
        let id = self.words.len() as WordId;
        self.words.push(word.to_string());
        self.ids.insert(word.to_string(), id);
        id
    }

    /// Returns the id for `word` without assigning one.
    #[inline]
    pub fn id(&self, word: &str) -> Option<WordId> {
        self.ids.get(word).copied()
    }

    /// Returns the word for `id`.
    #[inline]
    pub fn word(&self, id: WordId) -> Option<&str> {
        self.words.get(id as usize).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// The full id→word table, for serialization.
    pub fn words(&self) -> &[String] {
        &self.words
    }

    /// Rebuilds an interner from a stored id→word table.
    pub fn from_words(words: Vec<String>) -> Self {
        let ids = words
            .iter()
            .enumerate()
            .map(|(i, w)| (w.clone(), i as WordId))
            .collect();
        Self { ids, words }
    }
}

#[inline]
pub(super) fn bump(list: &mut Followers, id: WordId) {
    match list.binary_search_by_key(&id, |(k, _)| *k) {
        Ok(i) => list[i].1 = list[i].1.saturating_add(1),
        Err(i) => list.insert(i, (id, 1)),
    }
}

#[inline]
pub(super) fn lookup(list: &[(WordId, u32)], id: WordId) -> u32 {
    match list.binary_search_by_key(&id, |(k, _)| *k) {
        Ok(i) => list[i].1,
        Err(_) => 0,
    }
}

#[inline]
pub(super) fn total_of(list: &[(WordId, u32)]) -> u32 {
    list.iter().map(|(_, c)| *c).sum()
}
