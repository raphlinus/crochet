use crate::{Cx, Id};

use std::panic::Location;

// TODO
//
// Improve performance
// Remove track_caller
//
// Add VirtualDom::DomRef associated type
// Implement ExplicitState for events

// TODO - box?

pub trait VirtualDom<ExplicitState> {
    type Event;
    // TODO - rename to internal state
    type State;

    // update_value is intended to enable memoize-style HOC
    // where instead of returning a vdom node, it returns
    // something along the lines of struct KeepEverythingAsItWas()
    // Ugh. I'm not explaining this well.
    fn update_value(&mut self, other: Self);

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Self::State;

    fn apply_diff(&self, other: &Self, prev_state: Self::State, cx: &mut Cx) -> Self::State;

    fn process_event(&self, explicit_state: &mut ExplicitState, state: &mut Self::State, cx: &mut Cx) -> Option<Self::Event>;
}


#[derive(Debug, PartialEq)]
pub struct VDomLabelTarget<ExplicitState>(pub String, pub std::marker::PhantomData<ExplicitState>);

impl<ExplicitState> VirtualDom<ExplicitState> for VDomLabelTarget<ExplicitState> {
    type Event = ();
    type State = Id;

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

    fn process_event(&self, _explicit_state: &mut ExplicitState, _state: &mut Id, _cx: &mut Cx) -> Option<()> {
        None
    }
}



#[derive(Debug, PartialEq)]
pub struct VDomButtonTarget<ExplicitState>(pub String, pub std::marker::PhantomData<ExplicitState>);

pub struct ButtonPressed();

impl<ExplicitState> VirtualDom<ExplicitState> for VDomButtonTarget<ExplicitState> {
    type Event = ButtonPressed;
    type State = Id;

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
        }
        else {
            let text = &self.0;
            cx.leaf_view(crate::Button(text.clone()), Location::caller())
        }
    }

    fn process_event(&self, _explicit_state: &mut ExplicitState, state: &mut Id, cx: &mut Cx) -> Option<ButtonPressed> {
        let id = *state;
        if cx.app_data.dequeue_action(id).is_some() {
            Some(ButtonPressed())
        }
        else {
            None
        }
    }
}


pub struct ComponentTupleTarget<
    C0 : VirtualDom<ExplicitState>,
    C1 : VirtualDom<ExplicitState>,
    C2 : VirtualDom<ExplicitState>,
    C3 : VirtualDom<ExplicitState>,
    ExplicitState,
>(pub C0, pub C1, pub C2, pub C3, pub std::marker::PhantomData<ExplicitState>);

pub enum EventEnum<
    T0,
    T1,
    T2,
    T3,
> {
  E0(T0),
  E1(T1),
  E2(T2),
  E3(T3),
}

impl<
    C0 : VirtualDom<ExplicitState>,
    C1 : VirtualDom<ExplicitState>,
    C2 : VirtualDom<ExplicitState>,
    C3 : VirtualDom<ExplicitState>,
    ExplicitState,
> VirtualDom<ExplicitState> for ComponentTupleTarget<C0, C1, C2, C3, ExplicitState> {
    type Event = EventEnum<
        C0::Event,
        C1::Event,
        C2::Event,
        C3::Event,
    >;
    type State = (
        C0::State,
        C1::State,
        C2::State,
        C3::State,
    );

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Self::State {
        (
            self.0.init_tree(cx),
            self.1.init_tree(cx),
            self.2.init_tree(cx),
            self.3.init_tree(cx),
        )
    }

    #[track_caller]
    fn apply_diff(&self, other: &Self, prev_state: Self::State, cx: &mut Cx) -> Self::State {
        (
            self.0.apply_diff(&other.0, prev_state.0, cx),
            self.1.apply_diff(&other.1, prev_state.1, cx),
            self.2.apply_diff(&other.2, prev_state.2, cx),
            self.3.apply_diff(&other.3, prev_state.3, cx),
        )
    }

    fn process_event(&self, explicit_state: &mut ExplicitState, state: &mut Self::State, cx: &mut Cx) -> Option<Self::Event> {
        let event0 = self.0.process_event(explicit_state, &mut state.0, cx).map(|event| EventEnum::E0(event));
        let event1 = self.1.process_event(explicit_state, &mut state.1, cx).map(|event| EventEnum::E1(event));
        let event2 = self.2.process_event(explicit_state, &mut state.2, cx).map(|event| EventEnum::E2(event));
        let event3 = self.3.process_event(explicit_state, &mut state.3, cx).map(|event| EventEnum::E3(event));

        // FIXME
        event0.or(event1).or(event2).or(event3)
    }
}


