/// Module for all custom Iced widgets to be used for the project

pub mod number_input;
pub use number_input::{NumberInput, NumberInputMessage};

pub mod payoff_chart;
pub use payoff_chart::{PayoffChart, PayoffChartMessage};

pub mod custom_slider;
pub use custom_slider::{CustomSlider, CustomSliderMessage};

pub mod deletable_list;
pub use deletable_list::{DeletableList, DeletableListMessage};