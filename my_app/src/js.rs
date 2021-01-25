use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct JsCallQuery {
  kind: String,
  value: String,
}

#[wasm_bindgen]
impl JsCallQuery {
  #[wasm_bindgen(getter)]
  pub fn kind(&self) -> String {
    self.kind.clone()
  }

  #[wasm_bindgen(getter)]
  pub fn value(&self) -> String {
    self.value.clone()
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsCallResponse {
  pub kind: String,
  pub value: String,
}

pub mod api {
  use super::*;

  pub async fn get_quick_info(filename: String, offset: usize) {
    make_call(JsCallQuery {
      kind: "quickInfo".to_string(),
      value: format!("{}${}", filename, offset),
    })
    .await;
  }

  pub async fn go_to_definition(filename: String, offset: usize) {
    make_call(JsCallQuery {
      kind: "goToDefinition".to_string(),
      value: format!("{}${}", filename, offset),
    })
    .await;
  }
}

#[wasm_bindgen]
extern "C" {
  async fn make_call(value: JsCallQuery) -> JsValue;
}

#[derive(Deserialize, Debug)]
pub struct TextSpan {
  pub start: usize,
  pub length: usize,
}

impl TextSpan {
  pub fn includes_offset(&self, offset: usize) -> bool {
    offset >= self.start && offset < self.start + self.length
  }
}

#[derive(Deserialize, Debug)]
pub struct DisplayPart {
  pub text: String,
  pub kind: String,
}

#[derive(Deserialize, Debug)]
pub struct QuickInfoResponse {
  pub kind: String,
  #[serde(rename = "textSpan")]
  pub text_span: TextSpan,
  #[serde(rename = "displayParts")]
  pub display_parts: Vec<DisplayPart>,
}

#[derive(Deserialize, Debug)]
pub struct FileContentsResponse {
  pub filename: String,
  pub contents: String,
}
