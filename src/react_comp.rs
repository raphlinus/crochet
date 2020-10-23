use crate::{Cx, Id};

use std::panic::Location;

// TODO
//
// Improve performance
// Remove track_caller
//
// Add VirtualDom::DomRef associated type
// Implement ExplicitState for events

pub trait VirtualDom {
    type Event;
    type State;

    // update_value is intended to enable memoize-style HOC
    // where instead of returning a vdom node, it returns
    // something along the lines of struct KeepEverythingAsItWas()
    // Ugh. I'm not explaining this well.
    fn update_value(&mut self, other: Self);

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Self::State;

    fn apply_diff(&self, other: &Self, prev_state: Self::State, cx: &mut Cx) -> Self::State;

    fn process_event(&self, state: &mut Self::State, cx: &mut Cx) -> Option<Self::Event>;
}


#[derive(Debug, PartialEq)]
pub enum VDomLeaf {
    Button(String),
    Label(String),
}

pub struct ButtonPressed();

impl VirtualDom for VDomLeaf {
    type Event = ButtonPressed;
    type State = Id;

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    #[track_caller]
    fn init_tree(&self, cx: &mut Cx) -> Id {
        match self {
            VDomLeaf::Button(text) => {
                cx.leaf_view(crate::Button(text.clone()), Location::caller())
            },
            VDomLeaf::Label(text) => {
                cx.leaf_view(crate::Label(text.clone()), Location::caller())
            },
        }
    }

    #[track_caller]
    fn apply_diff(&self, other: &Self, prev_state: Id, cx: &mut Cx) -> Id {
        if false && &self == &other {
            cx.skip(1);
            prev_state
        }
        else {
            match self {
                VDomLeaf::Button(text) => {
                    cx.leaf_view(crate::Button(text.clone()), Location::caller())
                },
                VDomLeaf::Label(text) => {
                    cx.leaf_view(crate::Label(text.clone()), Location::caller())
                },
            }
        }
    }

    fn process_event(&self, state: &mut Id, cx: &mut Cx) -> Option<ButtonPressed> {
        let id = *state;
        if cx.app_data.dequeue_action(id).is_some() {
            Some(ButtonPressed())
        }
        else {
            None
        }
    }
}


pub struct ComponentTuple<
    C0 : VirtualDom,
    C1 : VirtualDom,
    C2 : VirtualDom,
    C3 : VirtualDom,
>(pub C0, pub C1, pub C2, pub C3);

pub enum EventEnum<
    C0 : VirtualDom,
    C1 : VirtualDom,
    C2 : VirtualDom,
    C3 : VirtualDom,
> {
  E0(C0::Event),
  E1(C1::Event),
  E2(C2::Event),
  E3(C3::Event),
}

impl<
    C0 : VirtualDom,
    C1 : VirtualDom,
    C2 : VirtualDom,
    C3 : VirtualDom,
> VirtualDom for ComponentTuple<C0, C1, C2, C3> {
    type Event = EventEnum<C0, C1, C2, C3>;
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

    fn process_event(&self, state: &mut Self::State, cx: &mut Cx) -> Option<Self::Event> {
        let event0 = self.0.process_event(&mut state.0, cx).map(|event| EventEnum::E0(event));
        let event1 = self.1.process_event(&mut state.1, cx).map(|event| EventEnum::E1(event));
        let event2 = self.2.process_event(&mut state.2, cx).map(|event| EventEnum::E2(event));
        let event3 = self.3.process_event(&mut state.3, cx).map(|event| EventEnum::E3(event));

        // FIXME
        event0.or(event1).or(event2).or(event3)
    }
}


// Instead of doing multiple implementations of TupleComponent for different tuple sizes,
// I'm being lazy and doing one implem for a huge tuple, and stuffing it with EmptyComponent
// when using it. It's *a lot* easier.
pub struct EmptyComponent();

impl VirtualDom for EmptyComponent {
    type Event = ();
    type State = ();

    fn update_value(&mut self, _other: Self) {}

    #[track_caller]
    fn init_tree(&self, _cx: &mut Cx) -> () {}

    #[track_caller]
    fn apply_diff(&self, _other: &Self, _prev_state: (), _cx: &mut Cx) -> () {}

    fn process_event(&self, _state: &mut (), _cx: &mut Cx) -> Option<()> {
        return None;
    }
}


pub struct ComponentList<Comp : VirtualDom> {
    pub components: Vec<(String, Comp)>
}

impl<Comp : VirtualDom> ComponentList<Comp> {
    // We use separate functions so that crochet correctly identifies that update_value and
    // init_tree operate on the same value; it dectects this through #[track_caller]
    fn my_begin(&self, cx: &mut Cx) {
        cx.begin_view(Box::new(crate::Column), Location::caller());
    }

    fn my_end(&self, cx: &mut Cx) {
        cx.end();
    }
}

impl<Comp : VirtualDom> VirtualDom for ComponentList<Comp> {
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

    fn process_event(&self, state: &mut Self::State, cx: &mut Cx) -> Option<(i32, Comp::Event)> {
        for (i, comp_data) in self.components.iter().zip(state).enumerate() {
            let (_key, comp) = comp_data.0;
            let comp_state = comp_data.1;
            if let Some(event) = comp.process_event(comp_state, cx) {
                // BAAAAAD
                return Some((i as i32, event));
            }
        }
        return None;
    }
}


pub struct ReactComponent<Props, VDom : VirtualDom, Cb> where Cb : Fn(&(), &Props) -> VDom {
    pub root_component: Cb,
    pub prev_vdom: Option<VDom>,
    pub prev_vdom_state: Option<VDom::State>,
    pub _props: std::marker::PhantomData<Props>,
}

impl<Props, VDom : VirtualDom, Cb> ReactComponent<Props, VDom, Cb> where Cb : Fn(&(), &Props) -> VDom {
    #[track_caller]
    pub fn run(&mut self, cx: &mut Cx, props: &mut Props, callback: impl Fn(&VDom::Event, &mut Props)) {
        let vdom = (self.root_component)(&(), props);

        if let Some(prev_vdom) = self.prev_vdom.as_mut() {
            let prev_vdom_state = self.prev_vdom_state.take().unwrap();
            let mut vdom_state = vdom.apply_diff(prev_vdom, prev_vdom_state, cx);

            if let Some(event) = vdom.process_event(&mut vdom_state, cx) {
                callback(&event, props);
            }

            prev_vdom.update_value(vdom);
            self.prev_vdom_state = Some(vdom_state);
        }
        else {
            let vdom_state = vdom.init_tree(cx);
            self.prev_vdom = Some(vdom);
            self.prev_vdom_state = Some(vdom_state);
        }
    }
}
