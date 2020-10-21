use crate::{Cx, Id};

use std::panic::Location;


pub trait VirtualDom {
    type Event;

    // update_value is intended to enable memoize-style HOC
    // where instead of returning a vdom node, it returns
    // something along the lines of struct KeepEverythingAsItWas()
    // Ugh. I'm not explaining this well.
    fn update_value(&mut self, other: Self);

    #[track_caller]
    fn init_tree(&mut self, cx: &mut Cx);

    fn apply_diff(&mut self, other: &Self, cx: &mut Cx);

    fn process_event(&self, cx: &mut Cx) -> Option<Self::Event>;
}


#[derive(Debug, PartialEq)]
pub enum VDomLeaf {
    // this is a BIG BAD HACK, but it's 5:50 AM, so I'm cutting corners
    Button(Id, String),
    Label(Id, String),
}

pub struct ButtonPressed();

impl VirtualDom for VDomLeaf {
    type Event = ButtonPressed;

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    #[track_caller]
    fn init_tree(&mut self, cx: &mut Cx) {
        match self {
            VDomLeaf::Button(i, text) => {
                *i = cx.leaf_view(crate::Button(text.clone()), Location::caller())
            },
            VDomLeaf::Label(i, text) => {
                *i = cx.leaf_view(crate::Label(text.clone()), Location::caller())
            },
        };
    }

    #[track_caller]
    fn apply_diff(&mut self, other: &Self, cx: &mut Cx) {
        if false && &self == &other {
            cx.skip(1);
        }
        else {
            match self {
                VDomLeaf::Button(i, text) => {
                    *i = cx.leaf_view(crate::Button(text.clone()), Location::caller())
                },
                VDomLeaf::Label(i, text) => {
                    *i = cx.leaf_view(crate::Label(text.clone()), Location::caller())
                },
            };
        }
    }

    fn process_event(&self, cx: &mut Cx) -> Option<ButtonPressed> {
        let id = match &self { VDomLeaf::Button(i, _) => i, VDomLeaf::Label(i, _) => i };
        if cx.app_data.dequeue_action(*id).is_some() {
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

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    #[track_caller]
    fn init_tree(&mut self, cx: &mut Cx) {
        self.0.init_tree(cx);
        self.1.init_tree(cx);
        self.2.init_tree(cx);
        self.3.init_tree(cx);
    }

    #[track_caller]
    fn apply_diff(&mut self, other: &Self, cx: &mut Cx) {
        self.0.apply_diff(&other.0, cx);
        self.1.apply_diff(&other.1, cx);
        self.2.apply_diff(&other.2, cx);
        self.3.apply_diff(&other.3, cx);
    }

    fn process_event(&self, cx: &mut Cx) -> Option<Self::Event> {
        let event0 = self.0.process_event(cx).map(|event| EventEnum::E0(event));
        let event1 = self.1.process_event(cx).map(|event| EventEnum::E1(event));
        let event2 = self.2.process_event(cx).map(|event| EventEnum::E2(event));
        let event3 = self.3.process_event(cx).map(|event| EventEnum::E3(event));

        event0.or(event1).or(event2).or(event3)
    }
}


// Instead of doing multiple implementations of TupleComponent for different tuple sizes,
// I'm being lazy and doing one implem for a huge tuple, and stuffing it with EmptyComponent
// when using it. It's *a lot* easier.
pub struct EmptyComponent();

impl VirtualDom for EmptyComponent {
    type Event = ();

    fn update_value(&mut self, _other: Self) {}

    #[track_caller]
    fn init_tree(&mut self, _cx: &mut Cx) {}

    #[track_caller]
    fn apply_diff(&mut self, _other: &Self, _cx: &mut Cx) {}

    fn process_event(&self, _cx: &mut Cx) -> Option<()> {
        return None;
    }
}


pub struct ComponentList<Comp> {
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

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    #[track_caller]
    fn init_tree(&mut self, cx: &mut Cx) {
        self.my_begin(cx);
        for (_, comp) in &mut self.components {
            cx.begin_view(Box::new(crate::Row), Location::caller());
            comp.init_tree(cx);
            cx.end();
        }
        self.my_end(cx);
    }

    // This only works if we assume that items are ever only added at the end of the list.
    // Sounds perfectly reasonable to me.
    // (seriously though, a serious implementation would try to do whatever crochet::List::run does)
    #[track_caller]
    fn apply_diff(&mut self, other: &Self, cx: &mut Cx) {
        self.my_begin(cx);

        for (other_id, other_comp) in &other.components {
            if let Some((_, comp)) = self.components.iter_mut().find(|(id, _)| id == other_id) {
                cx.begin_view(Box::new(crate::Row), Location::caller());
                comp.apply_diff(other_comp, cx);
                cx.end();
            }
            else {
                cx.delete(1);
            }
        }

        for (id, comp) in &mut self.components {
            if other.components.iter().find(|(other_id, _)| id == other_id).is_none() {
                cx.begin_insert();
                comp.init_tree(cx);
                cx.end();
            }
        }

        self.my_end(cx);
    }

    fn process_event(&self, cx: &mut Cx) -> Option<(i32, Comp::Event)> {
        for (i, comp) in self.components.iter().enumerate() {
            if let Some(event) = comp.1.process_event(cx) {
                // BAAAAAD
                return Some((i as i32, event));
            }
        }
        return None;
    }
}


pub struct ReactComponent<Props, VDom : VirtualDom, Comp> where Comp : Fn(&Props) -> VDom {
    pub root_component: Comp,
    pub prev_vdom: Option<VDom>,
    pub _props: std::marker::PhantomData<Props>,
}

impl<Props, VDom : VirtualDom, Comp> ReactComponent<Props, VDom, Comp> where Comp : Fn(&Props) -> VDom {
    #[track_caller]
    pub fn run(&mut self, cx: &mut Cx, props: &mut Props, callback: impl Fn(&VDom::Event, &mut Props)) {
        let mut vdom = (self.root_component)(props);

        if let Some(prev_vdom) = self.prev_vdom.as_mut() {
            vdom.apply_diff(prev_vdom, cx);

            if let Some(event) = vdom.process_event(cx) {
                callback(&event, props);
            }

            prev_vdom.update_value(vdom);
        }
        else {
            vdom.init_tree(cx);
            self.prev_vdom = Some(vdom);
        }
    }
}
