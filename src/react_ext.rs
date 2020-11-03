use crate::react_builder::{ElementTree, WithEvent};
use crate::react_comp::VirtualDom;

pub trait VirtualDomBuilderExt<ExplicitState>: ElementTree<ExplicitState> + Sized {
    fn with_event<
        Cb: Fn(&mut ExplicitState, &<Self::Target as VirtualDom<ExplicitState>>::Event),
    >(
        self,
        callback: Cb,
    ) -> WithEvent<Self, Cb, ExplicitState> {
        WithEvent {
            element: self,
            callback,
            _state: Default::default(),
        }
    }
}

impl<ExplicitState, VDB: ElementTree<ExplicitState>> VirtualDomBuilderExt<ExplicitState>
    for VDB
{
}
