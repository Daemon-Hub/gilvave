use serde::Serialize;

use crate::error::ErrorInfo;

pub trait RequestBody {
    fn to_json(&self) -> Result<Option<String>, ErrorInfo>;
}

impl RequestBody for () {
    fn to_json(&self) -> Result<Option<String>, ErrorInfo> {
        Ok(None)
    }
}

impl<T: Serialize + ?Sized> RequestBody for &T {
    fn to_json(&self) -> Result<Option<String>, ErrorInfo> {
        serde_json::to_string(self)
            .map(Some)
            .map_err(|e| ErrorInfo(1, e.to_string()))
    }
}
