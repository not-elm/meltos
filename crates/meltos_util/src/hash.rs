#[inline]
pub fn random() -> String {
    hash(&uuid::Uuid::new_v4().into_bytes())
}


#[inline]
pub fn hash(data: &[u8]) -> String {
    let mut hasher = sha1_smol::Sha1::new();
    hasher.update(data);
    hasher.digest().to_string()
}
