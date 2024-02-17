use super::{Gui, Message};
use iced::{
    widget::{checkbox, Checkbox},
    Element,
};

/// Parses the currently selected launcher and returns the available and
/// currently downloaded proton versions
pub fn download_list(gui_data: &Gui) -> Vec<Element<Message>> {
    let mut download_list: Vec<Element<Message>> = Vec::new();

    if let Some(releases) = &gui_data.release_data {
        for (launcher, wine_versions) in releases {
            // For the currently selected launcher
            if launcher == &gui_data.selected_launcher {
                //Create a list of Wine versions as a checkbox to be downloaded.
                for wine_version in wine_versions {
                    let list_item: Checkbox<'_, Message> =
                        checkbox(wine_version.tag_name.clone(), wine_version.selected).on_toggle(
                            |selected| {
                                // TODO possibly pass the SelectVersion message a struct containing the tuple info
                                Message::SelectVersion(*launcher, wine_version.clone(), selected)
                            },
                        );

                    // Extend the list item to a row of extra info
                    download_list.push(list_item.into());
                }
            }
        }
    }

    download_list
}
