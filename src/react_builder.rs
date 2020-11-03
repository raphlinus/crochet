use crate::Cx;

use crate::react_comp::{
    ButtonPressed, ButtonTarget, ElementListTarget, ElementTupleTarget, EmptyElementTarget,
    EventEnum, LabelTarget, VirtualDom,
};

pub trait ElementTree<ExplicitState> {
    type Event;
    type AggregateState: Default;
    type Target: VirtualDom<
        ExplicitState,
        Event = Self::Event,
        AggregateComponentState = Self::AggregateState,
    >;

    fn build(self, prev_state: Self::AggregateState) -> (Self::Target, Self::AggregateState);
}

pub struct Label<ExplicitState>(pub LabelTarget<ExplicitState>);
pub struct Button<ExplicitState>(pub ButtonTarget<ExplicitState>);

impl<ExplicitState> Label<ExplicitState> {
    pub fn new(text: impl Into<String>) -> Label<ExplicitState> {
        Label(LabelTarget(text.into(), Default::default()))
    }
}

impl<ExplicitState> Button<ExplicitState> {
    pub fn new(text: impl Into<String>) -> Button<ExplicitState> {
        Button(ButtonTarget(text.into(), Default::default()))
    }
}

impl<ExplicitState> ElementTree<ExplicitState> for Label<ExplicitState> {
    type Event = ();
    type AggregateState = ();
    type Target = LabelTarget<ExplicitState>;

    fn build(self, _prev_state: ()) -> (LabelTarget<ExplicitState>, ()) {
        (self.0, ())
    }
}

impl<ExplicitState> ElementTree<ExplicitState> for Button<ExplicitState> {
    type Event = ButtonPressed;
    type AggregateState = ();
    type Target = ButtonTarget<ExplicitState>;

    fn build(self, _prev_state: ()) -> (ButtonTarget<ExplicitState>, ()) {
        (self.0, ())
    }
}

pub struct ElementTuple<
    E0: ElementTree<ExplicitState>,
    E1: ElementTree<ExplicitState>,
    E2: ElementTree<ExplicitState>,
    E3: ElementTree<ExplicitState>,
    ExplicitState = (),
>(
    pub E0,
    pub E1,
    pub E2,
    pub E3,
    pub std::marker::PhantomData<ExplicitState>,
);

impl<
        ExplicitState,
        E0: ElementTree<ExplicitState>,
        E1: ElementTree<ExplicitState>,
        E2: ElementTree<ExplicitState>,
        E3: ElementTree<ExplicitState>,
    > ElementTree<ExplicitState> for ElementTuple<E0, E1, E2, E3, ExplicitState>
{
    type Event = EventEnum<E0::Event, E1::Event, E2::Event, E3::Event>;
    type AggregateState = (
        E0::AggregateState,
        E1::AggregateState,
        E2::AggregateState,
        E3::AggregateState,
    );
    type Target = ElementTupleTarget<E0::Target, E1::Target, E2::Target, E3::Target, ExplicitState>;

    fn build(self, prev_state: Self::AggregateState) -> (Self::Target, Self::AggregateState) {
        let (t0, s0) = self.0.build(prev_state.0);
        let (t1, s1) = self.1.build(prev_state.1);
        let (t2, s2) = self.2.build(prev_state.2);
        let (t3, s3) = self.3.build(prev_state.3);

        (
            ElementTupleTarget(t0, t1, t2, t3, Default::default()),
            (s0, s1, s2, s3),
        )
    }
}

pub struct EmptyElement<ExplicitState = ()>(pub std::marker::PhantomData<ExplicitState>);

impl<ExplicitState> EmptyElement<ExplicitState> {
    pub fn new() -> EmptyElement<ExplicitState> {
        EmptyElement(Default::default())
    }
}

impl<ExplicitState> ElementTree<ExplicitState> for EmptyElement<ExplicitState> {
    type Event = ();
    type AggregateState = ();
    type Target = EmptyElementTarget<ExplicitState>;

    fn build(self, _prev_state: ()) -> (EmptyElementTarget<ExplicitState>, ()) {
        (EmptyElementTarget(Default::default()), ())
    }
}

pub struct ElementList<Child: ElementTree<ExplicitState>, ExplicitState = ()> {
    pub elements: Vec<(String, Child)>,
    pub _state: std::marker::PhantomData<ExplicitState>,
}

