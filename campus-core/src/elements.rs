use std::{path::Path, fs::read_to_string};
use serde::{Deserialize, Serialize};
use crate::errors::CampusError;

/// 网络服务类型
#[derive(Deserialize, Serialize)]
pub struct ServiceMapping {
    /// id
    pub id: String,
    /// 名称
    pub name: String,
}


// 元素也都设置为pub
#[derive(Deserialize, Serialize)]
pub struct PageElements {
    pub login_button: String,
    pub username_tip: String,
    pub username_input: String,
    pub password_tip: String,
    pub password_input: String,
    pub service_tip: String,
}


#[derive(Deserialize, Serialize)]
pub struct Elements {
    pub page: PageElements,
    pub service: Vec<ServiceMapping>,
}


impl Elements {
    pub fn load_file(from_path: &Path) -> Result<Self, CampusError> {
        let text = match read_to_string(from_path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(CampusError::ElementsNotFound(from_path.display().to_string()));
            }
            Err(e) => return Err(e.into()),
        };
        let elements: Elements = toml::from_str(&text)
            .map_err(CampusError::ElementsParse)?;
        Ok(elements)
    }
}