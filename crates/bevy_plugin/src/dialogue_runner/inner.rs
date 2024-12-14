use crate::prelude::*;
use bevy::prelude::*;

pub(crate) fn inner_dialogue_runner_plugin(_app: &mut App) {}

/// Proxy for some functionality of the [`Dialogue`] used by [`DialogueRunner`].
/// Constructed by [`DialogueRunner::inner`]. Serves advanced use cases.
#[derive(Debug)]
pub struct InnerDialogue<'a>(pub(crate) &'a Dialogue);

/// Mutable proxy for some functionality of the [`Dialogue`] used by [`DialogueRunner`].
/// Constructed by [`DialogueRunner::inner_mut`]. Serves advanced use cases.
#[derive(Debug)]
pub struct InnerDialogueMut<'a>(pub(crate) &'a mut Dialogue);

impl InnerDialogue<'_> {
    /// Proxy for [`Dialogue::node_names`].
    pub fn node_names(&self) -> impl Iterator<Item = &str> {
        self.0.node_names().unwrap()
    }

    /// Proxy for [`Dialogue::get_line_id_for_node`].
    #[must_use]
    pub fn get_line_id_for_node(&self, node_name: &str) -> Option<LineId> {
        self.0.get_line_id_for_node(node_name)
    }

    /// Proxy for [`Dialogue::analyse`].
    #[must_use]
    pub fn analyse(&self, context: &mut YarnAnalysisContext) -> &Self {
        self.0.analyse(context);
        self
    }
}

impl InnerDialogueMut<'_> {
    /// Proxy for [`Dialogue::node_names`].
    pub fn node_names(&self) -> impl Iterator<Item = &str> {
        self.0.node_names().unwrap()
    }

    /// Proxy for [`Dialogue::get_line_id_for_node`]
    #[must_use]
    pub fn get_line_id_for_node(&self, node_name: &str) -> Option<LineId> {
        self.0.get_line_id_for_node(node_name)
    }

    /// Proxy for [`Dialogue::set_node`]
    pub fn set_node(&mut self, name: impl Into<String>) -> Result<&mut Self> {
        self.0.set_node(name)?;
        Ok(self)
    }

    /// Proxy for [`Dialogue::analyse`].
    #[must_use]
    pub fn analyse(&self, context: &mut YarnAnalysisContext) -> &Self {
        self.0.analyse(context);
        self
    }
}
