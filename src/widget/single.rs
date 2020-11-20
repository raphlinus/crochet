use crate::{any_widget::AnyWidget, DruidAppData, MutIterItem};
use druid::{EventCtx, WidgetPod};

/// Helper struct for single-child widgets.
pub struct SingleChild {
    children: Vec<WidgetPod<DruidAppData, AnyWidget>>,
}

impl SingleChild {
    /// Create a new single-child helper.
    pub fn new() -> Self {
        SingleChild {
            children: Vec::new(),
        }
    }

    /// Get the only interesting child.
    pub fn get(&self) -> Option<&WidgetPod<DruidAppData, AnyWidget>> {
        self.children.get(0)
    }

    /// Get the only interesting child mutably.
    pub fn get_mut(&mut self) -> Option<&mut WidgetPod<DruidAppData, AnyWidget>> {
        self.children.get_mut(0)
    }

    /// Apply mutations, potentially changing the interesting child.
    ///
    /// If the interesting child changed, it will call [`EventCtx::children_changed`].
    pub fn mutate(&mut self, ctx: &mut EventCtx, mut_iter: crate::MutationIter) {
        let mut ix = 0;
        let mut children_changed = false;
        for item in mut_iter {
            match item {
                MutIterItem::Skip(n) => {
                    ix += n;
                }
                MutIterItem::Delete(n) => {
                    self.children.drain(ix..ix + n);
                    if ix == 0 {
                        children_changed = true;
                    }
                }
                MutIterItem::Insert(id, body, child_iter) => {
                    let child = AnyWidget::mutate_insert(ctx, id, body, child_iter);
                    self.children.insert(ix, WidgetPod::new(child));
                    if ix == 0 {
                        children_changed = true;
                    }
                    ix += 1;
                }
                MutIterItem::Update(body, child_iter) => {
                    self.children[ix].with_event_context(ctx, |child, ctx| {
                        child.mutate_update(ctx, body, child_iter);
                    });
                    ix += 1;
                }
            }
        }
        if children_changed {
            ctx.children_changed();
        }
        if self.children.len() > 1 {
            // TODO: This should probably panic in debug mode using `debug_panic` from Druid.
            log::error!(
                "Single-child widget was created with {} children.",
                self.children.len()
            );
        }
    }
}
