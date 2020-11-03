use crate::{Cx, Id};

use std::panic::Location;

// TODO
//
// Improve performance
// Remove track_caller

pub trait VirtualDom<ParentComponentState> {
    type Event;
    type DomState;
    type AggregateComponentState;

    // update_value is intended to enable memoize-style HOC
    // where instead of returning a vdom node, it returns
    // something along the lines of struct KeepEverythingAsItWas()
    // Ugh. I'm not explaining this well.
    fn update_value(&mut self, other: Self);

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Self::DomState;

    fn apply_diff(&self, other: &Self, prev_state: Self::DomState, cx: &mut Cx) -> Self::DomState;

    fn process_event(
        &self,
        explicit_state: &mut ParentComponentState,
        children_state: &mut Self::AggregateComponentState,
        dom_state: &mut Self::DomState,
        cx: &mut Cx,
    ) -> Option<Self::Event>;
}

#[derive(Debug, PartialEq)]
pub struct LabelTarget<ParentComponentState>(pub String, pub std::marker::PhantomData<ParentComponentState>);

impl<ParentComponentState> VirtualDom<ParentComponentState> for LabelTarget<ParentComponentState> {
    type Event = ();
    type DomState = Id;
    type AggregateComponentState = ();

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Id {
        let text = &self.0;
        cx.leaf_view(crate::Label(text.clone()), Location::caller())
    }

    #[track_caller]
    fn apply_diff(&self, _other: &Self, _prev_state: Id, cx: &mut Cx) -> Id {
        let text = &self.0;
        cx.leaf_view(crate::Label(text.clone()), Location::caller())
    }

    fn process_event(
        &self,
        _explicit_state: &mut ParentComponentState,
        _children_state: &mut (),
        _dom_state: &mut Id,
        _cx: &mut Cx,
    ) -> Option<()> {
        None
    }
}

#[derive(Debug, PartialEq)]
pub struct ButtonTarget<ParentComponentState>(pub String, pub std::marker::PhantomData<ParentComponentState>);

pub struct ButtonPressed();

impl<ParentComponentState> VirtualDom<ParentComponentState> for ButtonTarget<ParentComponentState> {
    type Event = ButtonPressed;
    type DomState = Id;
    type AggregateComponentState = ();

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Id {
        let text = &self.0;
        cx.leaf_view(crate::Button(text.clone()), Location::caller())
    }

    #[track_caller]
    fn apply_diff(&self, other: &Self, prev_state: Id, cx: &mut Cx) -> Id {
        if false && &self.0 == &other.0 {
            cx.skip(1);
            prev_state
        } else {
            let text = &self.0;
            cx.leaf_view(crate::Button(text.clone()), Location::caller())
        }
    }

    fn process_event(
        &self,
        _explicit_state: &mut ParentComponentState,
        _children_state: &mut (),
        dom_state: &mut Id,
        cx: &mut Cx,
    ) -> Option<ButtonPressed> {
        let id = *dom_state;
        if cx.app_data.dequeue_action(id).is_some() {
            Some(ButtonPressed())
        } else {
            None
        }
    }
}

pub struct ElementTupleTarget<
    C0: VirtualDom<ParentComponentState>,
    C1: VirtualDom<ParentComponentState>,
    C2: VirtualDom<ParentComponentState>,
    C3: VirtualDom<ParentComponentState>,
    ParentComponentState,
>(
    pub C0,
    pub C1,
    pub C2,
    pub C3,
    pub std::marker::PhantomData<ParentComponentState>,
);

pub enum EventEnum<T0, T1, T2, T3> {
    E0(T0),
    E1(T1),
    E2(T2),
    E3(T3),
}

impl<
        C0: VirtualDom<ParentComponentState>,
        C1: VirtualDom<ParentComponentState>,
        C2: VirtualDom<ParentComponentState>,
        C3: VirtualDom<ParentComponentState>,
        ParentComponentState,
    > VirtualDom<ParentComponentState> for ElementTupleTarget<C0, C1, C2, C3, ParentComponentState>
{
    type Event = EventEnum<C0::Event, C1::Event, C2::Event, C3::Event>;
    type DomState = (C0::DomState, C1::DomState, C2::DomState, C3::DomState);
    type AggregateComponentState = (C0::AggregateComponentState, C1::AggregateComponentState, C2::AggregateComponentState, C3::AggregateComponentState);

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Self::DomState {
        (
            self.0.init_tree(cx),
            self.1.init_tree(cx),
            self.2.init_tree(cx),
            self.3.init_tree(cx),
        )
    }

    #[track_caller]
    fn apply_diff(&self, other: &Self, prev_state: Self::DomState, cx: &mut Cx) -> Self::DomState {
        (
            self.0.apply_diff(&other.0, prev_state.0, cx),
            self.1.apply_diff(&other.1, prev_state.1, cx),
            self.2.apply_diff(&other.2, prev_state.2, cx),
            self.3.apply_diff(&other.3, prev_state.3, cx),
        )
    }

    fn process_event(
        &self,
        explicit_state: &mut ParentComponentState,
        children_state: &mut Self::AggregateComponentState,
        dom_state: &mut Self::DomState,
        cx: &mut Cx,
    ) -> Option<Self::Event> {
        let event0 = self
            .0
            .process_event(explicit_state, &mut children_state.0, &mut dom_state.0, cx)
            .map(|event| EventEnum::E0(event));
        let event1 = self
            .1
            .process_event(explicit_state, &mut children_state.1, &mut dom_state.1, cx)
            .map(|event| EventEnum::E1(event));
        let event2 = self
            .2
            .process_event(explicit_state, &mut children_state.2, &mut dom_state.2, cx)
            .map(|event| EventEnum::E2(event));
        let event3 = self
            .3
            .process_event(explicit_state, &mut children_state.3, &mut dom_state.3, cx)
            .map(|event| EventEnum::E3(event));

        // FIXME - If several events happen simultaneously, this will swallow all but one
        // process_event should return an iterator or an observable instead.
        event0.or(event1).or(event2).or(event3)
    }
}

