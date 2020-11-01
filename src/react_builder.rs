#[allow(unused_imports)]
use crate::{Button, Cx, DruidAppData, Id, Label, List, ListData, Row};

#[allow(unused_imports)]
use crate::react_comp::{
    ComponentListTarget, ComponentTupleTarget, EmptyComponentTarget, VDomButtonTarget,
    VDomLabelTarget, VirtualDom,
};

pub trait VirtualDomBuilder<ExplicitState> {
    type Target: VirtualDom<ExplicitState>;

    fn build(self) -> Self::Target;
}

pub struct VDomLabel<ExplicitState>(pub VDomLabelTarget<ExplicitState>);
pub struct VDomButton<ExplicitState>(pub VDomButtonTarget<ExplicitState>);

impl<ExplicitState> VDomLabel<ExplicitState> {
    pub fn new(text: impl Into<String>) -> VDomLabel<ExplicitState> {
        VDomLabel(VDomLabelTarget(text.into(), Default::default()))
    }
}

impl<ExplicitState> VDomButton<ExplicitState> {
    pub fn new(text: impl Into<String>) -> VDomButton<ExplicitState> {
        VDomButton(VDomButtonTarget(text.into(), Default::default()))
    }
}

impl<ExplicitState> VirtualDomBuilder<ExplicitState> for VDomLabel<ExplicitState> {
    type Target = VDomLabelTarget<ExplicitState>;

    fn build(self) -> VDomLabelTarget<ExplicitState> {
        self.0
    }
}

impl<ExplicitState> VirtualDomBuilder<ExplicitState> for VDomButton<ExplicitState> {
    type Target = VDomButtonTarget<ExplicitState>;

    fn build(self) -> VDomButtonTarget<ExplicitState> {
        self.0
    }
}

pub struct ComponentTuple<
    C0: VirtualDomBuilder<ExplicitState>,
    C1: VirtualDomBuilder<ExplicitState>,
    C2: VirtualDomBuilder<ExplicitState>,
    C3: VirtualDomBuilder<ExplicitState>,
    ExplicitState = (),
>(
    pub C0,
    pub C1,
    pub C2,
    pub C3,
    pub std::marker::PhantomData<ExplicitState>,
);

impl<
        ExplicitState,
        C0: VirtualDomBuilder<ExplicitState>,
        C1: VirtualDomBuilder<ExplicitState>,
        C2: VirtualDomBuilder<ExplicitState>,
        C3: VirtualDomBuilder<ExplicitState>,
    > VirtualDomBuilder<ExplicitState> for ComponentTuple<C0, C1, C2, C3, ExplicitState>
{
    type Target =
        ComponentTupleTarget<C0::Target, C1::Target, C2::Target, C3::Target, ExplicitState>;

    fn build(self) -> Self::Target {
        ComponentTupleTarget(
            self.0.build(),
            self.1.build(),
            self.2.build(),
            self.3.build(),
            Default::default(),
        )
    }
}

pub struct EmptyComponent<ExplicitState = ()>(pub std::marker::PhantomData<ExplicitState>);

impl<ExplicitState> EmptyComponent<ExplicitState> {
    pub fn new() -> EmptyComponent<ExplicitState> {
        EmptyComponent(Default::default())
    }
}

impl<ExplicitState> VirtualDomBuilder<ExplicitState> for EmptyComponent<ExplicitState> {
    type Target = EmptyComponentTarget<ExplicitState>;
    fn build(self) -> EmptyComponentTarget<ExplicitState> {
        EmptyComponentTarget(Default::default())
    }
}

pub struct ComponentList<Comp: VirtualDomBuilder<ExplicitState>, ExplicitState = ()> {
    pub components: Vec<(String, Comp)>,
    pub _state: std::marker::PhantomData<ExplicitState>,
}

impl<ExplicitState, Comp: VirtualDomBuilder<ExplicitState>> VirtualDomBuilder<ExplicitState>
    for ComponentList<Comp, ExplicitState>
{
    type Target = ComponentListTarget<Comp::Target, ExplicitState>;

    fn build(self) -> Self::Target {
        ComponentListTarget {
            components: self
                .components
                .into_iter()
                .map(|(key, comp)| {
                    // TODO - handle identity
                    (key, comp.build())
                })
                .collect(),
            _expl_state: Default::default(),
        }
    }
}

pub struct WithEventTarget<
    Comp: VirtualDom<ExplicitState>,
    Cb: Fn(&mut ExplicitState, &Comp::Event),
    ExplicitState,
> {
    component: Comp,
    callback: Cb,
    _state: std::marker::PhantomData<ExplicitState>,
}

impl<Comp: VirtualDom<ExplicitState>, Cb: Fn(&mut ExplicitState, &Comp::Event), ExplicitState>
    VirtualDom<ExplicitState> for WithEventTarget<Comp, Cb, ExplicitState>
{
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

    fn process_event(
        &self,
        explicit_state: &mut ExplicitState,
        state: &mut Comp::State,
        cx: &mut Cx,
    ) -> Option<Comp::Event> {
        let event = self.component.process_event(explicit_state, state, cx);
        if let Some(event) = event.as_ref() {
            (self.callback)(explicit_state, event);
        }
        event
    }
}

