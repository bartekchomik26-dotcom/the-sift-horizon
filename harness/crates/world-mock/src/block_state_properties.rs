/// As Horizon's: properties sorted by name.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BlockProperties(Vec<(String, String)>);
impl BlockProperties {
    pub fn new() -> Self { Self::default() }
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.binary_search_by(|(k, _)| k.as_str().cmp(key)).ok().map(|i| &self.0[i].1)
    }
    pub fn contains_key(&self, key: &str) -> bool { self.get(key).is_some() }
    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        match self.0.binary_search_by(|(k, _)| k.as_str().cmp(key.as_str())) {
            Ok(i) => Some(std::mem::replace(&mut self.0[i].1, value)),
            Err(i) => { self.0.insert(i, (key, value)); None }
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> { self.0.iter().map(|(k, v)| (k, v)) }
}
impl<'a> IntoIterator for &'a BlockProperties {
    type Item = (&'a String, &'a String);
    type IntoIter = Box<dyn Iterator<Item = (&'a String, &'a String)> + 'a>;
    fn into_iter(self) -> Self::IntoIter { Box::new(self.0.iter().map(|(k, v)| (k, v))) }
}
