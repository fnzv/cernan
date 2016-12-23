#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CString {
    inner: sync::Arc<String>,
}
