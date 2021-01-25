use egui::{
  paint::{text::cursor::Cursor, Galley},
  *,
};

pub struct ColoredText(pub Vec<Vec<(syntect::highlighting::Style, String)>>);

/// Static text.
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct Code<'a> {
  pub(crate) text: &'a str,
  pub(crate) colored_text: &'a ColoredText,
}

impl<'a> Code<'a> {
  pub fn new(text: &'a str, colored_text: &'a ColoredText) -> Self {
    Self { text, colored_text }
  }

  pub fn layout(&self, ui: &Ui) -> Galley {
    let max_width = ui.available_width();
    self.layout_width(ui, max_width)
  }

  pub fn layout_width(&self, ui: &Ui, max_width: f32) -> Galley {
    let text_style = egui::TextStyle::Monospace;
    let font = &ui.fonts()[text_style];
    font.layout_multiline(self.text.to_owned(), max_width) // TODO: avoid clone
  }

  pub fn paint_galley(&self, ui: &mut Ui, pos: Pos2, galley: Galley) {
    let mut colors: Vec<(usize, egui::Color32)> = Vec::new();
    let mut iter = 0;
    for row in self.colored_text.0.iter() {
      for token in row {
        let token_len = token.1.len();
        if token_len == 0 {
          continue;
        }
        iter += token_len;
        let fg = token.0.foreground;
        let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b);
        colors.push((iter, text_color));
      }
      iter += 1;
    }
    ui.painter()
      .galley_colored(pos, galley, egui::TextStyle::Monospace, colors);
  }
}

impl<'a> Code<'a> {
  pub fn ui(self, ui: &mut Ui, hover_offset: &mut Option<usize>) -> Response {
    let galley = self.layout(ui);
    let (rect, response) = ui.allocate_exact_size(galley.size, Sense::click());

    if let Some(mouse_pos) = ui.input().mouse.pos {
      if response.hovered {
        let font_width = &ui.fonts()[TextStyle::Monospace].glyph_width(' ');
        let cursor =
          galley.cursor_from_pos(mouse_pos - response.rect.min + vec2(-*font_width / 2., 0.));
        paint_cursor(ui, rect.min, &galley, &cursor);

        *hover_offset = Some(cursor.ccursor.index);
      }
    }

    self.paint_galley(ui, rect.min, galley);

    response
  }
}

fn paint_cursor(ui: &mut Ui, pos: Pos2, galley: &Galley, cursor: &Cursor) {
  let rcursor = cursor.rcursor;
  let galley_row = &galley.rows[rcursor.row];
  let left = galley_row.x_offset(rcursor.column);
  let right = galley_row.x_offset(rcursor.column + 1);

  let rect = Rect::from_min_max(
    pos + vec2(left, galley_row.y_min),
    pos + vec2(right, galley_row.y_max),
  );
  ui.painter()
    .rect_filled(rect, 0.0, Color32::from_rgb(0x4f, 0x5b, 0x66));
}
