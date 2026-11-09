#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JavaString(pub Vec<u16>);
impl From<&str> for JavaString {
    fn from(s: &str) -> Self { JavaString(s.encode_utf16().collect()) }
}
impl JavaString {
    pub fn to_string_lossy(&self) -> String { String::from_utf16_lossy(&self.0) }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Tag { Byte(i8), Int(i32), Long(i64), String(JavaString), Compound(Compound) }
pub type Compound = std::collections::BTreeMap<JavaString, Tag>;