// Instead of doing multiple implementations of TupleComponent for different tuple sizes,
// I'm being lazy and doing one implem for a huge tuple, and stuffing it with EmptyComponent
// when using it. It's *a lot* easier.
pub struct EmptyComponentTarget<ExplicitState>(pub std::marker::PhantomData<ExplicitState>);

impl<ExplicitState> VirtualDom<ExplicitState> for EmptyComponentTarget<ExplicitState> {
    type Event = ();
    type State = ();

    fn update_value(&mut self, _other: Self) {}

    #[track_caller]
    fn init_tree(&self, _cx: &mut Cx) -> () {}

    #[track_caller]
    fn apply_diff(&self, _other: &Self, _prev_state: (), _cx: &mut Cx) -> () {}

    fn process_event(&self, _explicit_state: &mut ExplicitState, _state: &mut (), _cx: &mut Cx) -> Option<()> {
        return None;
    }
}


pub struct ComponentListTarget<Comp : VirtualDom<ExplicitState>, ExplicitState> {
    pub components: Vec<(String, Comp)>,
    pub _expl_state: std::marker::PhantomData<ExplicitState>,
}

impl<Comp : VirtualDom<ExplicitState>, ExplicitState> ComponentListTarget<Comp, ExplicitState> {
    // We use separate functions so that crochet correctly identifies that update_value and
    // init_tree operate on the same value; it dectects this through #[track_caller]
    fn my_begin(&self, cx: &mut Cx) {
        cx.begin_view(Box::new(crate::Column), Location::caller());
    }

    fn my_end(&self, cx: &mut Cx) {
        cx.end();
    }
}

impl<Comp : VirtualDom<ExplicitState>, ExplicitState> VirtualDom<ExplicitState> for ComponentListTarget<Comp, ExplicitState> {
    type Event = (i32, Comp::Event);
    type State = Vec<Comp::State>;

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Self::State {
        self.my_begin(cx);

        let state = self.components.iter().map(|(_, comp)| {
            cx.begin_view(Box::new(crate::Row), Location::caller());
            let sub_state = comp.init_tree(cx);
            cx.end();
            sub_state
        }).collect();

        self.my_end(cx);

        state
    }

    // This only works if we assume that items are ever only added at the end of the list.
    // Sounds perfectly reasonable to me.
    // (seriously though, a serious implementation would try to do whatever crochet::List::run does)
    #[track_caller]
    fn apply_diff(&self, other: &Self, prev_state: Self::State, cx: &mut Cx) -> Self::State {
        self.my_begin(cx);

        let mut updated_state : Vec<_> = other.components.iter().zip(prev_state).map(|item| {
                let (other_id, other_comp) = item.0;
                let prev_comp_state = item.1;

                if let Some((_, comp)) = self.components.iter().find(|(id, _)| id == other_id) {
                    cx.begin_view(Box::new(crate::Row), Location::caller());
                    let comp_state = comp.apply_diff(other_comp, prev_comp_state, cx);
                    cx.end();

                    Some(comp_state)
                }
                else {
                    cx.delete(1);
                    None
                }
            }
        ).flatten().collect();

        let mut new_state = self.components.iter().map(|(id, comp)| {
            if other.components.iter().find(|(other_id, _)| id == other_id).is_none() {
                cx.begin_insert();
                let new_comp_state = comp.init_tree(cx);
                cx.end();

                Some(new_comp_state)
            }
            else {
                None
            }
        }).flatten().collect();

        updated_state.append(&mut new_state);

        self.my_end(cx);

        updated_state
    }

    fn process_event(&self, explicit_state: &mut ExplicitState, state: &mut Self::State, cx: &mut Cx) -> Option<(i32, Comp::Event)> {
        for (i, comp_data) in self.components.iter().zip(state).enumerate() {
            let (_key, comp) = comp_data.0;
            let comp_state = comp_data.1;
            if let Some(event) = comp.process_event(explicit_state, comp_state, cx) {
                // BAAAAAD
                return Some((i as i32, event));
            }
        }
        return None;
    }
}
