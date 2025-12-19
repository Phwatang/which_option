use std::ops::RangeInclusive;
use iced::Color;
use iced::Element;
use iced::widget::text_input::Catalog;
use iced::widget::{TextInput, text_input};
use iced::Theme;

#[derive(Debug, Clone)]
pub enum NumberInputMessage {
    Edit(String),
}

/// A TextInput widget than ensures only valid numbers/floats can be entered into it
#[derive(Debug, Clone)]
pub struct NumberInput {
    /// The number/float in the textinput
    value_str: String,
    /// Determines the amount of numbers after the decimal point that can be entered
    dp_precision: Option<usize>,
    allowed_range: RangeInclusive<f64>
}
impl Default for NumberInput {
    fn default() -> Self {
        Self {
            value_str: Default::default(),
            dp_precision: None,
            allowed_range: f64::MIN..=f64::MAX
        }
    }
}
impl NumberInput {    
    pub fn update(&mut self, message: NumberInputMessage) {
        match message {
            NumberInputMessage::Edit(new_val) => {
                // Make sure the textbox can be empty or have a lone negative sign
                if new_val.is_empty() || new_val == "-" || new_val.parse::<f64>().is_ok() {
                    self.value_str = new_val;
                    self.apply_precision();
                }
            }
        }
    }

    /// Sets the max dp precision of this NumberInput
    pub fn set_precision(mut self, precision: usize) -> Self {
        self.dp_precision = Some(precision);
        return self;
    }

    pub fn set_range(&mut self, range: RangeInclusive<f64>) -> &mut Self {
        self.allowed_range = range;
        self
    }

    fn apply_precision(&mut self) {
        if let Some(precision) = self.dp_precision {
            let over = self.get_precision().saturating_sub(precision);
            self.value_str.truncate(self.value_str.len() - over);
        }
    }

    /// Checks if user inputted value is outside the allowed range. An incomplete user input is considered inside.
    pub fn value_outside_range(&self) -> bool {
        !self.allowed_range.contains(&self.value_str.parse::<f64>().unwrap_or(*self.allowed_range.start()))
    }

    /// Retrieves the value entered into the TextInput. Returns NAN if user input isn't a complete number.
    /// 
    /// Any non NAN values are clamped according to allowed range specified. (By default its f64::MIN..=f64::MAX)
    pub fn get_value(&self) -> f64 {
        self.value_str.parse::<f64>().unwrap_or(f64::NAN)
            .clamp(*self.allowed_range.start(), *self.allowed_range.end())
    }

    pub fn set_value(&mut self, value: f64) {
        self.value_str = value.to_string();
        self.apply_precision();
    }

    /// Calculates the "implied" precision of the number.
    /// 
    /// E.g "1." is 0 precision, "1.0" is 1 precision, "1.50" is 2 precision, etc
    pub fn get_precision(&self) -> usize {
        return self.value_str.split_once(".")
            .unwrap_or(("", ""))
            .1.len();
    }

    pub fn style_strategy(use_red: bool) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
        move |t: &Theme, status| {
            let mut style = t.style(&Theme::default(), status);
            if use_red {
                style.border = style.border.color(Color::from_rgb8(255, 0, 0));
            }
            return style;
        }
    }

    pub fn view(&self) -> Element<'_, NumberInputMessage, Theme> {
        text_input("", &self.value_str)
            .on_input(NumberInputMessage::Edit)
            .style(Self::style_strategy(self.value_outside_range()))
            .into()
    }

    pub fn adjust_then_view<'a>(&self, adjust: impl Fn(TextInput<'a, NumberInputMessage>) -> TextInput<'a, NumberInputMessage>) -> Element<'a, NumberInputMessage> {
        adjust(text_input("", &self.value_str)
            .on_input(NumberInputMessage::Edit)
        ).into()
    }
}