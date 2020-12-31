use crate::{Cx, Id};
use crate::react_widgets::{WidgetSequence, SingleWidget, WidgetTuple, WidgetList};

// TODO - refactor away WidgetPod
use druid::WidgetPod;
use druid::widget as druid_w;

// TODO
//
// Rename VirtualDom
// Remove crate::Cx and crate::Id

pub trait VirtualDom<ParentComponentState> {
    type Event;
    type DomState;
    type AggregateComponentState: Default;

    type TargetWidget : WidgetSequence;

    // update_value is intended to enable memoize-style HOC
    // where instead of returning a vdom node, it returns
    // something along the lines of struct KeepEverythingAsItWas()
    // Ugh. I'm not explaining this well.
    fn update_value(&mut self, other: Self);

    fn init_tree(&self, _cx: &mut Cx) -> (Self::TargetWidget, Self::DomState);

    fn apply_diff(
        &self,
        other: &Self,
        prev_state: Self::DomState,
        widget: &mut Self::TargetWidget,
        _cx: &mut Cx
    ) -> Self::DomState;

    fn process_event(
        &self,
        explicit_state: &mut ParentComponentState,
        children_state: &mut Self::AggregateComponentState,
        dom_state: &mut Self::DomState,
        _cx: &mut Cx,
    ) -> Option<Self::Event>;
}

#[derive(Debug, PartialEq)]
pub struct LabelTarget<ParentComponentState>(pub String, pub std::marker::PhantomData<ParentComponentState>);

impl<ParentComponentState> VirtualDom<ParentComponentState> for LabelTarget<ParentComponentState> {
    type Event = ();
    type DomState = Id;
    type AggregateComponentState = ();

    type TargetWidget = SingleWidget<druid_w::Label<()>>;

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    fn init_tree(&self, _cx: &mut Cx) -> (Self::TargetWidget, Id) {
        let text = &self.0;
        let id = Id::new();
        let label = druid_w::Label::new(text.clone());
        (SingleWidget(WidgetPod::new(label)), id)
    }

