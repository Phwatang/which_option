use std::ops::RangeInclusive;
use iced::alignment::Vertical;
use iced::Element;
use iced::Length;
use iced::widget::{TextInput, column, container, row, slider, text_input, text};

use super::{NumberInput, NumberInputMessage};

#[derive(Debug, Clone)]
pub enum CustomSliderMessage {
    Slide(f64),
    NumberInputMessage(usize, NumberInputMessage),
}

pub struct CustomSlider {
    title: String,
    value: f64,
    /// Purpose of each NumberInput widget is as follows:
    /// 
    /// 0: Set/display lower bound \
    /// 1: Set/display upper bound \
    /// 2: Set/display number read out
    number_inputs: [NumberInput; 3],
    allowed_range: RangeInclusive<f64>,
}
impl Default for CustomSlider {
    fn default() -> Self {
        Self {
            title: Default::default(),
            value: Default::default(),
            number_inputs: Default::default(),
            allowed_range: f64::MIN..=f64::MAX,
        }
    }
}
impl CustomSlider {
    pub fn update(&mut self, message: CustomSliderMessage) {
        match message {
            CustomSliderMessage::Slide(new_val) => {
                self.value = new_val;
                let value_str = self.value.to_string();
                self.number_inputs[2].update(NumberInputMessage::Edit(value_str));
            }
            CustomSliderMessage::NumberInputMessage(i, msg) => {
                self.number_inputs[i].update(msg);
                self.update_input_ranges();
                self.value = self.number_inputs[i].get_value();
                if self.value.is_nan() {
                    self.value = 0.0;
                }
            }
        }
    }

    fn update_input_ranges(&mut self) {
        self.number_inputs.iter_mut().for_each(|i| {i.set_range(self.allowed_range.clone());});
        let mut lower = self.number_inputs[0].get_value();
        if lower.is_nan() {
            lower = *self.allowed_range.start();
        }
        self.number_inputs[1].set_range(lower..=*self.allowed_range.end());
    }

    pub fn get_value(&self) -> f64 {
        return self.value;
    }

    pub fn set_title(&mut self, title: String) -> &mut Self {
        self.title = title;
        self
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value;
        self.number_inputs[2].set_value(value);
    }

    pub fn set_precision(mut self, precision: usize) -> Self {
        self.number_inputs = self.number_inputs.map(|x| x.set_precision(precision));
        return self;
    }

    pub fn set_allowed_range(&mut self, range: RangeInclusive<f64>) -> &mut Self {
        self.allowed_range = range;
        self
    }

    /// Retrieves the range of values the slider is configured to occupy from the NumberInput widgets.
    /// 
    /// Range given is guaranteed to be within self.allowed_range. An empty input for the NumberInputs will default
    /// to the starting value of self.allowed_range.
    pub fn get_slider_range(&self) -> RangeInclusive<f64> {
        let mut bounds = [self.number_inputs[0].get_value(), self.number_inputs[1].get_value()];
        for bound in bounds.iter_mut() {
            if bound.is_nan() {
                *bound = *self.allowed_range.start()
            }
        }
        let start = bounds[0].clamp(*self.allowed_range.start(), *self.allowed_range.end());
        let end = bounds[1].clamp(*self.allowed_range.start(), *self.allowed_range.end());
        return start..=end;
    }

    pub fn set_slider_range(&mut self, range: RangeInclusive<f64>) {
        self.number_inputs[0].set_value(*range.start());
        self.number_inputs[1].set_value(*range.end());
    }

    pub fn view(&self) -> Element<'_, CustomSliderMessage> {

        use iced::Theme;
        // Formatting for the textinputs of the range boundaries for the sliders
        fn style_strategy(show_red: bool) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
            move |theme, status| {
                let mut s = NumberInput::style_strategy(true)(theme, status);
                if !show_red {
                    s.border = s.border.width(0.0);
                }
                return s;
            }
        }

        // Ensure a proper range is defined (i.e lower number then higher number)
        let lower = self.number_inputs[0].get_value();
        let upper = self.number_inputs[1].get_value().max(lower);

        column![
            text!("{}", self.title),
            row![
                container(
                    self.number_inputs[0]
                        .adjust_then_view(|o: TextInput<'_, NumberInputMessage>| {
                            o.size(10)
                                .style(style_strategy(self.number_inputs[0].value_outside_range()))
                        })
                        .map(|number_msg| CustomSliderMessage::NumberInputMessage(0, number_msg))).width(Length::FillPortion(1)
                ),
                container(slider(lower..=upper, self.value, CustomSliderMessage::Slide).step((upper - lower) / 200.0)).width(Length::FillPortion(6)),
                container(
                    self.number_inputs[1]
                        .adjust_then_view(|o: TextInput<'_, NumberInputMessage>| {
                            o.size(10)
                                .style(style_strategy(self.number_inputs[1].value_outside_range()))
                        })
                        .map(|number_msg| CustomSliderMessage::NumberInputMessage(1, number_msg))
                ).width(Length::FillPortion(1)),
            ].align_y(Vertical::Center),
            self.number_inputs[2].view().map(|number_msg| CustomSliderMessage::NumberInputMessage(2, number_msg)),
        ].into()
    }
}