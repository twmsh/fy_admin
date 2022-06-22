use indexmap::IndexMap;

use axum::extract::Multipart;
use bytes::Bytes;

use log::warn;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::ops::Deref;

#[derive(Debug)]
pub enum MultipartFormItem {
    /// 文件类型
    File(MultipartFormFile),

    // 文本类型
    Text(MultipartFormText),
}

#[derive(Debug)]
pub struct MultipartFormText {
    /// 字段名称
    pub name: String,

    /// 字段值，数组形式
    pub values: Vec<Bytes>,
}

#[derive(Debug)]
pub struct MultipartFormFile {
    /// 字段名称
    pub name: String,

    /// 字段的值，数组形式，支持多个文件
    pub values: Vec<MultipartFormFileValue>,
}

pub struct MultipartFormFileValue {
    /// 文件名称
    pub file_name: String,

    /// 文件内容
    pub data: Bytes,
}

impl Debug for MultipartFormFileValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultipartFormFileValue")
            .field("file_name", &self.file_name)
            .field("data_len", &self.data.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct MultipartFormValues(pub IndexMap<String, MultipartFormItem>);

//---------------------
impl MultipartFormText {
    pub fn new(name: &str, data: bytes::Bytes) -> Self {
        MultipartFormText {
            name: name.to_string(),
            values: vec![data],
        }
    }

    pub fn add(&mut self, file_name: Option<&str>, data: bytes::Bytes) {
        if file_name.is_some() {
            // 文件类型的，忽略
            return;
        }
        self.values.push(data);
    }

    pub fn first_value(&self) -> Option<&Bytes> {
        self.values.get(0)
    }
}

impl MultipartFormFile {
    pub fn new(name: &str, file_name: &str, data: Bytes) -> Self {
        MultipartFormFile {
            name: name.to_string(),
            values: vec![MultipartFormFileValue {
                file_name: file_name.to_string(),
                data,
            }],
        }
    }

    pub fn add(&mut self, file_name: Option<&str>, data: bytes::Bytes) {
        if file_name.is_none() {
            // 不是文件类型的，忽略
            return;
        }

        self.values.push(MultipartFormFileValue {
            file_name: file_name.unwrap_or("").to_string(),
            data,
        });
    }

    pub fn first_value(&self) -> Option<&MultipartFormFileValue> {
        self.values.get(0)
    }
}

impl MultipartFormItem {
    pub fn new_file(field_name: &str, file_name: &str, data: bytes::Bytes) -> Self {
        MultipartFormItem::File(MultipartFormFile::new(field_name, file_name, data))
    }

    pub fn new_text(field_name: &str, data: bytes::Bytes) -> Self {
        MultipartFormItem::Text(MultipartFormText::new(field_name, data))
    }

    pub fn append_file(&mut self, file_name: &str, data: bytes::Bytes) {
        if let MultipartFormItem::File(item) = self {
            item.values.push(MultipartFormFileValue {
                file_name: file_name.to_string(),
                data,
            });
        }
    }

    pub fn append_text(&mut self, data: bytes::Bytes) {
        if let MultipartFormItem::Text(item) = self {
            item.values.push(data);
        }
    }
}

//---------------------
impl Default for MultipartFormValues {
    fn default() -> Self {
        Self::new()
    }
}

impl MultipartFormValues {
    pub fn new() -> Self {
        MultipartFormValues(IndexMap::new())
    }

    pub fn add(&mut self, name: &str, item: MultipartFormItem) {
        self.0.insert(name.to_string(), item);
    }

    /**
     如果text 和 file 的 field_name相同，会如何？
     以最早的出现的field为准
    */
    pub fn add_form_value(
        &mut self,
        field_name: Option<&str>,
        file_name: Option<&str>,
        data: bytes::Bytes,
    ) {
        if field_name.is_none() {
            warn!("field_name is none!");
            return;
        }

        let field_name = field_name.unwrap();
        let value = self.0.get_mut(field_name);

        if let Some(f) = file_name {
            // 文件

            if let Some(v) = value {
                v.append_file(f, data);
            } else {
                self.add(field_name, MultipartFormItem::new_file(field_name, f, data));
            }

            // if value.is_none() {
            //     self.add(field_name, MultipartFormItem::new_file(field_name, f, data));
            // } else {
            //     value.unwrap().append_file(f, data);
            // }
        } else {
            // 文本
            let value = self.0.get_mut(field_name);

            if let Some(v) = value {
                v.append_text(data);
            } else {
                self.add(field_name, MultipartFormItem::new_text(field_name, data));
            }

            // if value.is_none() {
            //     self.add(field_name, MultipartFormItem::new_text(field_name, data));
            // } else {
            //     value.unwrap().append_text(data);
            // }
        }
    }

    pub fn get_string_values(&self, name: &str) -> Option<Vec<String>> {
        if let Some(MultipartFormItem::Text(v)) = self.0.get(name) {
            let values: Vec<String> = v
                .values
                .iter()
                .map(|x| String::from_utf8_lossy(x.deref()).into_owned())
                .collect();
            Some(values)
        } else {
            None
        }
    }

    pub fn get_string_value(&self, name: &str) -> Option<String> {
        let values = self.get_string_values(name);
        let value = match values {
            Some(v) => v,
            None => return None,
        };

        value.get(0).cloned()
    }

    pub fn get_file_values(&self, name: &str) -> Option<Vec<(String, Bytes)>> {
        if let Some(MultipartFormItem::File(v)) = self.0.get(name) {
            let values = v
                .values
                .iter()
                .map(|x| (x.file_name.clone(), x.data.clone()))
                .collect();
            Some(values)
        } else {
            None
        }
    }

    pub fn get_file_value(&self, name: &str) -> Option<(String, Bytes)> {
        let values = self.get_file_values(name);
        let value = match values {
            Some(v) => v,
            None => return None,
        };

        value.get(0).map(|(s, b)| (s.clone(), b.clone()))
    }
}

pub async fn parse_multi_form(
    mut payload: Multipart,
) -> std::result::Result<MultipartFormValues, String> {
    let mut values = MultipartFormValues::new();

    while let Ok(Some(field)) = payload.next_field().await {
        let name = field.name().map(|x| x.to_string());
        let file_name = field.file_name().map(|x| x.to_string());

        let data = match field.bytes().await {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("parse_multi_form, {:?}", e));
            }
        };

        values.add_form_value(name.as_deref(), file_name.as_deref(), data);
    }

    Ok(values)
}
