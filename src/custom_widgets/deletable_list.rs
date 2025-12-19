use iced::Element;
use iced::Length;
use iced::widget::Column;
use iced::widget::{container, text, hover, button};
use iced::{Center, Right};
use iced::Padding;


#[derive(Debug, Clone)]
pub enum DeletableListMessage<T: Clone> {
    Delete(usize),
    Item(usize, T),
}

/// Custom widget for handling a list of items. Each item is deletable on the GUI with
/// a "X" delete button that appears on top-right of the item upon mouse hover.
/// 
/// Each Item is also paired up with some given ID information for ease of tracking.
pub struct DeletableList<Id, Item, ItemMessage, Update, View> 
where
    Id: Clone + PartialEq,
    ItemMessage: Clone,
    Update: Fn(&mut Item, ItemMessage),
    View: Fn(&Item) -> Element<'_, ItemMessage>,
{
    pub data: Vec<(Id, Item)>,
    item_view: View,
    item_update: Update,
}

impl<Id, Item, ItemMessage, Update, View> DeletableList<Id, Item, ItemMessage, Update, View> 
where
    Id: Clone + PartialEq,
    ItemMessage: Clone,
    Update: Fn(&mut Item, ItemMessage),
    View: Fn(&Item) -> Element<'_, ItemMessage>,
{
    pub fn new(update: Update, view: View) -> DeletableList<Id, Item, ItemMessage, Update, View> 
    where
        Update: Fn(&mut Item, ItemMessage),
        View: Fn(&Item) -> Element<'_, ItemMessage>
    {
        Self {
            data: Vec::new(),
            item_view: view,
            item_update: update,
        }
    }

    pub fn update(&mut self, message: DeletableListMessage<ItemMessage>) {
        match message {
            DeletableListMessage::Delete(i) => {
                self.data.remove(i);
            }
            DeletableListMessage::Item(i, message) => {
                (self.item_update)(&mut self.data[i].1, message);
            }
        }
    }

    /// Checks if a given ID is already held within the list.
    /// 
    /// Returns Some(i) with i being the lowest index where the ID exists. Returns None
    /// elsewise.
    #[allow(non_snake_case)]
    pub fn scan_ID(&self, id: &Id) -> Option<usize> {
        return self.data.iter().position(|(item_id,_)| *id == *item_id);
    }

    /// Appends the item to the bottom of the list if the given ID is not already present.
    pub fn unique_push(&mut self, id: Id, item: Item) {
        if self.scan_ID(&id).is_none() {
            self.data.push((id, item));
        }
    }

    pub fn view<Format>(&self, formatting: Format) -> Element<'_, DeletableListMessage<ItemMessage>> 
    where Format: Fn(Column<'_, DeletableListMessage<ItemMessage>>) -> Column<'_, DeletableListMessage<ItemMessage>>
    {
        (formatting)(
            Column::from_iter(self.data.iter()
                .enumerate()
                .map(|(i, (_, x))| hover(
                        (self.item_view)(x).map(move |message| DeletableListMessage::Item(i, message)),
                        container(button(text("X").size(10).align_x(Center)).width(15.0).height(15.0).padding(Padding::ZERO).on_press(DeletableListMessage::Delete(i))).width(Length::Fill).align_x(Right)
                    )
                )
        )).into()
    }
}
