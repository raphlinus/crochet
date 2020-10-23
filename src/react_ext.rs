use crate::react_comp::VirtualDom;
use crate::react_builder::{VirtualDomBuilder, WithEvent};

pub trait VirtualDomBuilderExt<ExplicitState>: VirtualDomBuilder<ExplicitState> + Sized {
    // TODO - shorten
    fn with_event<Cb : Fn(&mut ExplicitState, &<Self::Target as VirtualDom<ExplicitState>>::Event)>(self, callback: Cb) -> WithEvent<Self, Cb, ExplicitState> {
        WithEvent {
            component: self,
            callback,
            _state: Default::default(),
        }
    }
}

impl<ExplicitState, VDB : VirtualDomBuilder<ExplicitState>> VirtualDomBuilderExt<ExplicitState> for VDB {}
