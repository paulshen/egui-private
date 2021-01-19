use egui::{pos2, CursorIcon, Response, Sense, TextStyle, Ui, Widget};

pub struct Hyperlink {
  // TODO: wrap Label
  url: String,
  text: String,
  pub(crate) text_style: Option<TextStyle>,
}

impl Hyperlink {
  pub fn new(url: impl Into<String>) -> Self {
    let url = url.into();
    Self {
      text: url.clone(),
      url,
      text_style: None,
    }
  }

  /// Show some other text than the url
  pub fn text(mut self, text: impl Into<String>) -> Self {
    self.text = text.into();
    self
  }
}

impl Widget for Hyperlink {
  fn ui(self, ui: &mut Ui) -> Response {
    let Hyperlink {
      url,
      text,
      text_style,
    } = self;
    let text_style = text_style.unwrap_or_else(|| ui.style().body_text_style);
    let font = &ui.fonts()[text_style];
    let galley = font.layout_multiline(text, ui.available_width());
    let (rect, response) = ui.allocate_exact_size(galley.size, Sense::click());

    if response.hovered {
      ui.ctx().output().cursor_icon = CursorIcon::PointingHand;
    }
    if response.clicked {
      ui.ctx().output().open_url = Some(url.clone());
    }

    let color = ui.style().visuals.hyperlink_color;
    let visuals = ui.style().interact(&response);

    if response.hovered {
      // Underline:
      for line in &galley.rows {
        let pos = rect.min;
        let y = pos.y + line.y_max;
        let y = ui.painter().round_to_pixel(y);
        let min_x = pos.x + line.min_x();
        let max_x = pos.x + line.max_x();
        ui.painter().line_segment(
          [pos2(min_x, y), pos2(max_x, y)],
          (visuals.fg_stroke.width, color),
        );
      }
    }

    ui.painter().galley(rect.min, galley, text_style, color);
    response
  }
}