impl<ExplicitState, Child: ElementTree<ExplicitState>> ElementTree<ExplicitState>
    for ElementList<Child, ExplicitState>
{
    type Event = (usize, Child::Event);
    type AggregateState = Vec<(String, Child::AggregateState)>;
    type Target = ElementListTarget<Child::Target, ExplicitState>;

    fn build(self, prev_state: Self::AggregateState) -> (Self::Target, Self::AggregateState) {
        let mut prev_state = prev_state;
        let (elements, state): (Vec<_>, Vec<_>) = self
            .elements
            .into_iter()
            .map(|(key, comp)| {
                // FIXME, inefficient, and only works if items are only ever
                // appended at the end and keys are unique
                let existing = prev_state
                    .iter_mut()
                    .find(|(other_key, _)| key == *other_key);
                let (new_elem, new_state) = if let Some(comp_prev_state) = existing {
                    let (_, comp_prev_state) = std::mem::take(comp_prev_state);
                    comp.build(comp_prev_state)
                } else {
                    comp.build(Default::default())
                };
                ((key.clone(), new_elem), (key, new_state))
            })
            .unzip();
        (
            ElementListTarget {
                elements,
                _expl_state: Default::default(),
            },
            state,
        )
    }
}

pub struct WithEventTarget<
    Child: VirtualDom<ExplicitState>,
    Cb: Fn(&mut ExplicitState, &Child::Event),
    ExplicitState,
> {
    element: Child,
    callback: Cb,
    _state: std::marker::PhantomData<ExplicitState>,
}