// Instead of doing multiple implementations of TupleComponent for different tuple sizes,
// I'm being lazy and doing one implem for a huge tuple, and stuffing it with EmptyElement
// when using it. It's *a lot* easier.
pub struct EmptyElementTarget<ParentComponentState>(pub std::marker::PhantomData<ParentComponentState>);

impl<ParentComponentState> VirtualDom<ParentComponentState> for EmptyElementTarget<ParentComponentState> {
    type Event = ();
    type DomState = ();
    type AggregateComponentState = ();

    fn update_value(&mut self, _other: Self) {}

    #[track_caller]
    fn init_tree(&self, _cx: &mut Cx) -> () {}

    #[track_caller]
    fn apply_diff(&self, _other: &Self, _prev_state: (), _cx: &mut Cx) -> () {}

    fn process_event(
        &self,
        _explicit_state: &mut ParentComponentState,
        _children_state: &mut (),
        _dom_state: &mut (),
        _cx: &mut Cx,
    ) -> Option<()> {
        return None;
    }
}

pub struct ElementListTarget<Comp: VirtualDom<ParentComponentState>, ParentComponentState> {
    pub elements: Vec<(String, Comp)>,
    pub _expl_state: std::marker::PhantomData<ParentComponentState>,
}

impl<Comp: VirtualDom<ParentComponentState>, ParentComponentState> ElementListTarget<Comp, ParentComponentState> {
    // We use separate functions so that crochet correctly identifies that update_value and
    // init_tree operate on the same value; it dectects this through #[track_caller]
    fn my_begin(&self, cx: &mut Cx) {
        cx.begin_view(Box::new(crate::Column), Location::caller());
    }

    fn my_end(&self, cx: &mut Cx) {
        cx.end();
    }
}

impl<Comp: VirtualDom<ParentComponentState>, ParentComponentState> VirtualDom<ParentComponentState>
    for ElementListTarget<Comp, ParentComponentState>
{
    type Event = (usize, Comp::Event);
    type DomState = Vec<Comp::DomState>;
    type AggregateComponentState = Vec<(String, Comp::AggregateComponentState)>;

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Self::DomState {
        self.my_begin(cx);

        let state = self
            .elements
            .iter()
            .map(|(_, comp)| {
                cx.begin_view(Box::new(crate::Row), Location::caller());
                let sub_state = comp.init_tree(cx);
                cx.end();
                sub_state
            })
            .collect();

        self.my_end(cx);

        state
    }

    // FIXME
    // This only works if we assume that items are ever only added at the end of the list.
    // Sounds perfectly reasonable to me.
    // (seriously though, a serious implementation would try to do whatever crochet::List::run does)
    #[track_caller]
    fn apply_diff(&self, other: &Self, prev_state: Self::DomState, cx: &mut Cx) -> Self::DomState {
        self.my_begin(cx);

        let mut updated_state: Vec<_> = other
            .elements
            .iter()
            .zip(prev_state)
            .map(|item| {
                let (other_id, other_comp) = item.0;
                let prev_comp_state = item.1;

                if let Some((_, comp)) = self.elements.iter().find(|(id, _)| id == other_id) {
                    cx.begin_view(Box::new(crate::Row), Location::caller());
                    let comp_state = comp.apply_diff(other_comp, prev_comp_state, cx);
                    cx.end();

                    Some(comp_state)
                } else {
                    cx.delete(1);
                    None
                }
            })
            .flatten()
            .collect();

        let mut new_state = self
            .elements
            .iter()
            .map(|(id, comp)| {
                if other
                    .elements
                    .iter()
                    .find(|(other_id, _)| id == other_id)
                    .is_none()
                {
                    cx.begin_insert();
                    let new_comp_state = comp.init_tree(cx);
                    cx.end();

                    Some(new_comp_state)
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        updated_state.append(&mut new_state);

        self.my_end(cx);

        updated_state
    }

    fn process_event(
        &self,
        explicit_state: &mut ParentComponentState,
        children_state: &mut Self::AggregateComponentState,
        dom_state: &mut Self::DomState,
        cx: &mut Cx,
    ) -> Option<(usize, Comp::Event)> {
        for (i, elem_data) in self.elements.iter().zip(children_state).zip(dom_state).enumerate() {
            let (_key, element) = elem_data.0.0;
            let elem_comp_state = elem_data.0.1;
            let elem_dom_state = elem_data.1;
            if let Some(event) = element.process_event(explicit_state, &mut elem_comp_state.1, elem_dom_state, cx) {
                return Some((i, event));
            }
        }
        return None;
    }
}