    fn apply_diff(&self, _other: &Self, prev_state: Id, widget: &mut Self::TargetWidget, _cx: &mut Cx) -> Id {
        let text = &self.0;
        widget.0.widget_mut().set_text(text.clone());
        prev_state
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

    type TargetWidget = SingleWidget<druid_w::Button<()>>;

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    fn init_tree(&self, _cx: &mut Cx) -> (Self::TargetWidget, Id) {
        let text = &self.0;
        let id = Id::new();
        let button = druid_w::Button::new(text.clone());
        (SingleWidget(WidgetPod::new(button)), id)
    }

    fn apply_diff(&self, _other: &Self, prev_state: Self::DomState, _widget: &mut Self::TargetWidget, _cx: &mut Cx) -> Id {
        let _text = &self.0;
        //widget.set_text(text.clone());
        prev_state
    }

    fn process_event(
        &self,
        _explicit_state: &mut ParentComponentState,
        _children_state: &mut (),
        dom_state: &mut Id,
        _cx: &mut Cx,
    ) -> Option<ButtonPressed> {
        let id = *dom_state;
        if _cx.app_data.dequeue_action(id).is_some() {
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

    type TargetWidget = WidgetTuple<C0::TargetWidget, C1::TargetWidget, C2::TargetWidget, C3::TargetWidget>;

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    fn init_tree(&self, _cx: &mut Cx) -> (Self::TargetWidget, Self::DomState) {
        let (w0, s0) = self.0.init_tree(_cx);
        let (w1, s1) = self.1.init_tree(_cx);
        let (w2, s2) = self.2.init_tree(_cx);
        let (w3, s3) = self.3.init_tree(_cx);

        (WidgetTuple(w0, w1, w2, w3), (s0, s1, s2, s3))
    }

    fn apply_diff(
        &self,
        other: &Self,
        prev_state: Self::DomState,
        widget: &mut Self::TargetWidget,
        _cx: &mut Cx
    ) -> Self::DomState {
        (
            self.0.apply_diff(&other.0, prev_state.0, &mut widget.0, _cx),
            self.1.apply_diff(&other.1, prev_state.1, &mut widget.1, _cx),
            self.2.apply_diff(&other.2, prev_state.2, &mut widget.2, _cx),
            self.3.apply_diff(&other.3, prev_state.3, &mut widget.3, _cx),
        )
    }

    fn process_event(
        &self,
        explicit_state: &mut ParentComponentState,
        children_state: &mut Self::AggregateComponentState,
        dom_state: &mut Self::DomState,
        _cx: &mut Cx,
    ) -> Option<Self::Event> {
        let event0 = self
            .0
            .process_event(explicit_state, &mut children_state.0, &mut dom_state.0, _cx)
            .map(|event| EventEnum::E0(event));
        let event1 = self
            .1
            .process_event(explicit_state, &mut children_state.1, &mut dom_state.1, _cx)
            .map(|event| EventEnum::E1(event));
        let event2 = self
            .2
            .process_event(explicit_state, &mut children_state.2, &mut dom_state.2, _cx)
            .map(|event| EventEnum::E2(event));
        let event3 = self
            .3
            .process_event(explicit_state, &mut children_state.3, &mut dom_state.3, _cx)
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

    type TargetWidget = SingleWidget<druid_w::Flex<()>>;

    fn update_value(&mut self, _other: Self) {}

    fn init_tree(&self, _cx: &mut Cx) -> (Self::TargetWidget, ()) {
        (SingleWidget(WidgetPod::new(druid_w::Flex::row())), ())
    }

    fn apply_diff(&self, _other: &Self, _prev_state: (), _widget: &mut Self::TargetWidget, _cx: &mut Cx) -> () {}

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

impl<Comp: VirtualDom<ParentComponentState>, ParentComponentState> VirtualDom<ParentComponentState>
    for ElementListTarget<Comp, ParentComponentState>
{
    type Event = (usize, Comp::Event);
    type DomState = Vec<Comp::DomState>;
    type AggregateComponentState = Vec<(String, Comp::AggregateComponentState)>;

    type TargetWidget = WidgetList<Comp::TargetWidget>;

    fn update_value(&mut self, other: Self) {
        *self = other;
    }

    fn init_tree(&self, _cx: &mut Cx) -> (Self::TargetWidget, Self::DomState) {
        let (widgets, dom_state): (Vec<_>, Vec<_>) = self
            .elements
            .iter()
            .map(|(_, elem)| elem.init_tree(_cx))
            .unzip();

        (WidgetList { children: widgets }, dom_state)
    }

    // FIXME
    // This only works if we assume that items are ever only added at the end of the list.
    // Sounds perfectly reasonable to me.
    // (seriously though, a serious implementation would try to do whatever crochet::List::run does)
    fn apply_diff(
        &self, other: &Self,
        prev_state: Self::DomState,
        widget: &mut Self::TargetWidget,
        _cx: &mut Cx,
    ) -> Self::DomState {
        let mut updated_state: Vec<_> = other
            .elements
            .iter()
            .zip(prev_state)
            .map(|item| {
                let (other_id, other_elem) = item.0;
                let elem_prev_state = item.1;

                if let Some(((_, elem), ref mut widget)) = self.elements.iter()
                        .zip(widget.children.iter_mut())
                        .find(|((id, _), _)| id == other_id)
                {
                    let elem_state = elem.apply_diff(other_elem, elem_prev_state, widget, _cx);

                    Some(elem_state)
                } else {
                    _cx.delete(1);
                    None
                }
            })
            .flatten()
            .collect();

        let (mut new_widgets, mut new_state): (Vec<_>, Vec<_>)  = self
            .elements
            .iter()
            .map(|(id, elem)| {
                if other
                    .elements
                    .iter()
                    .find(|(other_id, _)| id == other_id)
                    .is_none()
                {
                    Some(elem.init_tree(_cx))
                } else {
                    None
                }
            })
            .flatten()
            .unzip();

        updated_state.append(&mut new_state);
        widget.children.append(&mut new_widgets);

        updated_state
    }

    fn process_event(
        &self,
        explicit_state: &mut ParentComponentState,
        children_state: &mut Self::AggregateComponentState,
        dom_state: &mut Self::DomState,
        _cx: &mut Cx,
    ) -> Option<(usize, Comp::Event)> {
        for (i, elem_data) in self.elements.iter().zip(children_state).zip(dom_state).enumerate() {
            let (_key, element) = elem_data.0.0;
            let elem_comp_state = elem_data.0.1;
            let elem_dom_state = elem_data.1;
            if let Some(event) = element.process_event(explicit_state, &mut elem_comp_state.1, elem_dom_state, _cx) {
                return Some((i, event));
            }
        }
        return None;
    }
}