impl<
        Child: VirtualDom<ParentComponentState>,
        Cb: Fn(&mut ParentComponentState, &Child::Event),
        ParentComponentState,
    > VirtualDom<ParentComponentState> for WithEventTarget<Child, Cb, ParentComponentState>
{
    type Event = Child::Event;
    type DomState = Child::DomState;
    type AggregateComponentState = Child::AggregateComponentState;

    fn update_value(&mut self, other: Self) {
        self.element.update_value(other.element);
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Child::DomState {
        self.element.init_tree(cx)
    }

    #[track_caller]
    fn apply_diff(
        &self,
        other: &Self,
        prev_state: Child::DomState,
        cx: &mut Cx,
    ) -> Child::DomState {
        self.element.apply_diff(&other.element, prev_state, cx)
    }

    fn process_event(
        &self,
        explicit_state: &mut ParentComponentState,
        children_state: &mut Child::AggregateComponentState,
        dom_state: &mut Child::DomState,
        cx: &mut Cx,
    ) -> Option<Child::Event> {
        let event = self
            .element
            .process_event(explicit_state, children_state, dom_state, cx);
        if let Some(event) = event.as_ref() {
            (self.callback)(explicit_state, event);
        }
        event
    }
}

pub struct WithEvent<
    Child: ElementTree<ExplicitState>,
    Cb: Fn(&mut ExplicitState, &<Child::Target as VirtualDom<ExplicitState>>::Event),
    ExplicitState = (),
> {
    pub element: Child,
    pub callback: Cb,
    pub _state: std::marker::PhantomData<ExplicitState>,
}

impl<
        Child: ElementTree<ExplicitState>,
        ExplicitState,
        Cb: Fn(&mut ExplicitState, &<Child::Target as VirtualDom<ExplicitState>>::Event),
    > ElementTree<ExplicitState> for WithEvent<Child, Cb, ExplicitState>
{
    type Event = Child::Event;
    type AggregateState = Child::AggregateState;
    type Target = WithEventTarget<Child::Target, Cb, ExplicitState>;

    fn build(self, prev_state: Self::AggregateState) -> (Self::Target, Self::AggregateState) {
        let (element, state) = self.element.build(prev_state);
        (
            WithEventTarget {
                element,
                callback: self.callback,
                _state: Default::default(),
            },
            state,
        )
    }
}

pub struct ComponentCallerTarget<
    ParentComponentState,
    ChildComponentState: Default,
    Child: VirtualDom<ChildComponentState>,
>(
    Child,
    std::marker::PhantomData<ParentComponentState>,
    std::marker::PhantomData<ChildComponentState>,
);

impl<
        ParentComponentState,
        ChildComponentState: Default,
        Child: VirtualDom<ChildComponentState>,
    > VirtualDom<ParentComponentState>
    for ComponentCallerTarget<ParentComponentState, ChildComponentState, Child>
{
    type Event = Child::Event;
    type DomState = Child::DomState;
    type AggregateComponentState = (ChildComponentState, Child::AggregateComponentState);

    fn update_value(&mut self, other: Self) {
        self.0.update_value(other.0);
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Self::DomState {
        self.0.init_tree(cx)
    }

    #[track_caller]
    fn apply_diff(&self, other: &Self, prev_state: Self::DomState, cx: &mut Cx) -> Self::DomState {
        self.0.apply_diff(&other.0, prev_state, cx)
    }

    fn process_event(
        &self,
        _explicit_state: &mut ParentComponentState,
        children_state: &mut Self::AggregateComponentState,
        dom_state: &mut Self::DomState,
        cx: &mut Cx,
    ) -> Option<Self::Event> {
        self.0
            .process_event(&mut children_state.0, &mut children_state.1, dom_state, cx)
    }
}

pub struct ComponentCaller<
    CompExplicitState,
    Props,
    ReturnedTree: ElementTree<CompExplicitState>,
    Comp: Fn(&CompExplicitState, Props) -> ReturnedTree,
    ExplicitState = (),
> {
    pub component: Comp,
    pub props: Props,
    pub _state: std::marker::PhantomData<CompExplicitState>,
    pub _tree: std::marker::PhantomData<ReturnedTree>,
    pub _expl_state: std::marker::PhantomData<ExplicitState>,
}

impl<
        ExplicitState,
        CompExplicitState,
        Props,
        ReturnedTree: ElementTree<CompExplicitState>,
        Comp: Fn(&CompExplicitState, Props) -> ReturnedTree,
    > ComponentCaller<CompExplicitState, Props, ReturnedTree, Comp, ExplicitState>
{
    pub fn prepare(
        component: Comp,
        props: Props,
    ) -> ComponentCaller<CompExplicitState, Props, ReturnedTree, Comp, ExplicitState> {
        ComponentCaller {
            component,
            props,
            _state: Default::default(),
            _tree: Default::default(),
            _expl_state: Default::default(),
        }
    }
}

impl<
        ExplicitState,
        CompExplicitState: Default,
        Props,
        ReturnedTree: ElementTree<CompExplicitState>,
        Comp: Fn(&CompExplicitState, Props) -> ReturnedTree,
    > ElementTree<ExplicitState>
    for ComponentCaller<CompExplicitState, Props, ReturnedTree, Comp, ExplicitState>
{
    type Event = ReturnedTree::Event;
    type AggregateState = (CompExplicitState, ReturnedTree::AggregateState);
    type Target = ComponentCallerTarget<ExplicitState, CompExplicitState, ReturnedTree::Target>;

    fn build(self, prev_state: Self::AggregateState) -> (Self::Target, Self::AggregateState) {
        let element_tree = (self.component)(&prev_state.0, self.props);
        let (element, component_state) = element_tree.build(prev_state.1);
        (
            ComponentCallerTarget(element, Default::default(), Default::default()),
            (prev_state.0, component_state),
        )
    }
}

pub struct ReactApp<
    RootCompState,
    ReturnedTree: ElementTree<RootCompState>,
    Comp: Fn(&RootCompState, ()) -> ReturnedTree,
> {
    pub root_component: ComponentCaller<RootCompState, (), ReturnedTree, Comp, ()>,
    pub component_state: (RootCompState, ReturnedTree::AggregateState),
    pub vdom: Option<ReturnedTree::Target>,
    pub vdom_state: Option<<ReturnedTree::Target as VirtualDom<RootCompState>>::DomState>,
}

impl<
        RootCompState,
        ReturnedTree: ElementTree<RootCompState>,
        Comp: Fn(&RootCompState, ()) -> ReturnedTree,
    > ReactApp<RootCompState, ReturnedTree, Comp>
{
    pub fn new(
        root_component: Comp,
        root_state: RootCompState,
    ) -> ReactApp<RootCompState, ReturnedTree, Comp> {
        ReactApp {
            root_component: ComponentCaller {
                component: root_component,
                props: (),
                _state: Default::default(),
                _tree: Default::default(),
                _expl_state: Default::default(),
            },
            component_state: (root_state, Default::default()),
            vdom: None,
            vdom_state: None,
        }
    }

    #[track_caller]
    pub fn run(&mut self, cx: &mut Cx) {
        let (vdom, component_state) = (self.root_component.component)(&self.component_state.0, ())
            .build(std::mem::take(&mut self.component_state.1));
        self.component_state.1 = component_state;

        let mut vdom_state;

        if let Some(prev_vdom) = self.vdom.as_mut() {
            let prev_vdom_state = self.vdom_state.take().unwrap();
            vdom_state = vdom.apply_diff(prev_vdom, prev_vdom_state, cx);
            prev_vdom.update_value(vdom);

            if let Some(_event) = prev_vdom.process_event(
                &mut self.component_state.0,
                &mut self.component_state.1,
                &mut vdom_state,
                cx,
            ) {
                // callback(&event, &mut self.state);
            }
        } else {
            vdom_state = vdom.init_tree(cx);
            self.vdom = Some(vdom);
        }

        self.vdom_state = Some(vdom_state);
    }
}
