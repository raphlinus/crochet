
#[allow(unused_imports)]
use crate::{Button, Cx, DruidAppData, Id, Label, List, ListData, Row};

#[allow(unused_imports)]
use crate::react_comp::{ReactComponent, ComponentTuple, ComponentList, VDomLeaf, VirtualDom, EmptyComponent};

pub trait VirtualDomBuilder {
    type Target : VirtualDom;

    fn build(self) -> Self::Target;
}


pub struct VDomLeafBuilder(pub VDomLeaf);

impl VirtualDomBuilder for VDomLeafBuilder {
    type Target = VDomLeaf;

    fn build(self) -> VDomLeaf {
        self.0
    }
}


pub struct ComponentTupleBuilder<
    C0 : VirtualDomBuilder,
    C1 : VirtualDomBuilder,
    C2 : VirtualDomBuilder,
    C3 : VirtualDomBuilder,
>(pub C0, pub C1, pub C2, pub C3);

impl<
    C0 : VirtualDomBuilder,
    C1 : VirtualDomBuilder,
    C2 : VirtualDomBuilder,
    C3 : VirtualDomBuilder,
> VirtualDomBuilder for ComponentTupleBuilder<C0, C1, C2, C3> {
    type Target = ComponentTuple<
        C0::Target,
        C1::Target,
        C2::Target,
        C3::Target,
    >;

    fn build(self) -> Self::Target {
        ComponentTuple (
            self.0.build(),
            self.1.build(),
            self.2.build(),
            self.3.build(),
        )
    }
}


pub struct EmptyComponentBuilder();

impl VirtualDomBuilder for EmptyComponentBuilder {
    type Target = EmptyComponent;
    fn build(self) -> EmptyComponent {
        EmptyComponent ()
    }
}


pub struct ComponentListBuilder<Comp : VirtualDomBuilder> {
    pub components: Vec<(String, Comp)>
}

impl<Comp : VirtualDomBuilder> VirtualDomBuilder for ComponentListBuilder<Comp> {
    type Target = ComponentList<Comp::Target>;

    fn build(self) -> Self::Target {
        ComponentList {
            components: self.components.into_iter().map(|(key, comp)| {
                // TODO - handle identity
                (key, comp.build())
            }).collect()
        }
    }
}


pub struct WithState<Comp : VirtualDom, InternalState : Default>(Comp, std::marker::PhantomData<InternalState>);

impl<Comp : VirtualDom, InternalState : Default> VirtualDom for WithState<Comp, InternalState> {
    type Event = Comp::Event;
    type State = (Comp::State, InternalState);

    fn update_value(&mut self, other: Self) {
        self.0.update_value(other.0);
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Self::State {
        (
            self.0.init_tree(cx),
            InternalState::default(),
        )
    }

    #[track_caller]
    fn apply_diff(&self, other: &Self, prev_state: Self::State, cx: &mut Cx) -> Self::State {
        (
            self.0.apply_diff(&other.0, prev_state.0, cx),
            prev_state.1,
        )
    }

    fn process_event(&self, state: &mut Self::State, cx: &mut Cx) -> Option<Self::Event> {
        self.0.process_event(&mut state.0, cx)
    }
}


// TODO - I really need to normalize names
pub struct ComponentBuilder<State : Default, Props, VDom : VirtualDom, Cb : Fn(&State, Props) -> VDom> {
    pub component : Cb,
    pub props : Props,
    pub _vdom : std::marker::PhantomData<VDom>,
    pub _state : std::marker::PhantomData<State>,
}

impl<State : Default, Props, VDom : VirtualDom, Cb : Fn(&State, Props) -> VDom> VirtualDomBuilder for ComponentBuilder<State, Props, VDom, Cb> {
    type Target = WithState<VDom, State>;

    fn build(self) -> Self::Target {
        WithState((self.component)(&State::default(), self.props), Default::default())
    }
}