pub struct WithEvent<
    Comp: VirtualDomBuilder<ExplicitState>,
    Cb: Fn(&mut ExplicitState, &<Comp::Target as VirtualDom<ExplicitState>>::Event),
    ExplicitState = (),
> {
    pub component: Comp,
    pub callback: Cb,
    pub _state: std::marker::PhantomData<ExplicitState>,
}

impl<
        Comp: VirtualDomBuilder<ExplicitState>,
        ExplicitState,
        Cb: Fn(&mut ExplicitState, &<Comp::Target as VirtualDom<ExplicitState>>::Event),
    > VirtualDomBuilder<ExplicitState> for WithEvent<Comp, Cb, ExplicitState>
{
    type Target = WithEventTarget<Comp::Target, Cb, ExplicitState>;

    fn build(self) -> Self::Target {
        WithEventTarget {
            component: self.component.build(),
            callback: self.callback,
            _state: Default::default(),
        }
    }
}

pub struct WithStateTarget<Comp: VirtualDom<InternalState>, InternalState: Default, ExplicitState>(
    Comp,
    std::marker::PhantomData<InternalState>,
    std::marker::PhantomData<ExplicitState>,
);

impl<Comp: VirtualDom<InternalState>, InternalState: Default, ExplicitState>
    VirtualDom<ExplicitState> for WithStateTarget<Comp, InternalState, ExplicitState>
{
    type Event = Comp::Event;
    type State = (Comp::State, InternalState);

    fn update_value(&mut self, other: Self) {
        self.0.update_value(other.0);
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Self::State {
        (self.0.init_tree(cx), InternalState::default())
    }

    #[track_caller]
    fn apply_diff(&self, other: &Self, prev_state: Self::State, cx: &mut Cx) -> Self::State {
        (self.0.apply_diff(&other.0, prev_state.0, cx), prev_state.1)
    }

    fn process_event(
        &self,
        _explicit_state: &mut ExplicitState,
        state: &mut Self::State,
        cx: &mut Cx,
    ) -> Option<Self::Event> {
        self.0.process_event(&mut state.1, &mut state.0, cx)
    }
}

// TODO - I really need to normalize names
pub struct ComponentBuilder<
    State: Default,
    Props,
    VDom: VirtualDom<State>,
    Cb: Fn(&State, Props) -> VDom,
    ExplicitState = (),
> {
    pub component: Cb,
    pub props: Props,
    pub _vdom: std::marker::PhantomData<VDom>,
    pub _state: std::marker::PhantomData<State>,
    pub _expl_state: std::marker::PhantomData<ExplicitState>,
}

impl<
        ExplicitState,
        State: Default,
        Props,
        VDom: VirtualDom<State>,
        Cb: Fn(&State, Props) -> VDom,
    > ComponentBuilder<State, Props, VDom, Cb, ExplicitState>
{
    pub fn prepare(
        component: Cb,
        props: Props,
    ) -> ComponentBuilder<State, Props, VDom, Cb, ExplicitState> {
        ComponentBuilder {
            component,
            props,
            _vdom: Default::default(),
            _state: Default::default(),
            _expl_state: Default::default(),
        }
    }
}

impl<
        ExplicitState,
        State: Default,
        Props,
        VDom: VirtualDom<State>,
        Cb: Fn(&State, Props) -> VDom,
    > VirtualDomBuilder<ExplicitState> for ComponentBuilder<State, Props, VDom, Cb, ExplicitState>
{
    type Target = WithStateTarget<VDom, State, ExplicitState>;

    fn build(self) -> Self::Target {
        WithStateTarget(
            (self.component)(&State::default(), self.props),
            Default::default(),
            Default::default(),
        )
    }
}

pub struct ReactApp<ExplicitState, Props, VDom: VirtualDom<ExplicitState>, Cb>
where
    Cb: Fn(&ExplicitState, &Props) -> VDom,
{
    pub state: ExplicitState,
    pub root_component: Cb,
    pub prev_vdom: Option<VDom>,
    pub prev_vdom_state: Option<VDom::State>,
    pub _props: std::marker::PhantomData<Props>,
}

impl<ExplicitState, Props, VDom: VirtualDom<ExplicitState>, Cb>
    ReactApp<ExplicitState, Props, VDom, Cb>
where
    Cb: Fn(&ExplicitState, &Props) -> VDom,
{
    #[track_caller]
    pub fn run(
        &mut self,
        cx: &mut Cx,
        props: &Props,
        callback: impl Fn(&VDom::Event, &mut ExplicitState),
    ) {
        let vdom = (self.root_component)(&self.state, props);

        if let Some(prev_vdom) = self.prev_vdom.as_mut() {
            let prev_vdom_state = self.prev_vdom_state.take().unwrap();
            let mut vdom_state = vdom.apply_diff(prev_vdom, prev_vdom_state, cx);

            if let Some(event) = vdom.process_event(&mut self.state, &mut vdom_state, cx) {
                callback(&event, &mut self.state);
            }

            prev_vdom.update_value(vdom);
            self.prev_vdom_state = Some(vdom_state);
        } else {
            let vdom_state = vdom.init_tree(cx);
            self.prev_vdom = Some(vdom);
            self.prev_vdom_state = Some(vdom_state);
        }
    }
}
