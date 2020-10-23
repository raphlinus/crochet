
#[allow(unused_imports)]
use crate::{Button, Cx, DruidAppData, Id, Label, List, ListData, Row};

#[allow(unused_imports)]
use crate::react_comp::{ReactComponent, ComponentTuple, ComponentList, VDomLabel, VDomButton, VirtualDom, EmptyComponent};

pub trait VirtualDomBuilder<ExplicitState> {
    type Target : VirtualDom<ExplicitState>;

    fn build(self) -> Self::Target;
}


pub struct VDomLabelBuilder<ExplicitState>(pub VDomLabel<ExplicitState>);
pub struct VDomButtonBuilder<ExplicitState>(pub VDomButton<ExplicitState>);

impl<ExplicitState> VirtualDomBuilder<ExplicitState> for VDomLabelBuilder<ExplicitState> {
    type Target = VDomLabel<ExplicitState>;

    fn build(self) -> VDomLabel<ExplicitState> {
        self.0
    }
}

impl<ExplicitState> VirtualDomBuilder<ExplicitState> for VDomButtonBuilder<ExplicitState> {
    type Target = VDomButton<ExplicitState>;

    fn build(self) -> VDomButton<ExplicitState> {
        self.0
    }
}


pub struct ComponentTupleBuilder<
    C0 : VirtualDomBuilder<ExplicitState>,
    C1 : VirtualDomBuilder<ExplicitState>,
    C2 : VirtualDomBuilder<ExplicitState>,
    C3 : VirtualDomBuilder<ExplicitState>,
    ExplicitState = (),
>(pub C0, pub C1, pub C2, pub C3, pub std::marker::PhantomData<ExplicitState>);

impl<
    ExplicitState,
    C0 : VirtualDomBuilder<ExplicitState>,
    C1 : VirtualDomBuilder<ExplicitState>,
    C2 : VirtualDomBuilder<ExplicitState>,
    C3 : VirtualDomBuilder<ExplicitState>,
> VirtualDomBuilder<ExplicitState> for ComponentTupleBuilder<C0, C1, C2, C3, ExplicitState> {
    type Target = ComponentTuple<
        C0::Target,
        C1::Target,
        C2::Target,
        C3::Target,
        ExplicitState,
    >;

    fn build(self) -> Self::Target {
        ComponentTuple (
            self.0.build(),
            self.1.build(),
            self.2.build(),
            self.3.build(),
            Default::default(),
        )
    }
}


pub struct EmptyComponentBuilder<ExplicitState = ()>(pub std::marker::PhantomData<ExplicitState>);

impl<ExplicitState> VirtualDomBuilder<ExplicitState> for EmptyComponentBuilder<ExplicitState> {
    type Target = EmptyComponent<ExplicitState>;
    fn build(self) -> EmptyComponent<ExplicitState> {
        EmptyComponent (Default::default())
    }
}


pub struct ComponentListBuilder<Comp : VirtualDomBuilder<ExplicitState>, ExplicitState = ()> {
    pub components: Vec<(String, Comp)>,
    pub _state: std::marker::PhantomData<ExplicitState>,
}

impl<ExplicitState, Comp : VirtualDomBuilder<ExplicitState>> VirtualDomBuilder<ExplicitState> for ComponentListBuilder<Comp, ExplicitState> {
    type Target = ComponentList<Comp::Target, ExplicitState>;

    fn build(self) -> Self::Target {
        ComponentList {
            components: self.components.into_iter().map(|(key, comp)| {
                // TODO - handle identity
                (key, comp.build())
            }).collect(),
            _expl_state: Default::default(),
        }
    }
}


pub struct WithEvent<
    Comp : VirtualDom<ExplicitState>,
    Cb : Fn(&mut ExplicitState, &Comp::Event),
    ExplicitState,
> {
    component: Comp,
    callback: Cb,
    _state: std::marker::PhantomData<ExplicitState>,
}

impl <
    Comp : VirtualDom<ExplicitState>,
    Cb : Fn(&mut ExplicitState, &Comp::Event),
    ExplicitState,
> VirtualDom<ExplicitState> for WithEvent< Comp, Cb, ExplicitState > {
    type Event = Comp::Event;
    type State = Comp::State;

    fn update_value(&mut self, other: Self) {
        self.component.update_value(other.component);
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Comp::State {
        self.component.init_tree(cx)
    }

    #[track_caller]
    fn apply_diff(&self, other: &Self, prev_state: Comp::State, cx: &mut Cx) -> Comp::State {
        self.component.apply_diff(&other.component, prev_state, cx)
    }

    fn process_event(&self, explicit_state: &mut ExplicitState, state: &mut Comp::State, cx: &mut Cx) -> Option<Comp::Event> {
        let event = self.component.process_event(explicit_state, state, cx);
        if let Some(event) = event.as_ref() {
            (self.callback)(explicit_state, event);
        }
        event
    }
}


pub struct WithEventBuilder<
    Comp : VirtualDomBuilder<ExplicitState>,
    Cb : Fn(&mut ExplicitState, &<Comp::Target as VirtualDom<ExplicitState>>::Event),
    ExplicitState = ()
> {
    pub component: Comp,
    pub callback: Cb,
    pub _state: std::marker::PhantomData<ExplicitState>,
}

impl <
    Comp : VirtualDomBuilder<ExplicitState>,
    ExplicitState,
    Cb : Fn(&mut ExplicitState, &<Comp::Target as VirtualDom<ExplicitState>>::Event),
> VirtualDomBuilder<ExplicitState> for WithEventBuilder< Comp, Cb, ExplicitState > {
    type Target = WithEvent< Comp::Target, Cb, ExplicitState >;

    fn build(self) -> Self::Target {
        WithEvent {
            component: self.component.build(),
            callback: self.callback,
            _state: Default::default(),
        }
    }

}


pub struct WithState<Comp : VirtualDom<InternalState>, InternalState : Default, ExplicitState>(Comp, std::marker::PhantomData<InternalState>, std::marker::PhantomData<ExplicitState>);

impl<Comp : VirtualDom<InternalState>, InternalState : Default, ExplicitState> VirtualDom<ExplicitState> for WithState<Comp, InternalState, ExplicitState> {
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

    fn process_event(&self, _explicit_state: &mut ExplicitState, state: &mut Self::State, cx: &mut Cx) -> Option<Self::Event> {
        self.0.process_event(&mut state.1, &mut state.0, cx)
    }
}


// TODO - I really need to normalize names
pub struct ComponentBuilder<State : Default, Props, VDom : VirtualDom<State>, Cb : Fn(&State, Props) -> VDom, ExplicitState = ()> {
    pub component : Cb,
    pub props : Props,
    pub _vdom : std::marker::PhantomData<VDom>,
    pub _state : std::marker::PhantomData<State>,
    pub _expl_state : std::marker::PhantomData<ExplicitState>,
}

impl<ExplicitState, State : Default, Props, VDom : VirtualDom<State>, Cb : Fn(&State, Props) -> VDom> VirtualDomBuilder<ExplicitState> for ComponentBuilder<State, Props, VDom, Cb, ExplicitState> {
    type Target = WithState<VDom, State, ExplicitState>;

    fn build(self) -> Self::Target {
        WithState((self.component)(&State::default(), self.props), Default::default(), Default::default())
    }
}
