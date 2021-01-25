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

pub async fn get_quick_info(hover_offset: usize) {
  make_call(JsCallQuery {
    kind: "quickInfo".to_string(),
    value: format!("{}", hover_offset),
  })
  .await;
}

#[wasm_bindgen]
extern "C" {
  async fn make_call(value: JsCallQuery) -> JsValue;
}
